use crate::{
    domain::{
        conversation::{
            build_conversation, normalize_message_content, AppendMessageInput, ChatRuntimeSettings,
            ChatRuntimeSettingsInput, Conversation, ConversationListFilter, ConversationMessage,
            CreateConversationInput,
        },
        project::ProjectStatus,
    },
    error::{AppError, AppResult},
    projects::repository::ProjectRepository,
};

use super::repository::ConversationRepository;

#[derive(Clone)]
pub struct ConversationService {
    repository: ConversationRepository,
    projects: ProjectRepository,
}

impl ConversationService {
    pub fn new(repository: ConversationRepository, projects: ProjectRepository) -> Self {
        Self {
            repository,
            projects,
        }
    }

    pub fn create(&self, input: CreateConversationInput) -> AppResult<Conversation> {
        let project = self.projects.get(&input.project_id)?;
        if project.status == ProjectStatus::Archived {
            return Err(AppError::ArchivedProject);
        }
        if input.kind == crate::domain::conversation::ConversationKind::ChapterChat {
            let chapter_id = input.chapter_id.as_deref().ok_or(AppError::Validation {
                message: "Chapter chat requires a chapter.".into(),
            })?;
            self.projects
                .chapter_belongs_to_project(chapter_id, &input.project_id)?;
            self.projects.chapter_is_active(chapter_id)?;
        } else if input.chapter_id.is_some() {
            return Err(AppError::Validation {
                message: "Only chapter chat conversations may reference a chapter.".into(),
            });
        }
        self.repository.create(&build_conversation(input)?)
    }

    pub fn get(&self, id: &str) -> AppResult<Conversation> {
        self.repository.get(id)
    }

    pub fn runtime_settings(&self, conversation_id: &str) -> AppResult<ChatRuntimeSettings> {
        let conversation = self.repository.get(conversation_id)?;
        Ok(self
            .repository
            .get_runtime_settings(conversation_id)?
            .unwrap_or_else(|| ChatRuntimeSettings::defaults(conversation.updated_at)))
    }

    pub fn update_runtime_settings(
        &self,
        conversation_id: &str,
        input: ChatRuntimeSettingsInput,
    ) -> AppResult<ChatRuntimeSettings> {
        self.repository.get(conversation_id)?;
        if input.assistant_id.trim().is_empty() {
            return Err(AppError::Validation {
                message: "Assistant is required.".into(),
            });
        }
        if input.temperature.is_some_and(|temperature| {
            !temperature.is_finite() || !(0.0..=2.0).contains(&temperature)
        }) {
            return Err(AppError::Validation {
                message: "Temperature must be between 0 and 2.".into(),
            });
        }
        self.repository.upsert_runtime_settings(
            conversation_id,
            ChatRuntimeSettingsInput {
                assistant_id: input.assistant_id.trim().into(),
                provider_id: input
                    .provider_id
                    .map(|provider_id| provider_id.trim().to_string()),
                model_id: input.model_id.map(|model_id| model_id.trim().to_string()),
                ..input
            },
        )
    }

    pub fn list(
        &self,
        project_id: &str,
        filter: ConversationListFilter,
    ) -> AppResult<Vec<Conversation>> {
        self.projects.get(project_id)?;
        self.repository.list(project_id, filter)
    }

    pub fn append_message(
        &self,
        conversation_id: &str,
        input: AppendMessageInput,
    ) -> AppResult<ConversationMessage> {
        let conversation = self.repository.get(conversation_id)?;
        let project = self.projects.get(&conversation.project_id)?;
        if project.status == ProjectStatus::Archived {
            return Err(AppError::ArchivedProject);
        }
        let content = normalize_message_content(&input.content)?;
        self.repository
            .append_message(conversation_id, AppendMessageInput { content, ..input })
    }

    pub fn list_messages(
        &self,
        conversation_id: &str,
        limit: Option<u32>,
    ) -> AppResult<Vec<ConversationMessage>> {
        self.repository.list_messages(conversation_id, limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db,
        domain::{
            conversation::{ConversationKind, MessageRole},
            project::CreateProjectInput,
        },
        projects::{repository::ProjectRepository, service::ProjectService},
        provider::QualityMode,
    };

    #[test]
    fn create_and_append_requires_shared_project_connection() {
        let connection = db::in_memory().unwrap();
        let projects = ProjectService::new(ProjectRepository::new(connection.clone()));
        let conversations = ConversationService::new(
            ConversationRepository::new(connection.clone()),
            ProjectRepository::new(connection),
        );
        let project = projects
            .create(CreateProjectInput {
                name: "Chat project".into(),
                description: None,
            })
            .unwrap();
        let conversation = conversations
            .create(CreateConversationInput {
                project_id: project.id,
                chapter_id: None,
                kind: ConversationKind::DeveloperChat,
                title: "Developer chat".into(),
            })
            .unwrap();
        let first = conversations
            .append_message(
                &conversation.id,
                AppendMessageInput {
                    role: MessageRole::User,
                    content: "  Hello  ".into(),
                    model: None,
                },
            )
            .unwrap();
        let second = conversations
            .append_message(
                &conversation.id,
                AppendMessageInput {
                    role: MessageRole::Assistant,
                    content: "World".into(),
                    model: None,
                },
            )
            .unwrap();
        assert_eq!(first.sequence, 1);
        assert_eq!(second.sequence, 2);
        assert_eq!(
            conversations
                .list_messages(&conversation.id, Some(1))
                .unwrap()[0]
                .content,
            "World"
        );
    }

    #[test]
    fn archived_project_rejects_new_conversation() {
        let connection = db::in_memory().unwrap();
        let projects = ProjectService::new(ProjectRepository::new(connection.clone()));
        let conversations = ConversationService::new(
            ConversationRepository::new(connection.clone()),
            ProjectRepository::new(connection),
        );
        let project = projects
            .create(CreateProjectInput {
                name: "Archived chat".into(),
                description: None,
            })
            .unwrap();
        projects.archive(&project.id).unwrap();
        assert_eq!(
            conversations
                .create(CreateConversationInput {
                    project_id: project.id,
                    chapter_id: None,
                    kind: ConversationKind::DeveloperChat,
                    title: "No chat".into(),
                })
                .unwrap_err(),
            AppError::ArchivedProject
        );
    }

    #[test]
    fn chapter_chat_requires_a_chapter_owned_by_the_project() {
        let connection = db::in_memory().unwrap();
        let projects = ProjectService::new(ProjectRepository::new(connection.clone()));
        let conversations = ConversationService::new(
            ConversationRepository::new(connection.clone()),
            ProjectRepository::new(connection.clone()),
        );
        let project = projects
            .create(CreateProjectInput {
                name: "Scoped chat".into(),
                description: None,
            })
            .unwrap();
        assert!(matches!(
            conversations.create(CreateConversationInput {
                project_id: project.id.clone(),
                chapter_id: None,
                kind: ConversationKind::ChapterChat,
                title: "Missing chapter".into(),
            }),
            Err(AppError::Validation { .. })
        ));
        assert!(matches!(
            conversations.create(CreateConversationInput {
                project_id: project.id,
                chapter_id: Some("unknown".into()),
                kind: ConversationKind::ChapterChat,
                title: "Unknown chapter".into(),
            }),
            Err(AppError::NotFound)
        ));
    }

    #[test]
    fn chapter_chat_cannot_be_created_for_an_archived_chapter() {
        let connection = db::in_memory().unwrap();
        let project_repository = ProjectRepository::new(connection.clone());
        let projects = ProjectService::new(project_repository.clone());
        let manuscripts = crate::manuscripts::service::ManuscriptService::new(
            crate::manuscripts::repository::ManuscriptRepository::new(connection.clone()),
            project_repository.clone(),
        );
        let conversations =
            ConversationService::new(ConversationRepository::new(connection), project_repository);
        let project = projects
            .create(CreateProjectInput {
                name: "Archived chapter chat".into(),
                description: None,
            })
            .unwrap();
        let chapter = manuscripts
            .create_chapter(
                project.id.clone(),
                crate::domain::manuscript::CreateChapterInput {
                    number: 1,
                    title: "Opening".into(),
                    synopsis: None,
                },
            )
            .unwrap();
        manuscripts
            .archive_chapter(&chapter.id, chapter.revision)
            .unwrap();

        assert_eq!(
            conversations
                .create(CreateConversationInput {
                    project_id: project.id,
                    chapter_id: Some(chapter.id),
                    kind: ConversationKind::ChapterChat,
                    title: "Archived chapter chat".into(),
                })
                .unwrap_err(),
            AppError::ArchivedChapter
        );
    }

    #[test]
    fn runtime_settings_use_exact_defaults_for_new_conversations() {
        let connection = db::in_memory().unwrap();
        let projects = ProjectService::new(ProjectRepository::new(connection.clone()));
        let conversations = ConversationService::new(
            ConversationRepository::new(connection.clone()),
            ProjectRepository::new(connection),
        );
        let project = projects
            .create(CreateProjectInput {
                name: "Runtime defaults".into(),
                description: None,
            })
            .unwrap();
        let conversation = conversations
            .create(CreateConversationInput {
                project_id: project.id,
                chapter_id: None,
                kind: ConversationKind::DeveloperChat,
                title: "Runtime defaults chat".into(),
            })
            .unwrap();

        let settings = conversations.runtime_settings(&conversation.id).unwrap();
        assert_eq!(settings.assistant_id, "general-assistant");
        assert_eq!(settings.provider_id, None);
        assert_eq!(settings.model_id, None);
        assert_eq!(settings.quality, QualityMode::Balanced);
        assert_eq!(settings.temperature, None);
        assert!(!settings.updated_at.is_empty());
    }

    #[test]
    fn runtime_settings_update_persists_choices_and_validates_input() {
        let connection = db::in_memory().unwrap();
        let projects = ProjectService::new(ProjectRepository::new(connection.clone()));
        let conversations = ConversationService::new(
            ConversationRepository::new(connection.clone()),
            ProjectRepository::new(connection),
        );
        let project = projects
            .create(CreateProjectInput {
                name: "Runtime update".into(),
                description: None,
            })
            .unwrap();
        let conversation = conversations
            .create(CreateConversationInput {
                project_id: project.id,
                chapter_id: None,
                kind: ConversationKind::DeveloperChat,
                title: "Runtime update chat".into(),
            })
            .unwrap();

        let updated = conversations
            .update_runtime_settings(
                &conversation.id,
                crate::domain::conversation::ChatRuntimeSettingsInput {
                    assistant_id: "writing-coach".into(),
                    provider_id: Some("openai".into()),
                    model_id: Some("gpt-5".into()),
                    quality: QualityMode::Deep,
                    temperature: Some(0.7),
                },
            )
            .unwrap();
        assert_eq!(updated.assistant_id, "writing-coach");
        assert_eq!(updated.provider_id.as_deref(), Some("openai"));
        assert_eq!(updated.model_id.as_deref(), Some("gpt-5"));
        assert_eq!(updated.quality, QualityMode::Deep);
        assert_eq!(updated.temperature, Some(0.7));
        assert_eq!(
            conversations.runtime_settings(&conversation.id).unwrap(),
            updated
        );

        assert!(matches!(
            conversations.update_runtime_settings(
                &conversation.id,
                crate::domain::conversation::ChatRuntimeSettingsInput {
                    assistant_id: "  ".into(),
                    provider_id: None,
                    model_id: None,
                    quality: QualityMode::Balanced,
                    temperature: None,
                },
            ),
            Err(AppError::Validation { .. })
        ));
        assert!(matches!(
            conversations.update_runtime_settings(
                &conversation.id,
                crate::domain::conversation::ChatRuntimeSettingsInput {
                    assistant_id: "general-assistant".into(),
                    provider_id: None,
                    model_id: None,
                    quality: QualityMode::Balanced,
                    temperature: Some(2.1),
                },
            ),
            Err(AppError::Validation { .. })
        ));
    }

    #[test]
    fn runtime_settings_missing_conversation_is_not_found() {
        let connection = db::in_memory().unwrap();
        let conversations = ConversationService::new(
            ConversationRepository::new(connection.clone()),
            ProjectRepository::new(connection),
        );

        assert_eq!(
            conversations
                .runtime_settings("missing-conversation")
                .unwrap_err(),
            AppError::NotFound
        );
    }
}

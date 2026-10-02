use crate::{
    domain::{
        conversation::{
            build_conversation, normalize_message_content, AppendMessageInput, Conversation,
            ConversationListFilter, ConversationMessage, CreateConversationInput,
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
}

use serde::{Deserialize, Serialize};

use crate::{
    context::{ContextBudget, ContextCompiler, ServiceContextSource, WorkingMemoryBlock},
    domain::conversation::{
        AppendMessageInput, Conversation, ConversationKind, ConversationMessage, MessageRole,
    },
    error::{AppError, AppResult},
    manuscripts::service::ManuscriptService,
    orchestration::{NarrativeOrchestrator, OrchestrationRequest, OrchestrationResult},
    provider::{ModelRef, ModelTask, ProviderCapabilities, ProviderRegistry, QualityMode},
};

use super::service::ConversationService;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeveloperChatSendRequest {
    pub conversation_id: String,
    pub task: ModelTask,
    pub quality: QualityMode,
    pub preferred_model: Option<ModelRef>,
    pub required_capabilities: ProviderCapabilities,
    pub minimum_context_window_tokens: Option<u32>,
    pub system_instructions: String,
    pub character_ids: Vec<String>,
    pub include_character_states: bool,
    pub context_budget: ContextBudget,
    pub message: String,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeveloperChatSendResult {
    pub conversation: Conversation,
    pub user_message: ConversationMessage,
    pub assistant_message: ConversationMessage,
    pub orchestration: OrchestrationResult,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DeveloperChatService;

impl DeveloperChatService {
    pub async fn send(
        &self,
        conversations: &ConversationService,
        registry: &ProviderRegistry,
        compiler: &ContextCompiler,
        context_source: &ServiceContextSource,
        request: DeveloperChatSendRequest,
    ) -> AppResult<DeveloperChatSendResult> {
        Self::send_for_kind(
            conversations,
            registry,
            compiler,
            context_source,
            request,
            ConversationKind::DeveloperChat,
            Vec::new(),
        )
        .await
    }

    pub async fn send_for_kind(
        conversations: &ConversationService,
        registry: &ProviderRegistry,
        compiler: &ContextCompiler,
        context_source: &ServiceContextSource,
        request: DeveloperChatSendRequest,
        expected_kind: ConversationKind,
        additional_working_memory: Vec<WorkingMemoryBlock>,
    ) -> AppResult<DeveloperChatSendResult> {
        let conversation = conversations.get(&request.conversation_id)?;
        if conversation.kind != expected_kind {
            return Err(AppError::Validation {
                message: "This conversation kind does not accept the requested command.".into(),
            });
        }
        let previous_messages = conversations.list_messages(&conversation.id, Some(20))?;
        let user_message = conversations.append_message(
            &conversation.id,
            AppendMessageInput {
                role: MessageRole::User,
                content: request.message.clone(),
                model: None,
            },
        )?;
        let mut working_memory = previous_messages
            .iter()
            .map(|message| WorkingMemoryBlock {
                id: format!("conversation:{}", message.id),
                label: format!("{} message", message.role.as_str()),
                content: message.content.clone(),
            })
            .collect::<Vec<_>>();
        working_memory.extend(additional_working_memory);
        let orchestrator_request = OrchestrationRequest {
            project_id: conversation.project_id.clone(),
            task: request.task,
            quality: request.quality,
            preferred_model: request.preferred_model,
            required_capabilities: request.required_capabilities,
            minimum_context_window_tokens: request.minimum_context_window_tokens,
            system_instructions: request.system_instructions,
            character_ids: request.character_ids,
            include_character_states: request.include_character_states,
            working_memory,
            context_budget: request.context_budget,
            user_prompt: request.message,
            temperature: request.temperature,
        };
        let orchestration = NarrativeOrchestrator
            .run(registry, compiler, context_source, orchestrator_request)
            .await?;
        let final_step = orchestration.steps.last().ok_or(AppError::Internal)?;
        let assistant_message = conversations.append_message(
            &conversation.id,
            AppendMessageInput {
                role: MessageRole::Assistant,
                content: final_step.response.text.clone(),
                model: Some(final_step.route.model.clone()),
            },
        )?;
        Ok(DeveloperChatSendResult {
            conversation: conversations.get(&conversation.id)?,
            user_message,
            assistant_message,
            orchestration,
        })
    }
}

#[derive(Clone)]
pub struct ChapterChatService;

impl ChapterChatService {
    pub async fn send(
        conversations: &ConversationService,
        manuscripts: &ManuscriptService,
        registry: &ProviderRegistry,
        compiler: &ContextCompiler,
        context_source: &ServiceContextSource,
        chapter_id: &str,
        request: DeveloperChatSendRequest,
    ) -> AppResult<DeveloperChatSendResult> {
        let conversation = conversations.get(&request.conversation_id)?;
        if conversation.kind != ConversationKind::ChapterChat
            || conversation.chapter_id.as_deref() != Some(chapter_id)
        {
            return Err(AppError::Validation {
                message: "Chapter chat must belong to the requested chapter.".into(),
            });
        }
        let manuscript = manuscripts.get_manuscript(chapter_id)?;
        DeveloperChatService::send_for_kind(
            conversations,
            registry,
            compiler,
            context_source,
            request,
            ConversationKind::ChapterChat,
            vec![WorkingMemoryBlock {
                id: format!("manuscript:{chapter_id}"),
                label: "Current manuscript".into(),
                content: manuscript.content,
            }],
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use futures::executor::block_on;

    use super::*;
    use crate::{
        characters::{repository::CharacterRepository, service::CharacterService},
        db,
        domain::{
            conversation::{ConversationKind, CreateConversationInput, MessageRole},
            manuscript::CreateChapterInput,
            project::CreateProjectInput,
        },
        manuscripts::{repository::ManuscriptRepository, service::ManuscriptService},
        projects::{repository::ProjectRepository, service::ProjectService},
        provider::{MockProvider, ModelProfile, ProviderDescriptor},
    };

    fn profile() -> ModelProfile {
        ModelProfile {
            provider_id: "mock".into(),
            model_id: "writer".into(),
            display_name: "Writer".into(),
            context_window_tokens: 8192,
            default_output_tokens: 512,
            strengths: vec!["planning".into(), "prose".into()],
            weaknesses: Vec::new(),
            strategy: Vec::new(),
            tier: crate::provider::ModelTier::Medium,
            capabilities: ProviderCapabilities::default(),
        }
    }

    fn request(conversation_id: String) -> DeveloperChatSendRequest {
        DeveloperChatSendRequest {
            conversation_id,
            task: ModelTask::MainWriting,
            quality: QualityMode::Fast,
            preferred_model: None,
            required_capabilities: ProviderCapabilities::default(),
            minimum_context_window_tokens: None,
            system_instructions: "Stay within canon.".into(),
            character_ids: Vec::new(),
            include_character_states: false,
            context_budget: ContextBudget::default(),
            message: "Draft the next scene.".into(),
            temperature: None,
        }
    }

    fn setup() -> (
        ConversationService,
        ServiceContextSource,
        ProviderRegistry,
        String,
    ) {
        let connection = db::in_memory().unwrap();
        let projects = ProjectService::new(ProjectRepository::new(connection.clone()));
        let characters = CharacterService::new(CharacterRepository::new(connection.clone()));
        let conversations = ConversationService::new(
            super::super::repository::ConversationRepository::new(connection.clone()),
            ProjectRepository::new(connection),
        );
        let project = projects
            .create(CreateProjectInput {
                name: "Developer Chat project".into(),
                description: Some("Premise".into()),
            })
            .unwrap();
        let conversation = conversations
            .create(CreateConversationInput {
                project_id: project.id.clone(),
                chapter_id: None,
                kind: ConversationKind::DeveloperChat,
                title: "Developer Chat".into(),
            })
            .unwrap();
        let provider = Arc::new(MockProvider::new(
            ProviderDescriptor {
                id: "mock".into(),
                display_name: "Mock".into(),
            },
            vec![profile()],
        ));
        let registry = ProviderRegistry::new();
        registry.register(provider).unwrap();
        (
            conversations,
            ServiceContextSource::new(projects, characters),
            registry,
            conversation.id,
        )
    }

    #[test]
    fn send_persists_user_and_final_assistant_turn() {
        let (conversations, source, registry, conversation_id) = setup();
        conversations
            .append_message(
                &conversation_id,
                AppendMessageInput {
                    role: MessageRole::User,
                    content: "Earlier idea".into(),
                    model: None,
                },
            )
            .unwrap();
        let result = block_on(DeveloperChatService.send(
            &conversations,
            &registry,
            &crate::context::ContextCompiler::default(),
            &source,
            request(conversation_id.clone()),
        ))
        .unwrap();
        assert_eq!(result.user_message.role, MessageRole::User);
        assert_eq!(result.assistant_message.role, MessageRole::Assistant);
        assert_eq!(result.orchestration.steps.len(), 3);
        assert!(result.orchestration.steps[0]
            .context
            .blocks
            .iter()
            .any(|block| block.kind == crate::context::ContextBlockKind::WorkingMemory));
        let messages = conversations.list_messages(&conversation_id, None).unwrap();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[2].model.as_ref().unwrap().model_id, "writer");
    }

    #[test]
    fn provider_failure_leaves_user_turn_without_partial_assistant() {
        let (conversations, source, _registry, conversation_id) = setup();
        let registry = ProviderRegistry::new();
        let error = block_on(DeveloperChatService.send(
            &conversations,
            &registry,
            &crate::context::ContextCompiler::default(),
            &source,
            request(conversation_id.clone()),
        ))
        .unwrap_err();
        assert_eq!(error, AppError::NoSuitableModel);
        let messages = conversations.list_messages(&conversation_id, None).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, MessageRole::User);
    }

    #[test]
    fn chapter_chat_scopes_conversation_and_includes_current_manuscript() {
        let connection = db::in_memory().unwrap();
        let projects = ProjectService::new(ProjectRepository::new(connection.clone()));
        let characters = CharacterService::new(CharacterRepository::new(connection.clone()));
        let project = projects
            .create(CreateProjectInput {
                name: "Chapter Chat project".into(),
                description: None,
            })
            .unwrap();
        let manuscripts = ManuscriptService::new(
            ManuscriptRepository::new(connection.clone()),
            ProjectRepository::new(connection.clone()),
        );
        let chapter = manuscripts
            .create_chapter(
                project.id.clone(),
                CreateChapterInput {
                    number: 1,
                    title: "Opening".into(),
                    synopsis: None,
                },
            )
            .unwrap();
        manuscripts
            .save_manuscript(
                &chapter.id,
                crate::domain::manuscript::SaveManuscriptInput {
                    content: "The gate waited.".into(),
                    content_format: None,
                    label: Some("Draft".into()),
                    actor_type: None,
                    actor_id: None,
                    expected_revision: 1,
                },
            )
            .unwrap();
        let conversations = ConversationService::new(
            super::super::repository::ConversationRepository::new(connection.clone()),
            ProjectRepository::new(connection),
        );
        let conversation = conversations
            .create(CreateConversationInput {
                project_id: project.id,
                chapter_id: Some(chapter.id.clone()),
                kind: ConversationKind::ChapterChat,
                title: "Chapter Chat".into(),
            })
            .unwrap();
        let provider = Arc::new(MockProvider::new(
            ProviderDescriptor {
                id: "mock".into(),
                display_name: "Mock".into(),
            },
            vec![profile()],
        ));
        let registry = ProviderRegistry::new();
        registry.register(provider).unwrap();
        let source = ServiceContextSource::new(projects, characters);
        let result = block_on(ChapterChatService::send(
            &conversations,
            &manuscripts,
            &registry,
            &crate::context::ContextCompiler::default(),
            &source,
            &chapter.id,
            request(conversation.id.clone()),
        ))
        .unwrap();
        assert_eq!(
            result.conversation.chapter_id.as_deref(),
            Some(chapter.id.as_str())
        );
        assert!(result.orchestration.steps[0]
            .context
            .blocks
            .iter()
            .any(|block| block.content.contains("The gate waited.")));
    }
}

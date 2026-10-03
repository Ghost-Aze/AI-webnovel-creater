use serde::{Deserialize, Serialize};

use crate::{
    context::{ContextBudget, ContextCompiler, ContextSource, ServiceContextSource},
    domain::conversation::{
        ChatRuntimeSettings, ChatRuntimeSettingsInput, Conversation, ConversationKind,
        ConversationListFilter, ConversationMessage,
    },
    domain::project::now_utc,
    error::{AppError, AppResult},
    manuscripts::service::ManuscriptService,
    provider::{ModelProfile, ModelRef, ModelTask, ProviderCapabilities, ProviderRegistry},
};

use super::{
    chat::{
        ChapterChatService, DeveloperChatSendRequest, DeveloperChatSendResult, DeveloperChatService,
    },
    service::ConversationService,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatRuntimeLoadRequest {
    pub project_id: String,
    pub chapter_id: Option<String>,
    pub conversation_id: Option<String>,
    pub kind: ConversationKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatAssistantDescriptor {
    pub id: String,
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatRuntimeSnapshot {
    pub conversations: Vec<Conversation>,
    pub conversation: Option<Conversation>,
    pub messages: Vec<ConversationMessage>,
    pub settings: ChatRuntimeSettings,
    pub assistants: Vec<ChatAssistantDescriptor>,
    pub models: Vec<ModelProfile>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatSendRequest {
    pub conversation_id: String,
    pub task: ModelTask,
    pub runtime: ChatRuntimeSettingsInput,
    pub system_instructions: String,
    pub character_ids: Vec<String>,
    pub include_character_states: bool,
    pub context_budget: ContextBudget,
    pub message: String,
    pub retry_attempt: bool,
}

pub type ChatSendResult = DeveloperChatSendResult;

#[derive(Clone)]
pub struct ChatRuntimeService {
    conversations: ConversationService,
    manuscripts: ManuscriptService,
    registry: ProviderRegistry,
    compiler: ContextCompiler,
    context_source: ServiceContextSource,
}

impl ChatRuntimeService {
    pub fn new(
        conversations: ConversationService,
        manuscripts: ManuscriptService,
        registry: ProviderRegistry,
        compiler: ContextCompiler,
        context_source: ServiceContextSource,
    ) -> Self {
        Self {
            conversations,
            manuscripts,
            registry,
            compiler,
            context_source,
        }
    }

    pub fn load(&self, request: ChatRuntimeLoadRequest) -> AppResult<ChatRuntimeSnapshot> {
        self.validate_scope(
            &request.project_id,
            request.chapter_id.as_deref(),
            request.kind,
        )?;
        let mut conversations = self.conversations.list(
            &request.project_id,
            ConversationListFilter {
                kind: Some(request.kind),
            },
        )?;
        if let Some(chapter_id) = request.chapter_id.as_deref() {
            conversations
                .retain(|conversation| conversation.chapter_id.as_deref() == Some(chapter_id));
        }
        let conversation = match request.conversation_id.as_deref() {
            Some(conversation_id) => Some(
                conversations
                    .iter()
                    .find(|conversation| conversation.id == conversation_id)
                    .cloned()
                    .ok_or(AppError::NotFound)?,
            ),
            None => conversations.first().cloned(),
        };
        let messages = conversation
            .as_ref()
            .map(|conversation| self.conversations.list_messages(&conversation.id, None))
            .transpose()?
            .unwrap_or_default();
        let settings = conversation
            .as_ref()
            .map(|conversation| self.conversations.runtime_settings(&conversation.id))
            .transpose()?
            .unwrap_or_else(|| ChatRuntimeSettings::defaults(now_utc()));
        self.validate_runtime_settings(&settings)?;
        Ok(ChatRuntimeSnapshot {
            conversations,
            conversation,
            messages,
            settings,
            assistants: assistant_catalog(),
            models: self.registry.list_models(None).map_err(AppError::from)?,
        })
    }

    pub fn update_settings(
        &self,
        conversation_id: &str,
        input: ChatRuntimeSettingsInput,
    ) -> AppResult<ChatRuntimeSettings> {
        self.validate_runtime_settings(&ChatRuntimeSettings {
            assistant_id: input.assistant_id.clone(),
            provider_id: input.provider_id.clone(),
            model_id: input.model_id.clone(),
            quality: input.quality,
            temperature: input.temperature,
            updated_at: now_utc(),
        })?;
        self.conversations
            .update_runtime_settings(conversation_id, input)
    }

    pub async fn send(&self, request: ChatSendRequest) -> AppResult<ChatSendResult> {
        let conversation = self.conversations.get(&request.conversation_id)?;
        let chapter_id = self.validate_send_scope(&conversation)?;
        self.validate_runtime_settings(&ChatRuntimeSettings {
            assistant_id: request.runtime.assistant_id.clone(),
            provider_id: request.runtime.provider_id.clone(),
            model_id: request.runtime.model_id.clone(),
            quality: request.runtime.quality,
            temperature: request.runtime.temperature,
            updated_at: now_utc(),
        })?;
        let legacy_request = DeveloperChatSendRequest {
            conversation_id: request.conversation_id,
            task: request.task,
            quality: request.runtime.quality,
            preferred_model: preferred_model(&request.runtime),
            required_capabilities: ProviderCapabilities::default(),
            minimum_context_window_tokens: None,
            system_instructions: request.system_instructions,
            character_ids: request.character_ids,
            include_character_states: request.include_character_states,
            context_budget: request.context_budget,
            message: request.message,
            temperature: request.runtime.temperature,
            retry_attempt: request.retry_attempt,
        };
        match chapter_id {
            Some(chapter_id) => {
                ChapterChatService::send(
                    &self.conversations,
                    &self.manuscripts,
                    &self.registry,
                    &self.compiler,
                    &self.context_source,
                    &chapter_id,
                    legacy_request,
                )
                .await
            }
            None => {
                DeveloperChatService
                    .send(
                        &self.conversations,
                        &self.registry,
                        &self.compiler,
                        &self.context_source,
                        legacy_request,
                    )
                    .await
            }
        }
    }

    pub async fn send_legacy(
        &self,
        request: DeveloperChatSendRequest,
        expected_chapter_id: Option<String>,
    ) -> AppResult<ChatSendResult> {
        let conversation = self.conversations.get(&request.conversation_id)?;
        if let Some(chapter_id) = expected_chapter_id {
            if conversation.chapter_id.as_deref() != Some(chapter_id.as_str()) {
                return Err(AppError::Validation {
                    message: "Chapter chat must belong to the requested chapter.".into(),
                });
            }
        } else if conversation.kind != ConversationKind::DeveloperChat {
            return Err(AppError::Validation {
                message: "This conversation kind does not accept the requested command.".into(),
            });
        }
        self.send(ChatSendRequest {
            conversation_id: request.conversation_id,
            task: request.task,
            runtime: ChatRuntimeSettingsInput {
                assistant_id: "general-assistant".into(),
                provider_id: request
                    .preferred_model
                    .as_ref()
                    .map(|model| model.provider_id.clone()),
                model_id: request
                    .preferred_model
                    .as_ref()
                    .map(|model| model.model_id.clone()),
                quality: request.quality,
                temperature: request.temperature,
            },
            system_instructions: request.system_instructions,
            character_ids: request.character_ids,
            include_character_states: request.include_character_states,
            context_budget: request.context_budget,
            message: request.message,
            retry_attempt: request.retry_attempt,
        })
        .await
    }

    fn validate_scope(
        &self,
        project_id: &str,
        chapter_id: Option<&str>,
        kind: ConversationKind,
    ) -> AppResult<()> {
        self.context_source.load_project(project_id)?;
        if kind == ConversationKind::ChapterChat {
            let chapter_id = chapter_id.ok_or_else(|| AppError::Validation {
                message: "Chapter chat requires a chapter.".into(),
            })?;
            let chapter = self.manuscripts.get_chapter(chapter_id)?;
            if chapter.project_id != project_id {
                return Err(AppError::NotFound);
            }
        } else if chapter_id.is_some() {
            return Err(AppError::Validation {
                message: "Only chapter chat scopes may reference a chapter.".into(),
            });
        }
        Ok(())
    }

    fn validate_send_scope(&self, conversation: &Conversation) -> AppResult<Option<String>> {
        let project = self.context_source.load_project(&conversation.project_id)?;
        if project.status == crate::domain::project::ProjectStatus::Archived {
            return Err(AppError::ArchivedProject);
        }
        if conversation.kind != ConversationKind::ChapterChat {
            return Ok(None);
        }
        let chapter_id =
            conversation
                .chapter_id
                .as_deref()
                .ok_or_else(|| AppError::Validation {
                    message: "Chapter chat requires a chapter.".into(),
                })?;
        let chapter = self.manuscripts.get_chapter(chapter_id)?;
        if chapter.project_id != conversation.project_id {
            return Err(AppError::NotFound);
        }
        if chapter.status == crate::domain::manuscript::ChapterStatus::Archived {
            return Err(AppError::ArchivedChapter);
        }
        Ok(Some(chapter_id.into()))
    }

    fn validate_runtime_settings(&self, settings: &ChatRuntimeSettings) -> AppResult<()> {
        if !assistant_catalog()
            .iter()
            .any(|assistant| assistant.id == settings.assistant_id)
        {
            return Err(AppError::Validation {
                message: "The requested assistant is not available.".into(),
            });
        }
        if settings.provider_id.is_some() != settings.model_id.is_some() {
            return Err(AppError::Validation {
                message: "Provider and model must be selected together.".into(),
            });
        }
        if let (Some(provider_id), Some(model_id)) = (
            settings.provider_id.as_deref(),
            settings.model_id.as_deref(),
        ) {
            self.registry
                .resolve(provider_id, model_id)
                .map_err(AppError::from)?;
        }
        if settings.temperature.is_some_and(|temperature| {
            !temperature.is_finite() || !(0.0..=2.0).contains(&temperature)
        }) {
            return Err(AppError::Validation {
                message: "Temperature must be between 0 and 2.".into(),
            });
        }
        Ok(())
    }
}

fn preferred_model(settings: &ChatRuntimeSettingsInput) -> Option<ModelRef> {
    settings
        .provider_id
        .as_ref()
        .zip(settings.model_id.as_ref())
        .map(|(provider_id, model_id)| ModelRef {
            provider_id: provider_id.clone(),
            model_id: model_id.clone(),
        })
}

fn assistant_catalog() -> Vec<ChatAssistantDescriptor> {
    vec![
        ChatAssistantDescriptor {
            id: "general-assistant".into(),
            label: "General Assistant".into(),
            description: "A balanced writing and planning assistant.".into(),
        },
        ChatAssistantDescriptor {
            id: "world-builder".into(),
            label: "World Builder".into(),
            description: "Helps shape settings, rules and story facts.".into(),
        },
        ChatAssistantDescriptor {
            id: "continuity-reviewer".into(),
            label: "Continuity Reviewer".into(),
            description: "Reviews plot and character consistency.".into(),
        },
        ChatAssistantDescriptor {
            id: "writing-coach".into(),
            label: "Writing Coach".into(),
            description: "Offers focused prose and revision guidance.".into(),
        },
    ]
}

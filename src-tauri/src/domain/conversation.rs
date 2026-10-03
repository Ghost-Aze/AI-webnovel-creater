use serde::{Deserialize, Serialize};

use crate::provider::{ModelRef, QualityMode};

use super::project::{new_id, now_utc};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationKind {
    #[default]
    DeveloperChat,
    ArcChat,
    ChapterChat,
}

impl ConversationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DeveloperChat => "developer_chat",
            Self::ArcChat => "arc_chat",
            Self::ChapterChat => "chapter_chat",
        }
    }
}

impl TryFrom<&str> for ConversationKind {
    type Error = crate::error::AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "developer_chat" => Ok(Self::DeveloperChat),
            "arc_chat" => Ok(Self::ArcChat),
            "chapter_chat" => Ok(Self::ChapterChat),
            _ => Err(crate::error::AppError::InvalidStatus),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

impl MessageRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
        }
    }
}

impl TryFrom<&str> for MessageRole {
    type Error = crate::error::AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "system" => Ok(Self::System),
            "user" => Ok(Self::User),
            "assistant" => Ok(Self::Assistant),
            "tool" => Ok(Self::Tool),
            _ => Err(crate::error::AppError::InvalidStatus),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub project_id: String,
    pub chapter_id: Option<String>,
    pub kind: ConversationKind,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub id: String,
    pub conversation_id: String,
    pub sequence: u64,
    pub role: MessageRole,
    pub content: String,
    pub model: Option<ModelRef>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatRuntimeSettings {
    pub assistant_id: String,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub quality: QualityMode,
    pub temperature: Option<f32>,
    pub updated_at: String,
}

impl ChatRuntimeSettings {
    pub fn defaults(updated_at: String) -> Self {
        Self {
            assistant_id: "general-assistant".into(),
            provider_id: None,
            model_id: None,
            quality: QualityMode::Balanced,
            temperature: None,
            updated_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatRuntimeSettingsInput {
    pub assistant_id: String,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub quality: QualityMode,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateConversationInput {
    pub project_id: String,
    #[serde(default)]
    pub chapter_id: Option<String>,
    #[serde(default)]
    pub kind: ConversationKind,
    pub title: String,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub struct ConversationListFilter {
    #[serde(default)]
    pub kind: Option<ConversationKind>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppendMessageInput {
    pub role: MessageRole,
    pub content: String,
    #[serde(default)]
    pub model: Option<ModelRef>,
}

pub fn build_conversation(input: CreateConversationInput) -> crate::error::AppResult<Conversation> {
    let title = input.title.trim();
    if input.project_id.trim().is_empty() || title.is_empty() {
        return Err(crate::error::AppError::Validation {
            message: "Conversation project and title are required.".into(),
        });
    }
    let timestamp = now_utc();
    Ok(Conversation {
        id: new_id(),
        project_id: input.project_id,
        chapter_id: input.chapter_id,
        kind: input.kind,
        title: title.into(),
        created_at: timestamp.clone(),
        updated_at: timestamp,
    })
}

pub fn normalize_message_content(content: &str) -> crate::error::AppResult<String> {
    let content = content.trim();
    if content.is_empty() {
        return Err(crate::error::AppError::Validation {
            message: "Message content is required.".into(),
        });
    }
    Ok(content.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversation_and_message_values_use_stable_wire_names() {
        assert_eq!(ConversationKind::DeveloperChat.as_str(), "developer_chat");
        assert_eq!(MessageRole::Assistant.as_str(), "assistant");
        assert_eq!(
            serde_json::to_string(&ConversationKind::ChapterChat).unwrap(),
            "\"chapter_chat\""
        );
    }

    #[test]
    fn builders_reject_blank_values_and_trim_valid_content() {
        assert!(build_conversation(CreateConversationInput {
            project_id: "project".into(),
            chapter_id: None,
            kind: ConversationKind::DeveloperChat,
            title: "  ".into(),
        })
        .is_err());
        assert_eq!(normalize_message_content("  hello  ").unwrap(), "hello");
        assert!(normalize_message_content("  ").is_err());
    }
}

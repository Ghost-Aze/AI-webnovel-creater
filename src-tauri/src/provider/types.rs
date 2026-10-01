use std::pin::Pin;

use async_trait::async_trait;
use futures_core::Stream;
use serde::{Deserialize, Serialize};

use super::error::{ProviderError, ProviderResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderDescriptor {
    pub id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub streaming: bool,
    pub embeddings: bool,
    pub tools: bool,
    pub vision: bool,
    pub structured_output: bool,
    pub prompt_caching: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelTier {
    Local,
    Small,
    #[default]
    Medium,
    Large,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelProfile {
    pub provider_id: String,
    pub model_id: String,
    pub display_name: String,
    pub context_window_tokens: u32,
    pub default_output_tokens: u32,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub strategy: Vec<String>,
    #[serde(default)]
    pub tier: ModelTier,
    pub capabilities: ProviderCapabilities,
}

impl ModelProfile {
    pub fn validate(&self) -> ProviderResult<()> {
        if self.provider_id.trim().is_empty()
            || self.model_id.trim().is_empty()
            || self.display_name.trim().is_empty()
            || self.default_output_tokens == 0
            || self.context_window_tokens <= self.default_output_tokens
        {
            return Err(ProviderError::InvalidRequest);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelRef {
    pub provider_id: String,
    pub model_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptMessage {
    pub role: PromptRole,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub model: ModelRef,
    pub messages: Vec<PromptMessage>,
    pub max_output_tokens: u32,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderUsage {
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub model: ModelRef,
    pub text: String,
    pub usage: ProviderUsage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbedRequest {
    pub model: ModelRef,
    pub inputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbedResponse {
    pub model: ModelRef,
    pub vectors: Vec<Vec<f32>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamEvent {
    pub text_delta: String,
    pub done: bool,
}

pub type ProviderStream = Pin<Box<dyn Stream<Item = ProviderResult<StreamEvent>> + Send>>;

#[async_trait]
pub trait AIProvider: Send + Sync {
    fn descriptor(&self) -> ProviderDescriptor;
    fn capabilities(&self) -> ProviderCapabilities;
    fn list_models(&self) -> ProviderResult<Vec<ModelProfile>>;

    async fn generate(&self, request: GenerateRequest) -> ProviderResult<GenerateResponse>;

    async fn stream(&self, _request: GenerateRequest) -> ProviderResult<ProviderStream> {
        Err(ProviderError::UnsupportedCapability {
            capability: "streaming".into(),
        })
    }

    async fn embed(&self, _request: EmbedRequest) -> ProviderResult<EmbedResponse> {
        Err(ProviderError::UnsupportedCapability {
            capability: "embeddings".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn profile() -> ModelProfile {
        ModelProfile {
            provider_id: "mock".into(),
            model_id: "mock-small".into(),
            display_name: "Mock Small".into(),
            context_window_tokens: 4096,
            default_output_tokens: 512,
            strengths: vec!["prose".into()],
            weaknesses: vec!["continuity".into()],
            strategy: vec!["use plans".into()],
            tier: ModelTier::Small,
            capabilities: ProviderCapabilities {
                streaming: false,
                embeddings: false,
                tools: false,
                vision: false,
                structured_output: true,
                prompt_caching: false,
            },
        }
    }

    #[test]
    fn rejects_blank_model_ids() {
        let mut value = profile();
        value.provider_id.clear();
        assert!(value.validate().is_err());

        let mut value = profile();
        value.model_id = "  ".into();
        assert!(value.validate().is_err());
    }

    #[test]
    fn rejects_context_window_without_output_room() {
        let mut value = profile();
        value.context_window_tokens = value.default_output_tokens;
        assert!(value.validate().is_err());
    }

    #[test]
    fn serializes_capabilities_and_model_reference() {
        let capabilities = ProviderCapabilities {
            streaming: true,
            embeddings: false,
            tools: true,
            vision: false,
            structured_output: true,
            prompt_caching: false,
        };
        let model_ref = ModelRef {
            provider_id: "mock".into(),
            model_id: "mock-small".into(),
        };
        let value = json!({ "capabilities": capabilities, "model": model_ref });
        assert_eq!(value["capabilities"]["streaming"], true);
        assert_eq!(value["model"]["provider_id"], "mock");
    }
}

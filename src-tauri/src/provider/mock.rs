use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;

use super::{
    AIProvider, EmbedRequest, EmbedResponse, GenerateRequest, GenerateResponse, ModelProfile,
    ProviderCapabilities, ProviderDescriptor, ProviderError, ProviderResult, ProviderStream,
    ProviderUsage,
};

pub struct MockProvider {
    descriptor: ProviderDescriptor,
    profiles: Vec<ModelProfile>,
    generate_calls: AtomicUsize,
}

impl MockProvider {
    pub fn new(descriptor: ProviderDescriptor, profiles: Vec<ModelProfile>) -> Self {
        Self {
            descriptor,
            profiles,
            generate_calls: AtomicUsize::new(0),
        }
    }

    pub fn generate_calls(&self) -> usize {
        self.generate_calls.load(Ordering::Relaxed)
    }
}

#[async_trait]
impl AIProvider for MockProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        self.descriptor.clone()
    }

    fn capabilities(&self) -> ProviderCapabilities {
        self.profiles.iter().fold(ProviderCapabilities::default(), |mut result, profile| {
            result.streaming |= profile.capabilities.streaming;
            result.embeddings |= profile.capabilities.embeddings;
            result.tools |= profile.capabilities.tools;
            result.vision |= profile.capabilities.vision;
            result.structured_output |= profile.capabilities.structured_output;
            result.prompt_caching |= profile.capabilities.prompt_caching;
            result
        })
    }

    fn list_models(&self) -> ProviderResult<Vec<ModelProfile>> {
        Ok(self.profiles.clone())
    }

    async fn generate(&self, request: GenerateRequest) -> ProviderResult<GenerateResponse> {
        if !self
            .profiles
            .iter()
            .any(|profile| profile.model_id == request.model.model_id)
        {
            return Err(ProviderError::ModelNotFound);
        }
        self.generate_calls.fetch_add(1, Ordering::Relaxed);
        let text = request
            .messages
            .last()
            .map(|message| format!("mock response: {}", message.content))
            .unwrap_or_else(|| "mock response".into());
        Ok(GenerateResponse {
            model: request.model,
            text,
            usage: ProviderUsage {
                input_tokens: None,
                output_tokens: None,
            },
        })
    }

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
    use futures::executor::block_on;

    use super::*;
    use crate::provider::{
        EmbedRequest, GenerateRequest, ModelRef, PromptMessage, PromptRole, ProviderCapabilities,
        ProviderDescriptor,
    };

    fn provider() -> MockProvider {
        MockProvider::new(
            ProviderDescriptor {
                id: "mock".into(),
                display_name: "Mock".into(),
            },
            vec![ModelProfile {
                provider_id: "mock".into(),
                model_id: "mock-small".into(),
                display_name: "Mock Small".into(),
                context_window_tokens: 4096,
                default_output_tokens: 512,
                strengths: Vec::new(),
                weaknesses: Vec::new(),
                strategy: Vec::new(),
                capabilities: ProviderCapabilities::default(),
            }],
        )
    }

    fn request() -> GenerateRequest {
        GenerateRequest {
            model: ModelRef {
                provider_id: "mock".into(),
                model_id: "mock-small".into(),
            },
            messages: vec![PromptMessage {
                role: PromptRole::User,
                content: "hello".into(),
            }],
            max_output_tokens: 32,
            temperature: None,
        }
    }

    #[test]
    fn mock_generate_is_deterministic() {
        let provider = provider();
        let first = block_on(provider.generate(request())).unwrap();
        let second = block_on(provider.generate(request())).unwrap();
        assert_eq!(first, second);
        assert_eq!(provider.generate_calls(), 2);
        assert_eq!(first.text, "mock response: hello");
    }

    #[test]
    fn mock_unsupported_operations_do_not_execute() {
        let provider = provider();
        let stream = block_on(provider.stream(request()));
        assert!(matches!(
            stream,
            Err(ProviderError::UnsupportedCapability { capability }) if capability == "streaming"
        ));
        let embed = block_on(provider.embed(EmbedRequest {
            model: ModelRef {
                provider_id: "mock".into(),
                model_id: "mock-small".into(),
            },
            inputs: vec!["secret prompt".into()],
        }));
        assert!(matches!(
            embed,
            Err(ProviderError::UnsupportedCapability { capability }) if capability == "embeddings"
        ));
        assert_eq!(provider.generate_calls(), 0);
    }
}

use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use super::{AIProvider, ModelProfile, ProviderDescriptor, ProviderError, ProviderResult};

#[derive(Clone, Default)]
pub struct ProviderRegistry {
    providers: Arc<RwLock<BTreeMap<String, Arc<dyn AIProvider>>>>,
}

pub struct ResolvedModel {
    pub provider: Arc<dyn AIProvider>,
    pub profile: ModelProfile,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, provider: Arc<dyn AIProvider>) -> ProviderResult<()> {
        let descriptor = provider.descriptor();
        if descriptor.id.trim().is_empty() {
            return Err(ProviderError::InvalidRequest);
        }
        let models = provider.list_models()?;
        let mut seen = std::collections::BTreeSet::new();
        for model in &models {
            model.validate()?;
            if model.provider_id != descriptor.id || !seen.insert(model.model_id.clone()) {
                return Err(ProviderError::DuplicateModel);
            }
        }
        let mut providers = self
            .providers
            .write()
            .map_err(|_| ProviderError::ProviderFailure)?;
        if providers.contains_key(&descriptor.id) {
            return Err(ProviderError::ProviderAlreadyRegistered);
        }
        providers.insert(descriptor.id, provider);
        Ok(())
    }

    pub fn list_providers(&self) -> ProviderResult<Vec<ProviderDescriptor>> {
        let providers = self
            .providers
            .read()
            .map_err(|_| ProviderError::ProviderFailure)?;
        Ok(providers
            .values()
            .map(|provider| provider.descriptor())
            .collect())
    }

    pub fn list_models(&self, provider_id: Option<&str>) -> ProviderResult<Vec<ModelProfile>> {
        let providers = self
            .providers
            .read()
            .map_err(|_| ProviderError::ProviderFailure)?;
        let selected = if let Some(provider_id) = provider_id {
            vec![providers
                .get(provider_id)
                .ok_or(ProviderError::ProviderNotFound)?]
        } else {
            providers.values().collect()
        };
        let mut models = Vec::new();
        for provider in selected {
            models.extend(provider.list_models()?);
        }
        models.sort_by(|left, right| {
            left.provider_id
                .cmp(&right.provider_id)
                .then(left.model_id.cmp(&right.model_id))
        });
        Ok(models)
    }

    pub fn resolve(&self, provider_id: &str, model_id: &str) -> ProviderResult<ResolvedModel> {
        let providers = self
            .providers
            .read()
            .map_err(|_| ProviderError::ProviderFailure)?;
        let provider = providers
            .get(provider_id)
            .ok_or(ProviderError::ProviderNotFound)?
            .clone();
        let profile = provider
            .list_models()?
            .into_iter()
            .find(|model| model.model_id == model_id)
            .ok_or(ProviderError::ModelNotFound)?;
        Ok(ResolvedModel { provider, profile })
    }

    pub fn unregister(&self, provider_id: &str) -> ProviderResult<ProviderDescriptor> {
        let mut providers = self
            .providers
            .write()
            .map_err(|_| ProviderError::ProviderFailure)?;
        providers
            .remove(provider_id)
            .map(|provider| provider.descriptor())
            .ok_or(ProviderError::ProviderNotFound)
    }

    pub fn replace(&self, provider: Arc<dyn AIProvider>) -> ProviderResult<Arc<dyn AIProvider>> {
        let descriptor = provider.descriptor();
        if descriptor.id.trim().is_empty() {
            return Err(ProviderError::InvalidRequest);
        }
        let models = provider.list_models()?;
        let mut seen = std::collections::BTreeSet::new();
        for model in &models {
            model.validate()?;
            if model.provider_id != descriptor.id || !seen.insert(model.model_id.clone()) {
                return Err(ProviderError::DuplicateModel);
            }
        }
        let mut providers = self
            .providers
            .write()
            .map_err(|_| ProviderError::ProviderFailure)?;
        providers
            .insert(descriptor.id, provider)
            .ok_or(ProviderError::ProviderNotFound)
    }

    pub fn contains(&self, provider_id: &str) -> ProviderResult<bool> {
        Ok(self
            .providers
            .read()
            .map_err(|_| ProviderError::ProviderFailure)?
            .contains_key(provider_id))
    }

    pub fn descriptor(&self, provider_id: &str) -> ProviderResult<ProviderDescriptor> {
        self.providers
            .read()
            .map_err(|_| ProviderError::ProviderFailure)?
            .get(provider_id)
            .map(|provider| provider.descriptor())
            .ok_or(ProviderError::ProviderNotFound)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::provider::{MockProvider, ProviderCapabilities, ProviderDescriptor};

    fn profile(provider_id: &str, model_id: &str) -> ModelProfile {
        ModelProfile {
            provider_id: provider_id.into(),
            model_id: model_id.into(),
            display_name: model_id.into(),
            context_window_tokens: 4096,
            default_output_tokens: 512,
            strengths: Vec::new(),
            weaknesses: Vec::new(),
            strategy: Vec::new(),
            tier: crate::provider::ModelTier::Medium,
            capabilities: ProviderCapabilities::default(),
        }
    }

    fn provider(id: &str, models: &[&str]) -> Arc<MockProvider> {
        Arc::new(MockProvider::new(
            ProviderDescriptor {
                id: id.into(),
                display_name: id.into(),
            },
            models.iter().map(|model| profile(id, model)).collect(),
        ))
    }

    #[test]
    fn rejects_duplicate_provider_id() {
        let registry = ProviderRegistry::new();
        registry.register(provider("mock", &["small"])).unwrap();
        assert_eq!(
            registry.register(provider("mock", &["large"])),
            Err(ProviderError::ProviderAlreadyRegistered)
        );
    }

    #[test]
    fn lists_and_resolves_models() {
        let registry = ProviderRegistry::new();
        registry
            .register(provider("zeta", &["model-b", "model-a"]))
            .unwrap();
        registry.register(provider("alpha", &["model-c"])).unwrap();

        let providers = registry.list_providers().unwrap();
        assert_eq!(
            providers
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            ["alpha", "zeta"]
        );
        let models = registry.list_models(None).unwrap();
        assert_eq!(
            models
                .iter()
                .map(|item| item.model_id.as_str())
                .collect::<Vec<_>>(),
            ["model-c", "model-a", "model-b"]
        );
        assert_eq!(
            registry
                .resolve("zeta", "model-a")
                .unwrap()
                .profile
                .model_id,
            "model-a"
        );
    }

    #[test]
    fn returns_safe_not_found_errors() {
        let registry = ProviderRegistry::new();
        assert_eq!(
            registry.list_models(Some("missing")),
            Err(ProviderError::ProviderNotFound)
        );
        assert!(matches!(
            registry.resolve("missing", "model"),
            Err(ProviderError::ProviderNotFound)
        ));
        registry.register(provider("mock", &["small"])).unwrap();
        assert!(matches!(
            registry.resolve("mock", "missing"),
            Err(ProviderError::ModelNotFound)
        ));
    }
}

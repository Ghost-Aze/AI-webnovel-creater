use std::{
    collections::BTreeMap,
    fmt,
    sync::{Arc, Mutex, RwLock},
};

use serde::{Deserialize, Serialize};

use super::{
    ModelProfile, OpenAiCompatibleProvider, ProviderDescriptor, ProviderError, ProviderRegistry,
    ProviderResult,
};

mod secure_store;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CredentialId(String);

impl CredentialId {
    pub fn new(value: impl Into<String>) -> ProviderResult<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(ProviderError::InvalidRequest);
        }
        Ok(Self(value))
    }

    #[cfg(any(target_os = "windows", target_os = "android"))]
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for CredentialId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("CredentialId")
            .field(&"<redacted>")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SecretValue(String);

impl SecretValue {
    pub fn new(value: impl Into<String>) -> ProviderResult<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(ProviderError::InvalidRequest);
        }
        Ok(Self(value))
    }

    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<redacted secret>")
    }
}

pub trait CredentialStore: Send + Sync {
    fn kind(&self) -> CredentialStoreKind;
    fn status(&self) -> CredentialStoreStatus {
        CredentialStoreStatus::for_kind(self.kind())
    }
    fn put(&self, id: &CredentialId, value: SecretValue) -> ProviderResult<()>;
    fn get(&self, id: &CredentialId) -> ProviderResult<SecretValue>;
    fn delete(&self, id: &CredentialId) -> ProviderResult<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialStoreKind {
    Ephemeral,
    PlatformSecure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialStoreStatus {
    pub kind: CredentialStoreKind,
    pub persistent: bool,
    pub available: bool,
}

impl CredentialStoreStatus {
    pub fn for_kind(kind: CredentialStoreKind) -> Self {
        Self {
            available: matches!(kind, CredentialStoreKind::Ephemeral),
            persistent: matches!(kind, CredentialStoreKind::PlatformSecure),
            kind,
        }
    }

    pub(crate) fn platform_secure(available: bool) -> Self {
        Self {
            kind: CredentialStoreKind::PlatformSecure,
            persistent: true,
            available,
        }
    }
}

#[derive(Clone, Default)]
pub struct EphemeralCredentialStore {
    values: Arc<RwLock<BTreeMap<CredentialId, SecretValue>>>,
}

impl EphemeralCredentialStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CredentialStore for EphemeralCredentialStore {
    fn kind(&self) -> CredentialStoreKind {
        CredentialStoreKind::Ephemeral
    }

    fn put(&self, id: &CredentialId, value: SecretValue) -> ProviderResult<()> {
        let mut values = self
            .values
            .write()
            .map_err(|_| ProviderError::ProviderFailure)?;
        if values.contains_key(id) {
            return Err(ProviderError::CredentialAlreadyRegistered);
        }
        values.insert(id.clone(), value);
        Ok(())
    }

    fn get(&self, id: &CredentialId) -> ProviderResult<SecretValue> {
        self.values
            .read()
            .map_err(|_| ProviderError::ProviderFailure)?
            .get(id)
            .cloned()
            .ok_or(ProviderError::CredentialNotFound)
    }

    fn delete(&self, id: &CredentialId) -> ProviderResult<()> {
        self.values
            .write()
            .map_err(|_| ProviderError::ProviderFailure)?
            .remove(id);
        Ok(())
    }
}

pub use secure_store::PlatformSecureCredentialStore;

pub fn credential_store_for(kind: CredentialStoreKind) -> Arc<dyn CredentialStore> {
    match kind {
        CredentialStoreKind::Ephemeral => Arc::new(EphemeralCredentialStore::new()),
        CredentialStoreKind::PlatformSecure => Arc::new(PlatformSecureCredentialStore::new()),
    }
}

/// Select the store that is safe for the current application target.
///
/// Native targets get persistent OS-backed storage. Headless, Linux-cloud and
/// other unsupported targets deliberately use the process-only store so they
/// remain deterministic without ever writing a plaintext fallback.
pub fn application_credential_store() -> Arc<dyn CredentialStore> {
    #[cfg(any(target_os = "windows", target_os = "android"))]
    {
        Arc::new(PlatformSecureCredentialStore::new())
    }
    #[cfg(not(any(target_os = "windows", target_os = "android")))]
    {
        Arc::new(EphemeralCredentialStore::new())
    }
}

#[derive(Clone, Deserialize)]
pub struct ProviderConfigureInput {
    pub descriptor: ProviderDescriptor,
    pub base_url: String,
    pub models: Vec<ModelProfile>,
    pub credential_id: String,
    pub credential_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderSettings {
    pub descriptor: ProviderDescriptor,
    pub base_url: String,
    pub models: Vec<ModelProfile>,
    pub credential_id: String,
}

#[derive(Clone, Deserialize)]
pub struct ProviderUpdateInput {
    pub descriptor: ProviderDescriptor,
    pub base_url: String,
    pub models: Vec<ModelProfile>,
    pub credential_id: String,
    pub credential_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderTestResult {
    pub provider_id: String,
    pub model_id: String,
    pub message: String,
}

impl fmt::Debug for ProviderUpdateInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProviderUpdateInput")
            .field("descriptor", &self.descriptor)
            .field("base_url", &"<redacted>")
            .field("model_count", &self.models.len())
            .field("credential_id", &"<redacted>")
            .field("credential_value", &"<redacted>")
            .finish()
    }
}

impl fmt::Debug for ProviderConfigureInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProviderConfigureInput")
            .field("descriptor", &self.descriptor)
            .field("base_url", &"<redacted>")
            .field("model_count", &self.models.len())
            .field("credential_id", &"<redacted>")
            .field("credential_value", &"<redacted>")
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderConfigureResult {
    pub descriptor: ProviderDescriptor,
    pub models: Vec<ModelProfile>,
}

#[derive(Clone)]
pub struct ProviderRuntime {
    registry: ProviderRegistry,
    credentials: Arc<dyn CredentialStore>,
    credential_refs: Arc<RwLock<BTreeMap<String, CredentialId>>>,
    configurations: Arc<RwLock<BTreeMap<String, ProviderSettings>>>,
    lifecycle: Arc<Mutex<()>>,
}

impl ProviderRuntime {
    pub fn new(credentials: Arc<dyn CredentialStore>) -> Self {
        Self {
            registry: ProviderRegistry::new(),
            credentials,
            credential_refs: Arc::new(RwLock::new(BTreeMap::new())),
            configurations: Arc::new(RwLock::new(BTreeMap::new())),
            lifecycle: Arc::new(Mutex::new(())),
        }
    }

    pub fn registry(&self) -> ProviderRegistry {
        self.registry.clone()
    }

    pub fn credential_store_status(&self) -> CredentialStoreStatus {
        self.credentials.status()
    }

    pub fn settings(&self, provider_id: &str) -> ProviderResult<ProviderSettings> {
        self.configurations
            .read()
            .map_err(|_| ProviderError::ProviderFailure)?
            .get(provider_id)
            .cloned()
            .ok_or(ProviderError::ProviderNotFound)
    }

    pub fn settings_list(&self) -> ProviderResult<Vec<ProviderSettings>> {
        Ok(self
            .configurations
            .read()
            .map_err(|_| ProviderError::ProviderFailure)?
            .values()
            .cloned()
            .collect())
    }

    pub fn remember_settings(&self, settings: ProviderSettings) -> ProviderResult<()> {
        if settings.descriptor.id.trim().is_empty()
            || settings.descriptor.display_name.trim().is_empty()
            || settings.base_url.trim().is_empty()
            || settings.credential_id.trim().is_empty()
        {
            return Err(ProviderError::InvalidRequest);
        }
        for model in &settings.models {
            model.validate()?;
        }
        self.configurations
            .write()
            .map_err(|_| ProviderError::ProviderFailure)?
            .insert(settings.descriptor.id.clone(), settings);
        Ok(())
    }

    pub fn restore(&self, settings: ProviderSettings) -> ProviderResult<()> {
        self.remember_settings(settings.clone())?;
        let credential_id = CredentialId::new(settings.credential_id.clone())?;
        let secret = self.credentials.get(&credential_id)?;
        let provider = OpenAiCompatibleProvider::new(
            settings.descriptor.clone(),
            settings.models,
            settings.base_url,
            Some(secret.expose().to_string()),
        )
        .map(Arc::new)?;
        self.registry.register(provider)?;
        self.credential_refs
            .write()
            .map_err(|_| ProviderError::ProviderFailure)?
            .insert(settings.descriptor.id, credential_id);
        Ok(())
    }

    fn write_credential(
        &self,
        id: &CredentialId,
        value: SecretValue,
    ) -> ProviderResult<Option<SecretValue>> {
        match self.credentials.put(id, value.clone()) {
            Ok(()) => Ok(None),
            Err(ProviderError::CredentialAlreadyRegistered) => {
                let is_active = self
                    .credential_refs
                    .read()
                    .map_err(|_| ProviderError::ProviderFailure)?
                    .values()
                    .any(|active_id| active_id == id);
                if is_active {
                    return Err(ProviderError::CredentialAlreadyRegistered);
                }

                let previous = self.credentials.get(id)?;
                self.credentials.delete(id)?;
                if let Err(error) = self.credentials.put(id, value) {
                    let _ = self.credentials.put(id, previous.clone());
                    return Err(error);
                }
                Ok(Some(previous))
            }
            Err(error) => Err(error),
        }
    }

    fn rollback_credential(&self, id: &CredentialId, previous: Option<SecretValue>) {
        let _ = self.credentials.delete(id);
        if let Some(previous) = previous {
            let _ = self.credentials.put(id, previous);
        }
    }

    pub fn configure(
        &self,
        input: ProviderConfigureInput,
    ) -> ProviderResult<ProviderConfigureResult> {
        let _lifecycle = self
            .lifecycle
            .lock()
            .map_err(|_| ProviderError::ProviderFailure)?;
        let provider_id = input.descriptor.id.clone();
        if self
            .credential_refs
            .read()
            .map_err(|_| ProviderError::ProviderFailure)?
            .contains_key(&provider_id)
        {
            return Err(ProviderError::ProviderAlreadyRegistered);
        }
        if self.registry.contains(&provider_id)? {
            return Err(ProviderError::ProviderAlreadyRegistered);
        }
        let credential_id_text = input.credential_id.clone();
        let base_url = input.base_url.clone();
        let credential_id = CredentialId::new(credential_id_text.clone())?;
        let secret = SecretValue::new(input.credential_value)?;
        let provider = OpenAiCompatibleProvider::new(
            input.descriptor.clone(),
            input.models.clone(),
            base_url.clone(),
            Some(secret.expose().to_string()),
        )
        .map(Arc::new)?;
        let previous_credential = self.write_credential(&credential_id, secret)?;
        if let Err(error) = self.registry.register(provider) {
            self.rollback_credential(&credential_id, previous_credential);
            return Err(error);
        }
        if let Err(error) = self
            .credential_refs
            .write()
            .map_err(|_| ProviderError::ProviderFailure)
            .map(|mut refs| refs.insert(provider_id.clone(), credential_id.clone()))
        {
            let _ = self.registry.unregister(&provider_id);
            self.rollback_credential(&credential_id, previous_credential);
            return Err(error);
        }
        if let Err(error) = self
            .configurations
            .write()
            .map_err(|_| ProviderError::ProviderFailure)
            .map(|mut configurations| {
                configurations.insert(
                    provider_id,
                    ProviderSettings {
                        descriptor: input.descriptor.clone(),
                        base_url,
                        models: input.models.clone(),
                        credential_id: credential_id_text,
                    },
                )
            })
        {
            let _ = self
                .credential_refs
                .write()
                .map(|mut refs| refs.remove(&input.descriptor.id));
            let _ = self.registry.unregister(&input.descriptor.id);
            self.rollback_credential(&credential_id, previous_credential);
            return Err(error);
        }
        Ok(ProviderConfigureResult {
            descriptor: input.descriptor,
            models: input.models,
        })
    }

    pub fn update(&self, input: ProviderUpdateInput) -> ProviderResult<ProviderConfigureResult> {
        let _lifecycle = self
            .lifecycle
            .lock()
            .map_err(|_| ProviderError::ProviderFailure)?;
        let current = self.settings(&input.descriptor.id)?;
        if input.credential_id.trim() != current.credential_id.trim() {
            return Err(ProviderError::InvalidRequest);
        }
        let credential_id = CredentialId::new(current.credential_id.clone())?;
        let existing_secret = match self.credentials.get(&credential_id) {
            Ok(secret) => Some(secret),
            Err(ProviderError::CredentialNotFound) if input.credential_value.is_some() => None,
            Err(error) => return Err(error),
        };
        let replacement_secret = match input.credential_value.as_ref() {
            Some(value) => SecretValue::new(value.clone())?,
            None => existing_secret
                .clone()
                .ok_or(ProviderError::CredentialNotFound)?,
        };
        let provider = OpenAiCompatibleProvider::new(
            input.descriptor.clone(),
            input.models.clone(),
            input.base_url.clone(),
            Some(replacement_secret.expose().to_string()),
        )
        .map(Arc::new)?;

        let secret_changed = input.credential_value.is_some();
        let was_active = self.registry.contains(&input.descriptor.id)?;
        let previous_secret = if secret_changed && was_active {
            if let Some(existing_secret) = &existing_secret {
                self.credentials.delete(&credential_id)?;
                Some(existing_secret.clone())
            } else {
                None
            }
        } else if secret_changed {
            self.write_credential(&credential_id, replacement_secret.clone())?
        } else {
            None
        };
        if secret_changed && was_active {
            if let Err(error) = self.credentials.put(&credential_id, replacement_secret) {
                self.rollback_credential(&credential_id, previous_secret);
                return Err(error);
            }
        }
        let previous_provider = if was_active {
            match self.registry.replace(provider) {
                Ok(previous) => Some(previous),
                Err(error) => {
                    if secret_changed {
                        self.rollback_credential(&credential_id, previous_secret);
                    }
                    return Err(error);
                }
            }
        } else {
            if let Err(error) = self.registry.register(provider) {
                if secret_changed {
                    self.rollback_credential(&credential_id, previous_secret);
                }
                return Err(error);
            }
            None
        };
        if let Err(error) = self
            .configurations
            .write()
            .map_err(|_| ProviderError::ProviderFailure)
            .map(|mut configurations| {
                configurations.insert(
                    input.descriptor.id.clone(),
                    ProviderSettings {
                        descriptor: input.descriptor.clone(),
                        base_url: input.base_url.clone(),
                        models: input.models.clone(),
                        credential_id: current.credential_id.clone(),
                    },
                )
            })
        {
            if let Some(previous_provider) = previous_provider {
                let _ = self.registry.replace(previous_provider);
            } else {
                let _ = self.registry.unregister(&input.descriptor.id);
            }
            if secret_changed {
                self.rollback_credential(&credential_id, previous_secret);
            }
            return Err(error);
        }
        if !was_active {
            self.credential_refs
                .write()
                .map_err(|_| ProviderError::ProviderFailure)?
                .insert(input.descriptor.id.clone(), credential_id);
        }
        Ok(ProviderConfigureResult {
            descriptor: input.descriptor,
            models: input.models,
        })
    }

    pub fn remove(&self, provider_id: &str) -> ProviderResult<ProviderDescriptor> {
        let _lifecycle = self
            .lifecycle
            .lock()
            .map_err(|_| ProviderError::ProviderFailure)?;
        let descriptor = match self.registry.descriptor(provider_id) {
            Ok(descriptor) => descriptor,
            Err(ProviderError::ProviderNotFound) => {
                let settings = self
                    .configurations
                    .write()
                    .map_err(|_| ProviderError::ProviderFailure)?
                    .remove(provider_id)
                    .ok_or(ProviderError::ProviderNotFound)?;
                return Ok(settings.descriptor);
            }
            Err(error) => return Err(error),
        };
        let credential_id = self
            .credential_refs
            .write()
            .map_err(|_| ProviderError::ProviderFailure)?
            .remove(provider_id);
        let secret = match &credential_id {
            Some(id) => match self.credentials.get(id) {
                Ok(secret) => Some(secret),
                Err(error) => {
                    let id = id.clone();
                    let _ = self
                        .credential_refs
                        .write()
                        .map(|mut refs| refs.insert(provider_id.to_string(), id));
                    return Err(error);
                }
            },
            None => None,
        };
        if let Some(id) = &credential_id {
            if let Err(error) = self.credentials.delete(id) {
                if let Some(id) = credential_id {
                    let _ = self
                        .credential_refs
                        .write()
                        .map(|mut refs| refs.insert(provider_id.to_string(), id));
                }
                return Err(error);
            }
        }
        if let Err(error) = self.registry.unregister(provider_id) {
            if let (Some(id), Some(secret)) = (credential_id.clone(), secret) {
                let _ = self.credentials.put(&id, secret);
            }
            if let Some(id) = credential_id {
                let _ = self
                    .credential_refs
                    .write()
                    .map(|mut refs| refs.insert(provider_id.to_string(), id));
            }
            return Err(error);
        }
        let _ = self
            .configurations
            .write()
            .map(|mut configurations| configurations.remove(provider_id));
        Ok(descriptor)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc, Barrier,
        },
        thread,
    };

    use super::*;
    use crate::provider::{ModelTier, ProviderCapabilities, ProviderDescriptor};

    fn models(provider_id: &str) -> Vec<ModelProfile> {
        vec![ModelProfile {
            provider_id: provider_id.into(),
            model_id: "writer-small".into(),
            display_name: "Writer Small".into(),
            context_window_tokens: 4096,
            default_output_tokens: 512,
            strengths: Vec::new(),
            weaknesses: Vec::new(),
            strategy: Vec::new(),
            tier: ModelTier::Small,
            capabilities: ProviderCapabilities::default(),
        }]
    }

    fn input(provider_id: &str) -> ProviderConfigureInput {
        ProviderConfigureInput {
            descriptor: ProviderDescriptor {
                id: provider_id.into(),
                display_name: "Configured Provider".into(),
            },
            base_url: "https://example.test/v1".into(),
            models: models(provider_id),
            credential_id: "credential-1".into(),
            credential_value: "secret-token".into(),
        }
    }

    #[test]
    fn redacts_secret_values_and_config_debug() {
        let secret = SecretValue::new("secret-token").unwrap();
        assert!(!format!("{secret:?}").contains("secret-token"));
        let input = input("provider");
        let debug = format!("{input:?}");
        assert!(!debug.contains("secret-token"));
        assert!(!debug.contains("example.test"));
        assert!(!debug.contains("writer-small"));
    }

    #[test]
    fn validates_blank_credentials() {
        assert_eq!(CredentialId::new(" "), Err(ProviderError::InvalidRequest));
        assert_eq!(SecretValue::new(""), Err(ProviderError::InvalidRequest));
        assert_eq!(SecretValue::new("   "), Err(ProviderError::InvalidRequest));
    }

    #[cfg(not(any(target_os = "windows", target_os = "android")))]
    #[test]
    fn exposes_explicit_credential_store_modes_without_plaintext_fallback() {
        let ephemeral = credential_store_for(CredentialStoreKind::Ephemeral);
        assert_eq!(ephemeral.kind(), CredentialStoreKind::Ephemeral,);
        assert_eq!(
            CredentialStoreStatus::for_kind(ephemeral.kind()),
            CredentialStoreStatus {
                available: true,
                kind: CredentialStoreKind::Ephemeral,
                persistent: false,
            }
        );

        let platform = credential_store_for(CredentialStoreKind::PlatformSecure);
        let id = CredentialId::new("credential").unwrap();
        assert_eq!(platform.kind(), CredentialStoreKind::PlatformSecure);
        assert_eq!(
            platform.put(&id, SecretValue::new("secret").unwrap()),
            Err(ProviderError::SecureStoreUnavailable)
        );
        assert_eq!(
            platform.get(&id),
            Err(ProviderError::SecureStoreUnavailable)
        );
        assert_eq!(
            platform.status(),
            CredentialStoreStatus {
                available: false,
                kind: CredentialStoreKind::PlatformSecure,
                persistent: true,
            }
        );
    }

    #[derive(Clone)]
    struct ReportingCredentialStore;

    impl CredentialStore for ReportingCredentialStore {
        fn kind(&self) -> CredentialStoreKind {
            CredentialStoreKind::PlatformSecure
        }

        fn status(&self) -> CredentialStoreStatus {
            CredentialStoreStatus::platform_secure(true)
        }

        fn put(&self, _id: &CredentialId, _value: SecretValue) -> ProviderResult<()> {
            Ok(())
        }

        fn get(&self, _id: &CredentialId) -> ProviderResult<SecretValue> {
            Err(ProviderError::CredentialNotFound)
        }

        fn delete(&self, _id: &CredentialId) -> ProviderResult<()> {
            Ok(())
        }
    }

    #[test]
    fn runtime_delegates_store_availability() {
        let runtime = ProviderRuntime::new(Arc::new(ReportingCredentialStore));
        assert_eq!(
            runtime.credential_store_status(),
            CredentialStoreStatus {
                kind: CredentialStoreKind::PlatformSecure,
                persistent: true,
                available: true,
            }
        );
    }

    #[test]
    fn application_store_is_ephemeral_on_headless_targets() {
        let store = application_credential_store();
        #[cfg(not(any(target_os = "windows", target_os = "android")))]
        assert_eq!(
            store.status(),
            CredentialStoreStatus {
                kind: CredentialStoreKind::Ephemeral,
                persistent: false,
                available: true,
            }
        );
        #[cfg(any(target_os = "windows", target_os = "android"))]
        assert_eq!(store.kind(), CredentialStoreKind::PlatformSecure);
    }

    #[test]
    fn runtime_reports_the_selected_store_mode() {
        let runtime = ProviderRuntime::new(credential_store_for(CredentialStoreKind::Ephemeral));
        assert_eq!(
            runtime.credential_store_status(),
            CredentialStoreStatus {
                available: true,
                kind: CredentialStoreKind::Ephemeral,
                persistent: false,
            }
        );
    }

    #[test]
    fn configures_discovers_and_removes_provider() {
        let store = Arc::new(EphemeralCredentialStore::new());
        let runtime = ProviderRuntime::new(store.clone());
        let result = runtime.configure(input("provider")).unwrap();
        assert_eq!(result.descriptor.id, "provider");
        assert_eq!(runtime.registry().list_models(None).unwrap().len(), 1);
        let credential = CredentialId::new("credential-1").unwrap();
        assert_eq!(store.get(&credential).unwrap().expose(), "secret-token");

        let removed = runtime.remove("provider").unwrap();
        assert_eq!(removed.id, "provider");
        assert_eq!(runtime.registry().list_providers().unwrap(), Vec::new());
        assert_eq!(
            store.get(&credential),
            Err(ProviderError::CredentialNotFound)
        );
    }

    #[test]
    fn removes_remembered_provider_when_secure_credential_is_missing() {
        let runtime = ProviderRuntime::new(Arc::new(EphemeralCredentialStore::new()));
        let configured = input("inactive");
        runtime
            .remember_settings(ProviderSettings {
                descriptor: configured.descriptor.clone(),
                base_url: configured.base_url.clone(),
                models: configured.models.clone(),
                credential_id: configured.credential_id.clone(),
            })
            .unwrap();

        assert_eq!(runtime.remove("inactive").unwrap().id, "inactive");
        assert_eq!(
            runtime.settings("inactive"),
            Err(ProviderError::ProviderNotFound)
        );
    }

    #[test]
    fn updates_provider_metadata_and_keeps_existing_secret_when_omitted() {
        let store = Arc::new(EphemeralCredentialStore::new());
        let runtime = ProviderRuntime::new(store);
        runtime.configure(input("provider")).unwrap();

        let mut replacement_model = models("provider").remove(0);
        replacement_model.model_id = "writer-large".into();
        replacement_model.display_name = "Writer Large".into();
        let updated = runtime
            .update(ProviderUpdateInput {
                descriptor: ProviderDescriptor {
                    id: "provider".into(),
                    display_name: "Updated Provider".into(),
                },
                base_url: "https://updated.example.test/v1".into(),
                models: vec![replacement_model],
                credential_id: "credential-1".into(),
                credential_value: None,
            })
            .unwrap();

        assert_eq!(updated.descriptor.display_name, "Updated Provider");
        assert_eq!(
            runtime.registry().list_models(None).unwrap()[0].model_id,
            "writer-large"
        );
        assert_eq!(
            runtime.settings("provider").unwrap().base_url,
            "https://updated.example.test/v1"
        );
    }

    #[test]
    fn duplicate_and_invalid_configurations_roll_back_credentials() {
        let store = Arc::new(EphemeralCredentialStore::new());
        let runtime = ProviderRuntime::new(store.clone());
        runtime.configure(input("provider")).unwrap();
        let duplicate = runtime.configure(ProviderConfigureInput {
            credential_id: "duplicate-credential".into(),
            ..input("provider")
        });
        assert_eq!(duplicate, Err(ProviderError::ProviderAlreadyRegistered));
        assert_eq!(
            store.get(&CredentialId::new("duplicate-credential").unwrap()),
            Err(ProviderError::CredentialNotFound)
        );

        let collision = runtime.configure(ProviderConfigureInput {
            descriptor: ProviderDescriptor {
                id: "other-provider".into(),
                display_name: "Other".into(),
            },
            credential_id: "credential-1".into(),
            ..input("other-provider")
        });
        assert_eq!(collision, Err(ProviderError::CredentialAlreadyRegistered));
        assert_eq!(runtime.registry().list_providers().unwrap().len(), 1);
        assert_eq!(
            store
                .get(&CredentialId::new("credential-1").unwrap())
                .unwrap()
                .expose(),
            "secret-token"
        );

        let invalid_store = Arc::new(EphemeralCredentialStore::new());
        let invalid_runtime = ProviderRuntime::new(invalid_store.clone());
        let invalid = invalid_runtime.configure(ProviderConfigureInput {
            base_url: "not-a-url".into(),
            ..input("invalid")
        });
        assert_eq!(invalid, Err(ProviderError::InvalidRequest));
        assert_eq!(
            invalid_store.get(&CredentialId::new("credential-1").unwrap()),
            Err(ProviderError::CredentialNotFound)
        );
    }

    #[test]
    fn a_new_store_is_empty_after_restart_equivalent() {
        let store = EphemeralCredentialStore::new();
        let id = CredentialId::new("credential").unwrap();
        store.put(&id, SecretValue::new("secret").unwrap()).unwrap();
        let fresh_store = EphemeralCredentialStore::new();
        assert_eq!(fresh_store.get(&id), Err(ProviderError::CredentialNotFound));
    }

    #[test]
    fn stale_persistent_credential_can_be_reused_after_runtime_restart() {
        let store = Arc::new(EphemeralCredentialStore::new());
        let credential_id = CredentialId::new("credential-1").unwrap();
        store
            .put(&credential_id, SecretValue::new("previous-token").unwrap())
            .unwrap();

        let runtime = ProviderRuntime::new(store.clone());
        runtime.configure(input("provider")).unwrap();

        assert_eq!(store.get(&credential_id).unwrap().expose(), "secret-token");
    }

    #[test]
    fn cloned_runtime_shares_registry_and_store() {
        let runtime = ProviderRuntime::new(Arc::new(EphemeralCredentialStore::new()));
        let clone = runtime.clone();
        runtime.configure(input("provider")).unwrap();
        assert_eq!(clone.registry().list_providers().unwrap().len(), 1);
        let handle = thread::spawn(move || clone.remove("provider").unwrap());
        assert_eq!(handle.join().unwrap().id, "provider");
        assert!(runtime.registry().list_providers().unwrap().is_empty());
    }

    #[test]
    fn concurrent_configure_keeps_one_consistent_registration() {
        let runtime = Arc::new(ProviderRuntime::new(Arc::new(
            EphemeralCredentialStore::new(),
        )));
        let first = runtime.clone();
        let second = runtime.clone();
        let barrier = Arc::new(Barrier::new(2));
        let first_barrier = barrier.clone();
        let first_handle = thread::spawn(move || {
            first_barrier.wait();
            first.configure(input("provider"))
        });
        let second_barrier = barrier;
        let second_handle = thread::spawn(move || {
            second_barrier.wait();
            second.configure(ProviderConfigureInput {
                credential_id: "credential-2".into(),
                ..input("provider")
            })
        });
        let outcomes = [first_handle.join().unwrap(), second_handle.join().unwrap()];
        assert_eq!(outcomes.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            outcomes
                .iter()
                .filter(|result| {
                    matches!(result, Err(ProviderError::ProviderAlreadyRegistered))
                })
                .count(),
            1
        );
        assert_eq!(runtime.registry().list_providers().unwrap().len(), 1);
        assert!(runtime.remove("provider").is_ok());
        assert!(runtime.registry().list_providers().unwrap().is_empty());
    }

    #[derive(Clone)]
    struct FailingDeleteStore {
        inner: EphemeralCredentialStore,
        fail_delete: Arc<AtomicBool>,
    }

    impl CredentialStore for FailingDeleteStore {
        fn kind(&self) -> CredentialStoreKind {
            CredentialStoreKind::Ephemeral
        }

        fn put(&self, id: &CredentialId, value: SecretValue) -> ProviderResult<()> {
            self.inner.put(id, value)
        }

        fn get(&self, id: &CredentialId) -> ProviderResult<SecretValue> {
            self.inner.get(id)
        }

        fn delete(&self, id: &CredentialId) -> ProviderResult<()> {
            if self.fail_delete.load(Ordering::Relaxed) {
                Err(ProviderError::ProviderFailure)
            } else {
                self.inner.delete(id)
            }
        }
    }

    #[test]
    fn delete_failure_keeps_runtime_recoverable() {
        let store = Arc::new(FailingDeleteStore {
            inner: EphemeralCredentialStore::new(),
            fail_delete: Arc::new(AtomicBool::new(true)),
        });
        let runtime = ProviderRuntime::new(store.clone());
        runtime.configure(input("provider")).unwrap();
        assert_eq!(
            runtime.remove("provider"),
            Err(ProviderError::ProviderFailure)
        );
        assert_eq!(runtime.registry().list_providers().unwrap().len(), 1);
        assert!(store
            .get(&CredentialId::new("credential-1").unwrap())
            .is_ok());

        store.fail_delete.store(false, Ordering::Relaxed);
        assert!(runtime.remove("provider").is_ok());
        assert!(runtime.registry().list_providers().unwrap().is_empty());
        assert_eq!(
            store.get(&CredentialId::new("credential-1").unwrap()),
            Err(ProviderError::CredentialNotFound)
        );
    }
}

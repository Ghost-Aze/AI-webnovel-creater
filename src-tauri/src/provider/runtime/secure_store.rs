use super::{
    CredentialId, CredentialStore, CredentialStoreKind, CredentialStoreStatus, ProviderError,
    ProviderResult, SecretValue,
};

/// Stable namespace used for provider credentials in the platform store.
/// The opaque credential ID is used as the platform username.
#[cfg(any(target_os = "windows", target_os = "android"))]
const SERVICE_NAME: &str = "com.ghostaze.webnovel-ai-studio.provider";

#[derive(Clone, Default)]
pub struct PlatformSecureCredentialStore;

impl PlatformSecureCredentialStore {
    pub fn new() -> Self {
        Self
    }
}

impl CredentialStore for PlatformSecureCredentialStore {
    fn kind(&self) -> CredentialStoreKind {
        CredentialStoreKind::PlatformSecure
    }

    fn status(&self) -> CredentialStoreStatus {
        CredentialStoreStatus::platform_secure(native_store::available())
    }

    fn put(&self, id: &CredentialId, value: SecretValue) -> ProviderResult<()> {
        native_store::put(id, &value)
    }

    fn get(&self, id: &CredentialId) -> ProviderResult<SecretValue> {
        native_store::get(id)
    }

    fn delete(&self, id: &CredentialId) -> ProviderResult<()> {
        native_store::delete(id)
    }
}

#[cfg(any(target_os = "windows", target_os = "android"))]
mod native_store {
    use std::sync::OnceLock;

    use keyring_core::{Entry, Error as KeyringError};

    use super::{CredentialId, ProviderError, ProviderResult, SecretValue, SERVICE_NAME};

    static INITIALIZED: OnceLock<Result<(), ()>> = OnceLock::new();

    fn initialize() -> ProviderResult<()> {
        INITIALIZED
            .get_or_init(|| {
                #[cfg(target_os = "windows")]
                let store = windows_native_keyring_store::Store::new().map_err(|_| ())?;
                #[cfg(target_os = "android")]
                let store = android_native_keyring_store::Store::new().map_err(|_| ())?;
                keyring_core::set_default_store(store);
                Ok(())
            })
            .as_ref()
            .map(|_| ())
            .map_err(|_| ProviderError::SecureStoreUnavailable)
    }

    fn entry(id: &CredentialId) -> ProviderResult<Entry> {
        initialize()?;
        Entry::new(SERVICE_NAME, id.as_str()).map_err(normalize_error)
    }

    fn normalize_error(error: KeyringError) -> ProviderError {
        match error {
            KeyringError::NoEntry => ProviderError::CredentialNotFound,
            _ => ProviderError::SecureStoreUnavailable,
        }
    }

    pub fn available() -> bool {
        initialize().is_ok()
    }

    pub fn put(id: &CredentialId, value: &SecretValue) -> ProviderResult<()> {
        let entry = entry(id)?;
        match entry.get_password() {
            Ok(_) => Err(ProviderError::CredentialAlreadyRegistered),
            Err(KeyringError::NoEntry) => {
                entry.set_password(value.expose()).map_err(normalize_error)
            }
            Err(error) => Err(normalize_error(error)),
        }
    }

    pub fn get(id: &CredentialId) -> ProviderResult<SecretValue> {
        let entry = entry(id)?;
        let value = entry.get_password().map_err(normalize_error)?;
        SecretValue::new(value).map_err(|_| ProviderError::SecureStoreUnavailable)
    }

    pub fn delete(id: &CredentialId) -> ProviderResult<()> {
        let entry = entry(id)?;
        match entry.delete_credential() {
            Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
            Err(error) => Err(normalize_error(error)),
        }
    }
}

#[cfg(not(any(target_os = "windows", target_os = "android")))]
mod native_store {
    use super::{CredentialId, ProviderError, ProviderResult, SecretValue};

    pub fn available() -> bool {
        false
    }

    pub fn put(_id: &CredentialId, _value: &SecretValue) -> ProviderResult<()> {
        Err(ProviderError::SecureStoreUnavailable)
    }

    pub fn get(_id: &CredentialId) -> ProviderResult<SecretValue> {
        Err(ProviderError::SecureStoreUnavailable)
    }

    pub fn delete(_id: &CredentialId) -> ProviderResult<()> {
        Err(ProviderError::SecureStoreUnavailable)
    }
}

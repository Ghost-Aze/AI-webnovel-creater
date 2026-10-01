pub mod error;
pub mod http;
pub mod mock;
pub mod registry;
pub mod runtime;
pub mod types;

pub use error::{ProviderError, ProviderResult};
pub use http::OpenAiCompatibleProvider;
pub use mock::MockProvider;
pub use registry::{ProviderRegistry, ResolvedModel};
pub use runtime::{
    credential_store_for, CredentialId, CredentialStore, CredentialStoreKind,
    CredentialStoreStatus, EphemeralCredentialStore, PlatformSecureCredentialStore,
    ProviderConfigureInput, ProviderConfigureResult, ProviderRuntime, SecretValue,
};
pub use types::*;

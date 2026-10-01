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
    CredentialId, CredentialStore, EphemeralCredentialStore, ProviderConfigureInput,
    ProviderConfigureResult, ProviderRuntime, SecretValue,
};
pub use types::*;

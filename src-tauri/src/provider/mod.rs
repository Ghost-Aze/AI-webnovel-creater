pub mod error;
pub mod http;
pub mod mock;
pub mod registry;
pub mod routing;
pub mod runtime;
pub mod settings;
pub mod types;

pub use error::{ProviderError, ProviderResult};
pub use http::OpenAiCompatibleProvider;
pub use mock::MockProvider;
pub use registry::{ProviderRegistry, ResolvedModel};
pub use routing::{
    ModelRouter, ModelTask, QualityMode, RouteDecision, RouteSelectionReason, RoutingRequest,
};
pub use runtime::{
    application_credential_store, credential_store_for, CredentialId, CredentialStore,
    CredentialStoreKind, CredentialStoreStatus, EphemeralCredentialStore,
    PlatformSecureCredentialStore, ProviderConfigureInput, ProviderConfigureResult,
    ProviderRuntime, ProviderSettings, ProviderTestResult, ProviderUpdateInput, SecretValue,
};
pub use settings::ProviderSettingsRepository;
pub use types::*;

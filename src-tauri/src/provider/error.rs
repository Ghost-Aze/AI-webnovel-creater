use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProviderError {
    #[error("provider was not found")]
    ProviderNotFound,
    #[error("model was not found")]
    ModelNotFound,
    #[error("no suitable model was found for the requested route")]
    NoSuitableModel,
    #[error("provider capability is unsupported")]
    UnsupportedCapability { capability: String },
    #[error("provider request is invalid")]
    InvalidRequest,
    #[error("provider operation failed")]
    ProviderFailure,
    #[error("provider credentials were rejected")]
    Unauthorized,
    #[error("provider endpoint or model was not found")]
    EndpointNotFound,
    #[error("provider rate limit was reached")]
    RateLimited,
    #[error("provider is temporarily unavailable")]
    Unavailable,
    #[error("provider request timed out")]
    Timeout,
    #[error("provider could not be reached")]
    NetworkFailure,
    #[error("provider credential was not found")]
    CredentialNotFound,
    #[error("provider credential is already registered")]
    CredentialAlreadyRegistered,
    #[error("platform secure credential storage is unavailable")]
    SecureStoreUnavailable,
    #[error("provider is already registered")]
    ProviderAlreadyRegistered,
    #[error("model is duplicated within a provider")]
    DuplicateModel,
}

pub type ProviderResult<T> = Result<T, ProviderError>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{AppError, AppResult};

    fn safe_error(error: ProviderError) -> AppResult<()> {
        Err(AppError::from(error))
    }

    #[test]
    fn maps_provider_errors_to_safe_codes_without_request_text() {
        let error = safe_error(ProviderError::ProviderFailure);
        let serialized = serde_json::to_string(&error.unwrap_err()).unwrap();
        assert!(serialized.contains("provider_failure"));
        assert!(!serialized.contains("secret prompt"));

        let error = safe_error(ProviderError::UnsupportedCapability {
            capability: "streaming".into(),
        });
        assert_eq!(error.unwrap_err().code(), "unsupported_capability");
    }

    #[test]
    fn maps_classified_provider_failures_to_safe_application_codes() {
        let cases = [
            (ProviderError::Unauthorized, "provider_unauthorized"),
            (
                ProviderError::EndpointNotFound,
                "provider_endpoint_not_found",
            ),
            (ProviderError::RateLimited, "provider_rate_limited"),
            (ProviderError::Unavailable, "provider_unavailable"),
            (ProviderError::Timeout, "provider_timeout"),
            (ProviderError::NetworkFailure, "provider_network_failure"),
        ];

        for (provider_error, code) in cases {
            assert_eq!(safe_error(provider_error).unwrap_err().code(), code);
        }
    }
}

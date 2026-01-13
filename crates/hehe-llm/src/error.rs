use hehe_core::error::Error as CoreError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("API error: {provider} - {message}")]
    Api { provider: String, message: String },

    #[error("Rate limited: {provider}, retry after {retry_after_ms:?}ms")]
    RateLimited {
        provider: String,
        retry_after_ms: Option<u64>,
    },

    #[error("Context length exceeded: {max_tokens} tokens maximum")]
    ContextLengthExceeded { max_tokens: u32 },

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    #[error("Stream error: {0}")]
    Stream(String),

    #[error("Provider not available: {0}")]
    ProviderNotAvailable(String),

    #[error("Tool error: {0}")]
    Tool(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error(transparent)]
    Core(#[from] CoreError),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, LlmError>;

impl LlmError {
    pub fn api(provider: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Api {
            provider: provider.into(),
            message: message.into(),
        }
    }

    pub fn rate_limited(provider: impl Into<String>, retry_after_ms: Option<u64>) -> Self {
        Self::RateLimited {
            provider: provider.into(),
            retry_after_ms,
        }
    }

    pub fn invalid_request(msg: impl Into<String>) -> Self {
        Self::InvalidRequest(msg.into())
    }

    pub fn invalid_response(msg: impl Into<String>) -> Self {
        Self::InvalidResponse(msg.into())
    }

    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }

    pub fn stream(msg: impl Into<String>) -> Self {
        Self::Stream(msg.into())
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            LlmError::RateLimited { .. } | LlmError::Timeout(_) | LlmError::Network(_)
        )
    }
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("invalid input: {field} - {message}")]
    InvalidInput { field: String, message: String },

    #[error("not found: {resource_type} [{id}]")]
    NotFound { resource_type: String, id: String },

    #[error("operation cancelled")]
    Cancelled,

    #[error("operation timeout after {0}ms")]
    Timeout(u64),

    #[error("not permitted: {0}")]
    NotPermitted(String),

    #[error("llm error: [{provider}] {message}")]
    Llm { provider: String, message: String },

    #[error("tool error: [{tool}] {message}")]
    Tool { tool: String, message: String },

    #[error("memory error: [{backend}] {message}")]
    Memory { backend: String, message: String },

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Internal(String),

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl Error {
    pub fn not_found(resource_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self::NotFound {
            resource_type: resource_type.into(),
            id: id.into(),
        }
    }

    pub fn invalid_input(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidInput {
            field: field.into(),
            message: message.into(),
        }
    }

    pub fn llm(provider: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Llm {
            provider: provider.into(),
            message: message.into(),
        }
    }

    pub fn tool(tool: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Tool {
            tool: tool.into(),
            message: message.into(),
        }
    }

    pub fn memory(backend: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Memory {
            backend: backend.into(),
            message: message.into(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

use thiserror::Error;

pub mod codes {
    pub const CONFIG_INVALID: &str = "E1001";
    pub const CONFIG_MISSING: &str = "E1002";
    pub const VALIDATION_FAILED: &str = "E2001";
    pub const INVALID_INPUT: &str = "E2002";
    pub const NOT_FOUND: &str = "E3001";
    pub const ALREADY_EXISTS: &str = "E3002";
    pub const CANCELLED: &str = "E4001";
    pub const TIMEOUT: &str = "E4002";
    pub const NOT_PERMITTED: &str = "E4003";
    pub const RATE_LIMITED: &str = "E4004";
    pub const LLM_REQUEST_FAILED: &str = "E5001";
    pub const LLM_RATE_LIMITED: &str = "E5002";
    pub const LLM_CONTEXT_LENGTH: &str = "E5003";
    pub const LLM_INVALID_RESPONSE: &str = "E5004";
    pub const TOOL_NOT_FOUND: &str = "E6001";
    pub const TOOL_EXECUTION_FAILED: &str = "E6002";
    pub const TOOL_INVALID_INPUT: &str = "E6003";
    pub const STORAGE_CONNECTION: &str = "E7001";
    pub const STORAGE_QUERY: &str = "E7002";
    pub const STORAGE_WRITE: &str = "E7003";
    pub const INTERNAL: &str = "E9001";
    pub const NOT_IMPLEMENTED: &str = "E9002";
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Missing required config: {0}")]
    MissingConfig(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Invalid input: {field} - {message}")]
    InvalidInput { field: String, message: String },

    #[error("Not found: {resource_type} with id {id}")]
    NotFound { resource_type: String, id: String },

    #[error("Already exists: {resource_type} with id {id}")]
    AlreadyExists { resource_type: String, id: String },

    #[error("Operation cancelled")]
    Cancelled,

    #[error("Operation timeout after {0}ms")]
    Timeout(u64),

    #[error("Rate limited: {0}")]
    RateLimited(String),

    #[error("Operation not permitted: {0}")]
    NotPermitted(String),

    #[error("LLM error: {provider} - {message}")]
    Llm { provider: String, message: String },

    #[error("Tool error: {tool} - {message}")]
    Tool { tool: String, message: String },

    #[error("Storage error: {backend} - {message}")]
    Storage { backend: String, message: String },

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl Error {
    pub fn code(&self) -> &'static str {
        match self {
            Error::Config(_) => codes::CONFIG_INVALID,
            Error::MissingConfig(_) => codes::CONFIG_MISSING,
            Error::Json(_) => codes::VALIDATION_FAILED,
            Error::Io(_) => codes::INTERNAL,
            Error::Validation(_) => codes::VALIDATION_FAILED,
            Error::InvalidInput { .. } => codes::INVALID_INPUT,
            Error::NotFound { .. } => codes::NOT_FOUND,
            Error::AlreadyExists { .. } => codes::ALREADY_EXISTS,
            Error::Cancelled => codes::CANCELLED,
            Error::Timeout(_) => codes::TIMEOUT,
            Error::RateLimited(_) => codes::RATE_LIMITED,
            Error::NotPermitted(_) => codes::NOT_PERMITTED,
            Error::Llm { .. } => codes::LLM_REQUEST_FAILED,
            Error::Tool { .. } => codes::TOOL_EXECUTION_FAILED,
            Error::Storage { .. } => codes::STORAGE_CONNECTION,
            Error::NotImplemented(_) => codes::NOT_IMPLEMENTED,
            Error::Internal(_) => codes::INTERNAL,
            Error::Other(_) => codes::INTERNAL,
        }
    }

    pub fn not_found(resource_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self::NotFound {
            resource_type: resource_type.into(),
            id: id.into(),
        }
    }

    pub fn already_exists(resource_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self::AlreadyExists {
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

    pub fn storage(backend: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Storage {
            backend: backend.into(),
            message: message.into(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait ResultExt<T> {
    fn with_context<F: FnOnce() -> String>(self, f: F) -> Result<T>;
}

impl<T, E: Into<Error>> ResultExt<T> for std::result::Result<T, E> {
    fn with_context<F: FnOnce() -> String>(self, f: F) -> Result<T> {
        self.map_err(|e| {
            let inner = e.into();
            Error::Internal(format!("{}: {}", f(), inner))
        })
    }
}

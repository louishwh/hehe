use hehe_core::error::Error as CoreError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    NotFound(String),

    #[error("Tool already registered: {0}")]
    AlreadyRegistered(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Execution failed: {tool} - {message}")]
    ExecutionFailed { tool: String, message: String },

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    #[error("Cancelled")]
    Cancelled,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Core(#[from] CoreError),

    #[cfg(feature = "http")]
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, ToolError>;

impl ToolError {
    pub fn not_found(name: impl Into<String>) -> Self {
        Self::NotFound(name.into())
    }

    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    pub fn execution_failed(tool: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ExecutionFailed {
            tool: tool.into(),
            message: message.into(),
        }
    }

    pub fn permission_denied(msg: impl Into<String>) -> Self {
        Self::PermissionDenied(msg.into())
    }
}

use hehe_core::error::Error as CoreError;
use hehe_llm::LlmError;
use hehe_tools::ToolError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Max iterations reached: {0}")]
    MaxIterationsReached(usize),

    #[error("LLM error: {0}")]
    Llm(#[from] LlmError),

    #[error("Tool error: {0}")]
    Tool(#[from] ToolError),

    #[error("Core error: {0}")]
    Core(#[from] CoreError),

    #[error("Cancelled")]
    Cancelled,

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, AgentError>;

impl AgentError {
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

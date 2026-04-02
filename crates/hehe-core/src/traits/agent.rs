use crate::error::Result;
use crate::message::Message;
use crate::types::{AgentId, Id, Metadata, SessionId, Timestamp};
use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub created_at: Timestamp,
    pub messages: Vec<Message>,
    #[serde(default)]
    pub metadata: Metadata,
}

impl Session {
    pub fn new() -> Self {
        Self {
            id: SessionId::new(),
            created_at: Timestamp::now(),
            messages: Vec::new(),
            metadata: Metadata::new(),
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn last_messages(&self, n: usize) -> Vec<Message> {
        let len = self.messages.len();
        if n >= len {
            self.messages.clone()
        } else {
            self.messages[len - n..].to_vec()
        }
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentResponse {
    pub session_id: SessionId,
    pub text: String,
    pub tool_calls: Vec<ToolCallRecord>,
    pub iterations: usize,
}

impl AgentResponse {
    pub fn new(session_id: SessionId, text: impl Into<String>) -> Self {
        Self {
            session_id,
            text: text.into(),
            tool_calls: Vec::new(),
            iterations: 1,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn has_tool_calls(&self) -> bool {
        !self.tool_calls.is_empty()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
    pub output: String,
    pub is_error: bool,
    pub duration_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    MessageStart { session_id: Id },
    TextDelta { delta: String },
    TextComplete { text: String },
    ToolUseStart { id: String, name: String, input: serde_json::Value },
    ToolUseEnd { id: String, output: String, is_error: bool },
    Thinking { content: String },
    MessageEnd { session_id: Id },
    Error { message: String },
}

impl AgentEvent {
    pub fn text_delta(delta: impl Into<String>) -> Self {
        Self::TextDelta { delta: delta.into() }
    }

    pub fn text_complete(text: impl Into<String>) -> Self {
        Self::TextComplete { text: text.into() }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::Error { message: message.into() }
    }

    pub fn is_end(&self) -> bool {
        matches!(self, Self::MessageEnd { .. } | Self::Error { .. })
    }
}

pub type AgentEventStream = Pin<Box<dyn Stream<Item = AgentEvent> + Send>>;

#[async_trait]
pub trait AgentRuntime: Send + Sync {
    fn id(&self) -> &AgentId;
    fn name(&self) -> &str;
    async fn process(&self, session: &mut Session, input: &str) -> Result<AgentResponse>;
    fn process_stream(&self, session: &mut Session, input: &str) -> AgentEventStream;
}

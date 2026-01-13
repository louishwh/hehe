use crate::types::{AgentId, EventId, MessageId, SessionId, Timestamp, ToolCallId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    AgentStarted,
    AgentStopped,
    AgentPaused,
    AgentResumed,
    AgentError,
    SessionCreated,
    SessionEnded,
    MessageReceived,
    MessageSent,
    MessageStreaming,
    MessageStreamEnd,
    ToolCallStarted,
    ToolCallCompleted,
    ToolCallFailed,
    ToolCallCancelled,
    LlmRequestStarted,
    LlmRequestCompleted,
    LlmRequestFailed,
    LlmTokenUsage,
    StorageWrite,
    StorageDelete,
    ConfigReloaded,
    PluginLoaded,
    PluginUnloaded,
    Custom(String),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_write_tokens: Option<u32>,
}

impl TokenUsage {
    pub fn new(input: u32, output: u32) -> Self {
        Self {
            input_tokens: input,
            output_tokens: output,
            cache_read_tokens: None,
            cache_write_tokens: None,
        }
    }

    pub fn total(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventPayload {
    None,
    Agent {
        agent_id: AgentId,
    },
    Session {
        session_id: SessionId,
        agent_id: AgentId,
    },
    Message {
        message_id: MessageId,
        session_id: SessionId,
    },
    ToolCall {
        tool_call_id: ToolCallId,
        tool_name: String,
    },
    Llm {
        provider: String,
        model: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        usage: Option<TokenUsage>,
    },
    Error {
        code: String,
        message: String,
    },
    Custom(serde_json::Value),
}

impl Default for EventPayload {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub kind: EventKind,
    pub payload: EventPayload,
    pub timestamp: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

impl Event {
    pub fn new(kind: EventKind) -> Self {
        Self {
            id: EventId::new(),
            kind,
            payload: EventPayload::None,
            timestamp: Timestamp::now(),
            source: None,
            trace_id: None,
        }
    }

    pub fn with_payload(mut self, payload: EventPayload) -> Self {
        self.payload = payload;
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    pub fn agent_started(agent_id: AgentId) -> Self {
        Self::new(EventKind::AgentStarted).with_payload(EventPayload::Agent { agent_id })
    }

    pub fn agent_stopped(agent_id: AgentId) -> Self {
        Self::new(EventKind::AgentStopped).with_payload(EventPayload::Agent { agent_id })
    }

    pub fn tool_call_started(tool_call_id: ToolCallId, tool_name: impl Into<String>) -> Self {
        Self::new(EventKind::ToolCallStarted).with_payload(EventPayload::ToolCall {
            tool_call_id,
            tool_name: tool_name.into(),
        })
    }

    pub fn tool_call_completed(tool_call_id: ToolCallId, tool_name: impl Into<String>) -> Self {
        Self::new(EventKind::ToolCallCompleted).with_payload(EventPayload::ToolCall {
            tool_call_id,
            tool_name: tool_name.into(),
        })
    }

    pub fn llm_completed(
        provider: impl Into<String>,
        model: impl Into<String>,
        usage: Option<TokenUsage>,
    ) -> Self {
        Self::new(EventKind::LlmRequestCompleted).with_payload(EventPayload::Llm {
            provider: provider.into(),
            model: model.into(),
            usage,
        })
    }

    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(EventKind::AgentError).with_payload(EventPayload::Error {
            code: code.into(),
            message: message.into(),
        })
    }
}

#[async_trait::async_trait]
pub trait EventEmitter: Send + Sync {
    async fn emit(&self, event: Event);
}

#[async_trait::async_trait]
pub trait EventSubscriber: Send + Sync {
    fn event_kinds(&self) -> Vec<EventKind>;
    async fn on_event(&self, event: &Event);
}

use hehe_core::Id;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    MessageStart {
        session_id: Id,
    },

    TextDelta {
        delta: String,
    },

    TextComplete {
        text: String,
    },

    ToolUseStart {
        id: String,
        name: String,
        input: serde_json::Value,
    },

    ToolUseEnd {
        id: String,
        output: String,
        is_error: bool,
    },

    Thinking {
        content: String,
    },

    MessageEnd {
        session_id: Id,
    },

    Error {
        message: String,
    },
}

impl AgentEvent {
    pub fn message_start(session_id: Id) -> Self {
        Self::MessageStart { session_id }
    }

    pub fn text_delta(delta: impl Into<String>) -> Self {
        Self::TextDelta {
            delta: delta.into(),
        }
    }

    pub fn text_complete(text: impl Into<String>) -> Self {
        Self::TextComplete { text: text.into() }
    }

    pub fn tool_use_start(id: impl Into<String>, name: impl Into<String>, input: serde_json::Value) -> Self {
        Self::ToolUseStart {
            id: id.into(),
            name: name.into(),
            input,
        }
    }

    pub fn tool_use_end(id: impl Into<String>, output: impl Into<String>, is_error: bool) -> Self {
        Self::ToolUseEnd {
            id: id.into(),
            output: output.into(),
            is_error,
        }
    }

    pub fn thinking(content: impl Into<String>) -> Self {
        Self::Thinking {
            content: content.into(),
        }
    }

    pub fn message_end(session_id: Id) -> Self {
        Self::MessageEnd { session_id }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }

    pub fn is_end(&self) -> bool {
        matches!(self, Self::MessageEnd { .. } | Self::Error { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = AgentEvent::text_delta("Hello");
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("text_delta"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_tool_use_event() {
        let event = AgentEvent::tool_use_start("call_123", "read_file", serde_json::json!({"path": "/tmp"}));
        
        if let AgentEvent::ToolUseStart { id, name, input } = event {
            assert_eq!(id, "call_123");
            assert_eq!(name, "read_file");
            assert!(input.get("path").is_some());
        } else {
            panic!("Expected ToolUseStart event");
        }
    }

    #[test]
    fn test_is_end() {
        assert!(AgentEvent::message_end(Id::new()).is_end());
        assert!(AgentEvent::error("oops").is_end());
        assert!(!AgentEvent::text_delta("hi").is_end());
    }
}

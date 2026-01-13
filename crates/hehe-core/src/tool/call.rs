use crate::types::{Metadata, Timestamp, ToolCallId};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCallStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl ToolCallStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            ToolCallStatus::Completed | ToolCallStatus::Failed | ToolCallStatus::Cancelled
        )
    }

    pub fn is_success(&self) -> bool {
        matches!(self, ToolCallStatus::Completed)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: ToolCallId,
    pub name: String,
    pub input: Value,
    pub status: ToolCallStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub created_at: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<Timestamp>,
    #[serde(default, skip_serializing_if = "Metadata::is_empty")]
    pub metadata: Metadata,
}

impl ToolCall {
    pub fn new(name: impl Into<String>, input: Value) -> Self {
        Self {
            id: ToolCallId::new(),
            name: name.into(),
            input,
            status: ToolCallStatus::Pending,
            output: None,
            error: None,
            created_at: Timestamp::now(),
            started_at: None,
            completed_at: None,
            metadata: Metadata::new(),
        }
    }

    pub fn with_id(mut self, id: ToolCallId) -> Self {
        self.id = id;
        self
    }

    pub fn start(&mut self) {
        self.status = ToolCallStatus::Running;
        self.started_at = Some(Timestamp::now());
    }

    pub fn complete(&mut self, output: Value) {
        self.status = ToolCallStatus::Completed;
        self.output = Some(output);
        self.completed_at = Some(Timestamp::now());
    }

    pub fn complete_with_text(&mut self, text: impl Into<String>) {
        self.complete(Value::String(text.into()));
    }

    pub fn fail(&mut self, error: impl Into<String>) {
        self.status = ToolCallStatus::Failed;
        self.error = Some(error.into());
        self.completed_at = Some(Timestamp::now());
    }

    pub fn cancel(&mut self) {
        self.status = ToolCallStatus::Cancelled;
        self.completed_at = Some(Timestamp::now());
    }

    pub fn is_pending(&self) -> bool {
        self.status == ToolCallStatus::Pending
    }

    pub fn is_running(&self) -> bool {
        self.status == ToolCallStatus::Running
    }

    pub fn is_completed(&self) -> bool {
        self.status == ToolCallStatus::Completed
    }

    pub fn is_failed(&self) -> bool {
        self.status == ToolCallStatus::Failed
    }

    pub fn is_terminal(&self) -> bool {
        self.status.is_terminal()
    }

    pub fn duration_ms(&self) -> Option<i64> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end.unix_millis() - start.unix_millis()),
            _ => None,
        }
    }

    pub fn output_as_string(&self) -> Option<String> {
        self.output.as_ref().and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => serde_json::to_string(v).ok(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_call_lifecycle() {
        let mut call = ToolCall::new("test_tool", serde_json::json!({"arg": "value"}));

        assert!(call.is_pending());
        assert!(!call.is_terminal());

        call.start();
        assert!(call.is_running());
        assert!(call.started_at.is_some());

        call.complete(serde_json::json!({"result": "success"}));
        assert!(call.is_completed());
        assert!(call.is_terminal());
        assert!(call.completed_at.is_some());
        assert!(call.duration_ms().is_some());
    }

    #[test]
    fn test_tool_call_failure() {
        let mut call = ToolCall::new("failing_tool", serde_json::json!({}));

        call.start();
        call.fail("Something went wrong");

        assert!(call.is_failed());
        assert!(call.is_terminal());
        assert_eq!(call.error, Some("Something went wrong".to_string()));
    }
}

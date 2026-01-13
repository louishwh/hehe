use hehe_core::{Id, Metadata};
use hehe_tools::ToolOutput;
use serde::{Deserialize, Serialize};

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
pub struct AgentResponse {
    pub session_id: Id,
    pub text: String,
    pub tool_calls: Vec<ToolCallRecord>,
    pub iterations: usize,
    pub metadata: Metadata,
}

impl AgentResponse {
    pub fn new(session_id: Id, text: impl Into<String>) -> Self {
        Self {
            session_id,
            text: text.into(),
            tool_calls: Vec::new(),
            iterations: 1,
            metadata: Metadata::new(),
        }
    }

    pub fn with_tool_calls(mut self, tool_calls: Vec<ToolCallRecord>) -> Self {
        self.tool_calls = tool_calls;
        self
    }

    pub fn with_iterations(mut self, iterations: usize) -> Self {
        self.iterations = iterations;
        self
    }

    pub fn with_metadata<K: Into<String>, V: Serialize>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn has_tool_calls(&self) -> bool {
        !self.tool_calls.is_empty()
    }

    pub fn tool_call_count(&self) -> usize {
        self.tool_calls.len()
    }

    pub fn successful_tool_calls(&self) -> impl Iterator<Item = &ToolCallRecord> {
        self.tool_calls.iter().filter(|tc| !tc.is_error)
    }

    pub fn failed_tool_calls(&self) -> impl Iterator<Item = &ToolCallRecord> {
        self.tool_calls.iter().filter(|tc| tc.is_error)
    }
}

impl ToolCallRecord {
    pub fn success(
        id: impl Into<String>,
        name: impl Into<String>,
        input: serde_json::Value,
        output: &ToolOutput,
        duration_ms: u64,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            input,
            output: output.content.clone(),
            is_error: output.is_error,
            duration_ms,
        }
    }

    pub fn error(
        id: impl Into<String>,
        name: impl Into<String>,
        input: serde_json::Value,
        error_msg: impl Into<String>,
        duration_ms: u64,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            input,
            output: error_msg.into(),
            is_error: true,
            duration_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_basic() {
        let response = AgentResponse::new(Id::new(), "Hello, world!");
        assert_eq!(response.text(), "Hello, world!");
        assert!(!response.has_tool_calls());
    }

    #[test]
    fn test_response_with_tool_calls() {
        let tool_call = ToolCallRecord {
            id: "call_1".to_string(),
            name: "read_file".to_string(),
            input: serde_json::json!({"path": "/tmp/test.txt"}),
            output: "file content".to_string(),
            is_error: false,
            duration_ms: 100,
        };

        let response = AgentResponse::new(Id::new(), "Done!")
            .with_tool_calls(vec![tool_call])
            .with_iterations(2);

        assert!(response.has_tool_calls());
        assert_eq!(response.tool_call_count(), 1);
        assert_eq!(response.iterations, 2);
    }

    #[test]
    fn test_tool_call_record() {
        let record = ToolCallRecord::error(
            "call_1",
            "write_file",
            serde_json::json!({}),
            "Permission denied",
            50,
        );

        assert!(record.is_error);
        assert_eq!(record.output, "Permission denied");
    }
}

use crate::error::Result;
use async_trait::async_trait;
use hehe_core::{Context, Metadata, ToolDefinition};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ToolOutput {
    pub content: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<Artifact>,
    #[serde(default, skip_serializing_if = "Metadata::is_empty")]
    pub metadata: Metadata,
    #[serde(default)]
    pub is_error: bool,
}

impl ToolOutput {
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            artifacts: vec![],
            metadata: Metadata::new(),
            is_error: false,
        }
    }

    pub fn json<T: Serialize>(value: &T) -> Result<Self> {
        Ok(Self {
            content: serde_json::to_string_pretty(value)?,
            artifacts: vec![],
            metadata: Metadata::new(),
            is_error: false,
        })
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            content: message.into(),
            artifacts: vec![],
            metadata: Metadata::new(),
            is_error: true,
        }
    }

    pub fn with_artifact(mut self, artifact: Artifact) -> Self {
        self.artifacts.push(artifact);
        self
    }

    pub fn with_metadata<K: Into<String>, V: Serialize>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub content_type: String,
    pub data: ArtifactData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ArtifactData {
    Text { text: String },
    Base64 { data: String },
    File { path: String },
}

impl Artifact {
    pub fn text(name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            content_type: "text/plain".to_string(),
            data: ArtifactData::Text {
                text: content.into(),
            },
        }
    }

    pub fn file(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            content_type: "application/octet-stream".to_string(),
            data: ArtifactData::File { path: path.into() },
        }
    }

    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = content_type.into();
        self
    }
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn definition(&self) -> &ToolDefinition;

    async fn execute(&self, ctx: &Context, input: Value) -> Result<ToolOutput>;

    fn validate_input(&self, _input: &Value) -> Result<()> {
        Ok(())
    }

    fn name(&self) -> &str {
        &self.definition().name
    }

    fn is_dangerous(&self) -> bool {
        self.definition().dangerous
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_output_text() {
        let output = ToolOutput::text("Hello, world!");
        assert_eq!(output.content, "Hello, world!");
        assert!(!output.is_error);
    }

    #[test]
    fn test_tool_output_json() {
        let data = serde_json::json!({"key": "value"});
        let output = ToolOutput::json(&data).unwrap();
        assert!(output.content.contains("key"));
    }

    #[test]
    fn test_tool_output_error() {
        let output = ToolOutput::error("Something went wrong");
        assert!(output.is_error);
        assert_eq!(output.content, "Something went wrong");
    }

    #[test]
    fn test_artifact() {
        let artifact = Artifact::text("readme", "# Hello")
            .with_content_type("text/markdown");
        
        assert_eq!(artifact.name, "readme");
        assert_eq!(artifact.content_type, "text/markdown");
    }
}

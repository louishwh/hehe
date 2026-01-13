use hehe_core::{event::TokenUsage, stream::StopReason, Message, Metadata};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub id: String,
    pub model: String,
    pub message: Message,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<StopReason>,
    pub usage: TokenUsage,
    #[serde(default, skip_serializing_if = "Metadata::is_empty")]
    pub metadata: Metadata,
}

impl CompletionResponse {
    pub fn new(id: impl Into<String>, model: impl Into<String>, message: Message) -> Self {
        Self {
            id: id.into(),
            model: model.into(),
            message,
            stop_reason: None,
            usage: TokenUsage::default(),
            metadata: Metadata::new(),
        }
    }

    pub fn with_stop_reason(mut self, reason: StopReason) -> Self {
        self.stop_reason = Some(reason);
        self
    }

    pub fn with_usage(mut self, usage: TokenUsage) -> Self {
        self.usage = usage;
        self
    }

    pub fn text_content(&self) -> String {
        self.message.text_content()
    }

    pub fn has_tool_use(&self) -> bool {
        self.message.has_tool_use()
    }
}

#[derive(Clone, Debug)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub context_window: Option<u32>,
    pub max_output_tokens: Option<u32>,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_streaming: bool,
}

impl ModelInfo {
    pub fn new(id: impl Into<String>, provider: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            name: id.clone(),
            id,
            provider: provider.into(),
            context_window: None,
            max_output_tokens: None,
            supports_tools: false,
            supports_vision: false,
            supports_streaming: true,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn with_context_window(mut self, size: u32) -> Self {
        self.context_window = Some(size);
        self
    }

    pub fn with_max_output_tokens(mut self, max: u32) -> Self {
        self.max_output_tokens = Some(max);
        self
    }

    pub fn with_tools(mut self) -> Self {
        self.supports_tools = true;
        self
    }

    pub fn with_vision(mut self) -> Self {
        self.supports_vision = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_response() {
        let resp = CompletionResponse::new("resp-123", "gpt-4", Message::assistant("Hello"))
            .with_stop_reason(StopReason::EndTurn)
            .with_usage(TokenUsage::new(10, 5));

        assert_eq!(resp.id, "resp-123");
        assert_eq!(resp.text_content(), "Hello");
        assert_eq!(resp.usage.total(), 15);
    }

    #[test]
    fn test_model_info() {
        let model = ModelInfo::new("gpt-4o", "openai")
            .with_name("GPT-4o")
            .with_context_window(128000)
            .with_tools()
            .with_vision();

        assert_eq!(model.id, "gpt-4o");
        assert!(model.supports_tools);
        assert!(model.supports_vision);
    }
}

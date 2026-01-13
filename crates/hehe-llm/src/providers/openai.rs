use crate::error::{LlmError, Result};
use crate::traits::{BoxStream, LlmProvider};
use crate::types::{CompletionRequest, CompletionResponse, ModelInfo, ToolChoice};
use async_stream::try_stream;
use async_trait::async_trait;
use futures::StreamExt;
use hehe_core::capability::{Capabilities, Capability};
use hehe_core::event::TokenUsage;
use hehe_core::message::{ContentBlock, ToolResult, ToolUse};
use hehe_core::stream::{StopReason, StreamChunk};
use hehe_core::{Message, MessageId, Role};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    base_url: String,
    default_model: String,
    capabilities: Capabilities,
}

impl OpenAiProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_base_url(api_key, "https://api.openai.com/v1")
    }

    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key: api_key.into(),
            base_url: base_url.into(),
            default_model: "gpt-4o".to_string(),
            capabilities: Capabilities::full_agent(),
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = model.into();
        self
    }

    fn convert_messages(&self, messages: &[Message], system: Option<&str>) -> Vec<OpenAiMessage> {
        let mut result = Vec::new();

        if let Some(sys) = system {
            result.push(OpenAiMessage {
                role: "system".to_string(),
                content: Some(OpenAiContent::Text(sys.to_string())),
                tool_calls: None,
                tool_call_id: None,
            });
        }

        for msg in messages {
            if msg.role == Role::System {
                continue;
            }

            let openai_msg = match msg.role {
                Role::User => {
                    let content = if msg.content.len() == 1 {
                        if let Some(text) = msg.content[0].as_text() {
                            OpenAiContent::Text(text.to_string())
                        } else {
                            self.convert_content_parts(&msg.content)
                        }
                    } else {
                        self.convert_content_parts(&msg.content)
                    };

                    OpenAiMessage {
                        role: "user".to_string(),
                        content: Some(content),
                        tool_calls: None,
                        tool_call_id: None,
                    }
                }
                Role::Assistant => {
                    let tool_calls: Vec<_> = msg
                        .content
                        .iter()
                        .filter_map(|c| {
                            if let ContentBlock::ToolUse(tu) = c {
                                Some(OpenAiToolCall {
                                    id: tu.id.clone(),
                                    r#type: "function".to_string(),
                                    function: OpenAiFunctionCall {
                                        name: tu.name.clone(),
                                        arguments: tu.input.to_string(),
                                    },
                                })
                            } else {
                                None
                            }
                        })
                        .collect();

                    let text_content: String = msg
                        .content
                        .iter()
                        .filter_map(|c| c.as_text())
                        .collect::<Vec<_>>()
                        .join("");

                    OpenAiMessage {
                        role: "assistant".to_string(),
                        content: if text_content.is_empty() {
                            None
                        } else {
                            Some(OpenAiContent::Text(text_content))
                        },
                        tool_calls: if tool_calls.is_empty() {
                            None
                        } else {
                            Some(tool_calls)
                        },
                        tool_call_id: None,
                    }
                }
                Role::Tool => {
                    for block in &msg.content {
                        if let ContentBlock::ToolResult(tr) = block {
                            result.push(OpenAiMessage {
                                role: "tool".to_string(),
                                content: Some(OpenAiContent::Text(
                                    tr.content.clone().unwrap_or_default(),
                                )),
                                tool_calls: None,
                                tool_call_id: Some(tr.tool_use_id.clone()),
                            });
                        }
                    }
                    continue;
                }
                Role::System => continue,
            };

            result.push(openai_msg);
        }

        result
    }

    fn convert_content_parts(&self, content: &[ContentBlock]) -> OpenAiContent {
        let parts: Vec<OpenAiContentPart> = content
            .iter()
            .filter_map(|c| match c {
                ContentBlock::Text { text } => Some(OpenAiContentPart::Text { text: text.clone() }),
                ContentBlock::Image(img) => {
                    if let hehe_core::message::Source::Base64 { data } = &img.source {
                        let media_type = img.media_type.as_deref().unwrap_or("image/png");
                        Some(OpenAiContentPart::ImageUrl {
                            image_url: OpenAiImageUrl {
                                url: format!("data:{};base64,{}", media_type, data),
                            },
                        })
                    } else if let hehe_core::message::Source::Url { url } = &img.source {
                        Some(OpenAiContentPart::ImageUrl {
                            image_url: OpenAiImageUrl {
                                url: url.to_string(),
                            },
                        })
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect();

        OpenAiContent::Parts(parts)
    }

    fn convert_tools(
        &self,
        tools: &[hehe_core::ToolDefinition],
    ) -> Vec<OpenAiTool> {
        tools
            .iter()
            .map(|t| OpenAiTool {
                r#type: "function".to_string(),
                function: OpenAiFunction {
                    name: t.name.clone(),
                    description: Some(t.description.clone()),
                    parameters: serde_json::to_value(&t.parameters).unwrap_or(Value::Object(Default::default())),
                },
            })
            .collect()
    }

    fn convert_tool_choice(&self, choice: &ToolChoice) -> Value {
        match choice {
            ToolChoice::Auto => Value::String("auto".to_string()),
            ToolChoice::None => Value::String("none".to_string()),
            ToolChoice::Required => Value::String("required".to_string()),
            ToolChoice::Tool { name } => serde_json::json!({
                "type": "function",
                "function": { "name": name }
            }),
        }
    }

    fn parse_response(&self, response: OpenAiResponse) -> Result<CompletionResponse> {
        let choice = response
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| LlmError::invalid_response("No choices in response"))?;

        let mut content_blocks = Vec::new();

        if let Some(text) = choice.message.content {
            content_blocks.push(ContentBlock::text(text));
        }

        if let Some(tool_calls) = choice.message.tool_calls {
            for tc in tool_calls {
                let input: Value = serde_json::from_str(&tc.function.arguments)
                    .unwrap_or(Value::Object(Default::default()));
                content_blocks.push(ContentBlock::ToolUse(ToolUse::new(
                    tc.id,
                    tc.function.name,
                    input,
                )));
            }
        }

        let message = Message::new(Role::Assistant, content_blocks);

        let stop_reason = match choice.finish_reason.as_deref() {
            Some("stop") => Some(StopReason::EndTurn),
            Some("length") => Some(StopReason::MaxTokens),
            Some("tool_calls") => Some(StopReason::ToolUse),
            _ => None,
        };

        let usage = response
            .usage
            .map(|u| TokenUsage::new(u.prompt_tokens, u.completion_tokens))
            .unwrap_or_default();

        Ok(CompletionResponse::new(response.id, response.model, message)
            .with_usage(usage)
            .with_stop_reason(stop_reason.unwrap_or(StopReason::EndTurn)))
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    fn name(&self) -> &str {
        "openai"
    }

    fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let messages = self.convert_messages(&request.messages, request.system.as_deref());

        let mut body = serde_json::json!({
            "model": request.model,
            "messages": messages,
        });

        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = max_tokens.into();
        }
        if let Some(temp) = request.temperature {
            body["temperature"] = temp.into();
        }
        if let Some(top_p) = request.top_p {
            body["top_p"] = top_p.into();
        }
        if let Some(stop) = &request.stop {
            body["stop"] = stop.clone().into();
        }
        if let Some(tools) = &request.tools {
            body["tools"] = serde_json::to_value(self.convert_tools(tools))?;
        }
        if let Some(choice) = &request.tool_choice {
            body["tool_choice"] = self.convert_tool_choice(choice);
        }

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            
            if status.as_u16() == 429 {
                return Err(LlmError::rate_limited("openai", None));
            }
            
            return Err(LlmError::api("openai", format!("{}: {}", status, text)));
        }

        let openai_response: OpenAiResponse = response.json().await?;
        self.parse_response(openai_response)
    }

    async fn complete_stream(&self, request: CompletionRequest) -> Result<BoxStream<StreamChunk>> {
        let messages = self.convert_messages(&request.messages, request.system.as_deref());

        let mut body = serde_json::json!({
            "model": request.model,
            "messages": messages,
            "stream": true,
        });

        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = max_tokens.into();
        }
        if let Some(temp) = request.temperature {
            body["temperature"] = temp.into();
        }
        if let Some(tools) = &request.tools {
            body["tools"] = serde_json::to_value(self.convert_tools(tools))?;
        }

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::api("openai", format!("{}: {}", status, text)));
        }

        let stream = try_stream! {
            let mut bytes_stream = response.bytes_stream();
            let mut buffer = String::new();
            let message_id = MessageId::new();

            yield StreamChunk::MessageStart { message_id };

            while let Some(chunk) = bytes_stream.next().await {
                let chunk = chunk.map_err(|e| LlmError::stream(e.to_string()))?;
                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(pos) = buffer.find("\n\n") {
                    let line = buffer[..pos].to_string();
                    buffer = buffer[pos + 2..].to_string();

                    if line.starts_with("data: ") {
                        let data = &line[6..];
                        if data == "[DONE]" {
                            yield StreamChunk::MessageEnd { stop_reason: Some(StopReason::EndTurn) };
                            return;
                        }

                        if let Ok(event) = serde_json::from_str::<OpenAiStreamEvent>(data) {
                            if let Some(choice) = event.choices.first() {
                                if let Some(content) = &choice.delta.content {
                                    yield StreamChunk::TextDelta { text: content.clone() };
                                }

                                if let Some(tool_calls) = &choice.delta.tool_calls {
                                    for tc in tool_calls {
                                        if let Some(ref func) = tc.function {
                                            if let Some(ref name) = func.name {
                                                yield StreamChunk::ToolUseStart {
                                                    id: tc.id.clone().unwrap_or_default(),
                                                    name: name.clone(),
                                                };
                                            }
                                            if let Some(ref args) = func.arguments {
                                                yield StreamChunk::ToolUseDelta {
                                                    id: tc.id.clone().unwrap_or_default(),
                                                    input_delta: args.clone(),
                                                };
                                            }
                                        }
                                    }
                                }

                                if let Some(ref finish_reason) = choice.finish_reason {
                                    let stop = match finish_reason.as_str() {
                                        "stop" => StopReason::EndTurn,
                                        "length" => StopReason::MaxTokens,
                                        "tool_calls" => StopReason::ToolUse,
                                        _ => StopReason::EndTurn,
                                    };
                                    yield StreamChunk::MessageEnd { stop_reason: Some(stop) };
                                }
                            }
                        }
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        Ok(vec![
            ModelInfo::new("gpt-4o", "openai")
                .with_name("GPT-4o")
                .with_context_window(128000)
                .with_max_output_tokens(16384)
                .with_tools()
                .with_vision(),
            ModelInfo::new("gpt-4o-mini", "openai")
                .with_name("GPT-4o Mini")
                .with_context_window(128000)
                .with_max_output_tokens(16384)
                .with_tools()
                .with_vision(),
            ModelInfo::new("gpt-4-turbo", "openai")
                .with_name("GPT-4 Turbo")
                .with_context_window(128000)
                .with_tools()
                .with_vision(),
            ModelInfo::new("o1", "openai")
                .with_name("o1")
                .with_context_window(200000)
                .with_max_output_tokens(100000),
            ModelInfo::new("o1-mini", "openai")
                .with_name("o1 Mini")
                .with_context_window(128000)
                .with_max_output_tokens(65536),
        ])
    }

    fn default_model(&self) -> &str {
        &self.default_model
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<OpenAiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAiToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum OpenAiContent {
    Text(String),
    Parts(Vec<OpenAiContentPart>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum OpenAiContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: OpenAiImageUrl },
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiImageUrl {
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiToolCall {
    id: String,
    r#type: String,
    function: OpenAiFunctionCall,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiTool {
    r#type: String,
    function: OpenAiFunction,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiFunction {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    parameters: Value,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    id: String,
    model: String,
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponseMessage {
    content: Option<String>,
    tool_calls: Option<Vec<OpenAiToolCall>>,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamEvent {
    choices: Vec<OpenAiStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamChoice {
    delta: OpenAiStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamDelta {
    content: Option<String>,
    tool_calls: Option<Vec<OpenAiStreamToolCall>>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamToolCall {
    id: Option<String>,
    function: Option<OpenAiStreamFunction>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamFunction {
    name: Option<String>,
    arguments: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = OpenAiProvider::new("test-key").with_model("gpt-4");

        assert_eq!(provider.name(), "openai");
        assert_eq!(provider.default_model(), "gpt-4");
        assert!(provider.capabilities().has(&Capability::ToolUse));
    }

    #[test]
    fn test_message_conversion() {
        let provider = OpenAiProvider::new("test-key");

        let messages = vec![
            Message::user("Hello"),
            Message::assistant("Hi there!"),
        ];

        let converted = provider.convert_messages(&messages, Some("You are helpful"));

        assert_eq!(converted.len(), 3);
        assert_eq!(converted[0].role, "system");
        assert_eq!(converted[1].role, "user");
        assert_eq!(converted[2].role, "assistant");
    }

    #[tokio::test]
    async fn test_list_models() {
        let provider = OpenAiProvider::new("test-key");
        let models = provider.list_models().await.unwrap();

        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.id == "gpt-4o"));
    }
}

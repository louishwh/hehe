use crate::types::MessageId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamChunk {
    MessageStart {
        message_id: MessageId,
    },
    TextDelta {
        text: String,
    },
    ToolUseStart {
        id: String,
        name: String,
    },
    ToolUseDelta {
        id: String,
        input_delta: String,
    },
    ToolUseEnd {
        id: String,
    },
    ContentBlockStart {
        index: usize,
    },
    ContentBlockEnd {
        index: usize,
    },
    MessageEnd {
        stop_reason: Option<StopReason>,
    },
    Usage {
        input_tokens: u32,
        output_tokens: u32,
    },
    Error {
        code: String,
        message: String,
    },
    Ping,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
}

#[derive(Default)]
struct ToolUseBuilder {
    id: String,
    name: String,
    input_json: String,
}

#[derive(Default)]
pub struct StreamAggregator {
    message_id: Option<MessageId>,
    text_buffer: String,
    tool_uses: Vec<ToolUseBuilder>,
    stop_reason: Option<StopReason>,
    input_tokens: u32,
    output_tokens: u32,
    error: Option<(String, String)>,
}

impl StreamAggregator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, chunk: StreamChunk) {
        match chunk {
            StreamChunk::MessageStart { message_id } => {
                self.message_id = Some(message_id);
            }
            StreamChunk::TextDelta { text } => {
                self.text_buffer.push_str(&text);
            }
            StreamChunk::ToolUseStart { id, name } => {
                self.tool_uses.push(ToolUseBuilder {
                    id,
                    name,
                    input_json: String::new(),
                });
            }
            StreamChunk::ToolUseDelta { id, input_delta } => {
                if let Some(tu) = self.tool_uses.iter_mut().find(|t| t.id == id) {
                    tu.input_json.push_str(&input_delta);
                }
            }
            StreamChunk::MessageEnd { stop_reason } => {
                self.stop_reason = stop_reason;
            }
            StreamChunk::Usage {
                input_tokens,
                output_tokens,
            } => {
                self.input_tokens = input_tokens;
                self.output_tokens = output_tokens;
            }
            StreamChunk::Error { code, message } => {
                self.error = Some((code, message));
            }
            _ => {}
        }
    }

    pub fn message_id(&self) -> Option<MessageId> {
        self.message_id
    }

    pub fn text(&self) -> &str {
        &self.text_buffer
    }

    pub fn stop_reason(&self) -> Option<&StopReason> {
        self.stop_reason.as_ref()
    }

    pub fn is_complete(&self) -> bool {
        self.stop_reason.is_some()
    }

    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }

    pub fn error(&self) -> Option<(&str, &str)> {
        self.error.as_ref().map(|(c, m)| (c.as_str(), m.as_str()))
    }

    pub fn input_tokens(&self) -> u32 {
        self.input_tokens
    }

    pub fn output_tokens(&self) -> u32 {
        self.output_tokens
    }

    pub fn total_tokens(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }

    pub fn tool_use_count(&self) -> usize {
        self.tool_uses.len()
    }

    pub fn has_tool_use(&self) -> bool {
        !self.tool_uses.is_empty()
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

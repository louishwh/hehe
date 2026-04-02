use bytes::Bytes;
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Source {
    Base64 {
        data: String,
    },
    Url {
        url: Url,
    },
    File {
        path: Utf8PathBuf,
    },
    #[serde(skip)]
    Bytes(Bytes),
}

impl Source {
    pub fn base64(data: impl Into<String>) -> Self {
        Self::Base64 { data: data.into() }
    }

    pub fn url(url: Url) -> Self {
        Self::Url { url }
    }

    pub fn file(path: impl Into<Utf8PathBuf>) -> Self {
        Self::File { path: path.into() }
    }

    pub fn bytes(data: impl Into<Bytes>) -> Self {
        Self::Bytes(data.into())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageContent {
    pub source: Source,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,
}

impl ImageContent {
    pub fn new(source: Source) -> Self {
        Self {
            source,
            media_type: None,
            alt: None,
        }
    }

    pub fn with_media_type(mut self, media_type: impl Into<String>) -> Self {
        self.media_type = Some(media_type.into());
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioContent {
    pub source: Source,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript: Option<String>,
}

impl AudioContent {
    pub fn new(source: Source) -> Self {
        Self {
            source,
            media_type: None,
            transcript: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileContent {
    pub source: Source,
    pub filename: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
}

impl FileContent {
    pub fn new(source: Source, filename: impl Into<String>) -> Self {
        Self {
            source,
            filename: filename.into(),
            media_type: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolUse {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

impl ToolUse {
    pub fn new(id: impl Into<String>, name: impl Into<String>, input: serde_json::Value) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            input,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_use_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default)]
    pub is_error: bool,
}

impl ToolResult {
    pub fn success(tool_use_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content: Some(content.into()),
            is_error: false,
        }
    }

    pub fn error(tool_use_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content: Some(content.into()),
            is_error: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text { text: String },
    Image(ImageContent),
    Audio(AudioContent),
    File(FileContent),
    ToolUse(ToolUse),
    ToolResult(ToolResult),
}

impl ContentBlock {
    pub fn text(s: impl Into<String>) -> Self {
        Self::Text { text: s.into() }
    }

    pub fn tool_use(tu: ToolUse) -> Self {
        Self::ToolUse(tu)
    }

    pub fn tool_result(tr: ToolResult) -> Self {
        Self::ToolResult(tr)
    }

    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text { .. })
    }

    pub fn is_tool_use(&self) -> bool {
        matches!(self, Self::ToolUse(_))
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text { text } => Some(text),
            _ => None,
        }
    }

    pub fn as_tool_use(&self) -> Option<&ToolUse> {
        match self {
            Self::ToolUse(tu) => Some(tu),
            _ => None,
        }
    }

    pub fn as_tool_result(&self) -> Option<&ToolResult> {
        match self {
            Self::ToolResult(tr) => Some(tr),
            _ => None,
        }
    }
}

use bytes::Bytes;
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Source {
    Base64 { data: String },
    Url { url: Url },
    File { path: Utf8PathBuf },
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

    pub fn with_alt(mut self, alt: impl Into<String>) -> Self {
        self.alt = Some(alt.into());
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioContent {
    pub source: Source,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript: Option<String>,
}

impl AudioContent {
    pub fn new(source: Source) -> Self {
        Self {
            source,
            media_type: None,
            duration_ms: None,
            transcript: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VideoContent {
    pub source: Source,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

impl VideoContent {
    pub fn new(source: Source) -> Self {
        Self {
            source,
            media_type: None,
            duration_ms: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileContent {
    pub source: Source,
    pub filename: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

impl FileContent {
    pub fn new(source: Source, filename: impl Into<String>) -> Self {
        Self {
            source,
            filename: filename.into(),
            media_type: None,
            size: None,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default)]
    pub is_error: bool,
}

impl ToolResult {
    pub fn success(tool_use_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content: Some(content.into()),
            error: None,
            is_error: false,
        }
    }

    pub fn error(tool_use_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content: None,
            error: Some(error.into()),
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
    Video(VideoContent),
    File(FileContent),
    ToolUse(ToolUse),
    ToolResult(ToolResult),
    #[serde(rename = "x-custom")]
    Custom {
        #[serde(rename = "x-type")]
        custom_type: String,
        data: serde_json::Value,
    },
}

impl ContentBlock {
    pub fn text(s: impl Into<String>) -> Self {
        Self::Text { text: s.into() }
    }

    pub fn image(content: ImageContent) -> Self {
        Self::Image(content)
    }

    pub fn audio(content: AudioContent) -> Self {
        Self::Audio(content)
    }

    pub fn video(content: VideoContent) -> Self {
        Self::Video(content)
    }

    pub fn file(content: FileContent) -> Self {
        Self::File(content)
    }

    pub fn tool_use(tool_use: ToolUse) -> Self {
        Self::ToolUse(tool_use)
    }

    pub fn tool_result(result: ToolResult) -> Self {
        Self::ToolResult(result)
    }

    pub fn custom(custom_type: impl Into<String>, data: serde_json::Value) -> Self {
        Self::Custom {
            custom_type: custom_type.into(),
            data,
        }
    }

    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text { .. })
    }

    pub fn is_image(&self) -> bool {
        matches!(self, Self::Image(_))
    }

    pub fn is_tool_use(&self) -> bool {
        matches!(self, Self::ToolUse(_))
    }

    pub fn is_tool_result(&self) -> bool {
        matches!(self, Self::ToolResult(_))
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

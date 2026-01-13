pub mod capability;
pub mod config;
pub mod context;
pub mod error;
pub mod event;
pub mod message;
pub mod resource;
pub mod stream;
pub mod tool;
pub mod traits;
pub mod types;
pub mod utils;
pub mod version;

pub use config::Config;
pub use context::Context;
pub use error::{Error, Result};
pub use message::{ContentBlock, Message, MessageBuilder, Role};
pub use tool::{ToolCall, ToolCallStatus, ToolDefinition, ToolParameter};
pub use types::{AgentId, Id, MessageId, Metadata, SessionId, Timestamp, ToolCallId};
pub use version::VersionInfo;

pub mod prelude {
    pub use crate::capability::{Capabilities, Capability, CapabilityProvider};
    pub use crate::config::Config;
    pub use crate::context::Context;
    pub use crate::error::{Error, Result, ResultExt};
    pub use crate::event::{Event, EventEmitter, EventKind, EventPayload, EventSubscriber};
    pub use crate::message::{ContentBlock, Message, MessageBuilder, Role};
    pub use crate::resource::{Resource, ResourceRef, ResourceResolver, ResourceStore};
    pub use crate::stream::{StopReason, StreamAggregator, StreamChunk};
    pub use crate::tool::{ToolCall, ToolCallStatus, ToolDefinition, ToolParameter};
    pub use crate::traits::{Identifiable, Lifecycle, Named, Validatable};
    pub use crate::types::{AgentId, Id, MessageId, Metadata, SessionId, Timestamp, ToolCallId};
}

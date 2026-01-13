pub mod error;
pub mod providers;
pub mod traits;
pub mod types;

pub use error::{LlmError, Result};
pub use traits::{BoxStream, EmbeddingProvider, LlmProvider};
pub use types::{CompletionRequest, CompletionResponse, ModelInfo, ToolChoice};

#[cfg(feature = "openai")]
pub use providers::OpenAiProvider;

pub mod prelude {
    pub use crate::error::{LlmError, Result};
    pub use crate::traits::{BoxStream, EmbeddingProvider, LlmProvider};
    pub use crate::types::{CompletionRequest, CompletionResponse, ModelInfo, ToolChoice};

    #[cfg(feature = "openai")]
    pub use crate::providers::OpenAiProvider;
}

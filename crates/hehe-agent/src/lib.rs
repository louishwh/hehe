pub mod error;
pub mod config;
pub mod event;
pub mod session;
pub mod response;
pub mod executor;
pub mod agent;

pub use error::{AgentError, Result};
pub use config::AgentConfig;
pub use event::AgentEvent;
pub use session::{Session, SessionStats};
pub use response::{AgentResponse, ToolCallRecord};
pub use agent::{Agent, AgentBuilder};

pub mod prelude {
    pub use crate::error::{AgentError, Result};
    pub use crate::config::AgentConfig;
    pub use crate::event::AgentEvent;
    pub use crate::session::{Session, SessionStats};
    pub use crate::response::{AgentResponse, ToolCallRecord};
    pub use crate::agent::{Agent, AgentBuilder};
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use hehe_core::capability::Capabilities;
    use hehe_core::stream::StreamChunk;
    use hehe_core::Message;
    use hehe_llm::{BoxStream, CompletionRequest, CompletionResponse, LlmError, LlmProvider, ModelInfo};
    use std::sync::Arc;

    struct MockLlm;

    #[async_trait]
    impl LlmProvider for MockLlm {
        fn name(&self) -> &str {
            "mock"
        }

        fn capabilities(&self) -> &Capabilities {
            static CAPS: std::sync::OnceLock<Capabilities> = std::sync::OnceLock::new();
            CAPS.get_or_init(Capabilities::text_basic)
        }

        async fn complete(&self, _request: CompletionRequest) -> std::result::Result<CompletionResponse, LlmError> {
            Ok(CompletionResponse::new("id", "mock", Message::assistant("Test response")))
        }

        async fn complete_stream(
            &self,
            _request: CompletionRequest,
        ) -> std::result::Result<BoxStream<StreamChunk>, LlmError> {
            use futures::stream;
            Ok(Box::pin(stream::empty()))
        }

        async fn list_models(&self) -> std::result::Result<Vec<ModelInfo>, LlmError> {
            Ok(vec![])
        }

        fn default_model(&self) -> &str {
            "mock"
        }
    }

    #[tokio::test]
    async fn test_full_agent_flow() {
        let agent = Agent::builder()
            .system_prompt("You are a helpful assistant.")
            .model("mock")
            .llm(Arc::new(MockLlm))
            .build()
            .unwrap();

        let session = agent.create_session();

        let response = agent.chat(&session, "Hello!").await.unwrap();
        assert_eq!(response, "Test response");

        let response2 = agent.chat(&session, "How are you?").await.unwrap();
        assert_eq!(response2, "Test response");

        assert_eq!(session.message_count(), 4);

        let stats = session.stats();
        assert_eq!(stats.message_count, 4);
        assert_eq!(stats.iteration_count, 2);
    }

    #[tokio::test]
    async fn test_agent_with_config() {
        let config = AgentConfig::new("mock", "System prompt")
            .with_name("test-agent")
            .with_max_iterations(5)
            .with_temperature(0.3);

        let agent = Agent::builder()
            .config(config)
            .llm(Arc::new(MockLlm))
            .build()
            .unwrap();

        assert_eq!(agent.config().name, "test-agent");
        assert_eq!(agent.config().max_iterations, 5);
        assert_eq!(agent.config().temperature, 0.3);
    }
}

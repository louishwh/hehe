use crate::config::AgentConfig;
use crate::error::{AgentError, Result};
use crate::event::AgentEvent;
use crate::executor::Executor;
use crate::response::AgentResponse;
use crate::session::Session;
use hehe_llm::LlmProvider;
use hehe_tools::{ToolExecutor, ToolRegistry};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;

pub struct Agent {
    config: AgentConfig,
    llm: Arc<dyn LlmProvider>,
    tools: Option<Arc<ToolExecutor>>,
}

impl Agent {
    pub fn builder() -> AgentBuilder {
        AgentBuilder::new()
    }

    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    pub fn llm(&self) -> &Arc<dyn LlmProvider> {
        &self.llm
    }

    pub fn create_session(&self) -> Session {
        Session::new()
    }

    pub async fn chat(&self, session: &Session, message: &str) -> Result<String> {
        let response = self.process(session, message).await?;
        Ok(response.text)
    }

    pub async fn process(&self, session: &Session, message: &str) -> Result<AgentResponse> {
        let executor = Executor::new(self.config.clone(), self.llm.clone(), self.tools.clone());
        executor.execute(session, message).await
    }

    pub fn chat_stream(
        &self,
        session: &Session,
        message: &str,
    ) -> impl Stream<Item = AgentEvent> + Send {
        let (tx, rx) = mpsc::channel(100);
        let executor = Executor::new(self.config.clone(), self.llm.clone(), self.tools.clone());
        let session = session.clone();
        let message = message.to_string();

        tokio::spawn(async move {
            let _ = executor.execute_stream(&session, &message, tx).await;
        });

        ReceiverStream::new(rx)
    }
}

#[derive(Default)]
pub struct AgentBuilder {
    config: Option<AgentConfig>,
    name: Option<String>,
    system_prompt: Option<String>,
    model: Option<String>,
    temperature: Option<f32>,
    max_tokens: Option<usize>,
    max_iterations: Option<usize>,
    tools_enabled: Option<bool>,
    llm: Option<Arc<dyn LlmProvider>>,
    tool_registry: Option<Arc<ToolRegistry>>,
}

impl AgentBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn config(mut self, config: AgentConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = Some(max_iterations);
        self
    }

    pub fn tools_enabled(mut self, enabled: bool) -> Self {
        self.tools_enabled = Some(enabled);
        self
    }

    pub fn llm(mut self, llm: Arc<dyn LlmProvider>) -> Self {
        self.llm = Some(llm);
        self
    }

    pub fn tool_registry(mut self, registry: Arc<ToolRegistry>) -> Self {
        self.tool_registry = Some(registry);
        self
    }

    pub fn build(self) -> Result<Agent> {
        let llm = self.llm.ok_or_else(|| AgentError::config("LLM provider is required"))?;

        let mut config = self.config.unwrap_or_default();

        if let Some(name) = self.name {
            config.name = name;
        }
        if let Some(prompt) = self.system_prompt {
            config.system_prompt = prompt;
        }
        if let Some(model) = self.model {
            config.model = model;
        }
        if let Some(temp) = self.temperature {
            config.temperature = temp;
        }
        if let Some(max) = self.max_tokens {
            config.max_tokens = Some(max);
        }
        if let Some(max) = self.max_iterations {
            config.max_iterations = max;
        }
        if let Some(enabled) = self.tools_enabled {
            config.tools_enabled = enabled;
        }

        let tools = self.tool_registry.map(|registry| {
            Arc::new(ToolExecutor::new(registry))
        });

        Ok(Agent { config, llm, tools })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use hehe_core::capability::Capabilities;
    use hehe_core::stream::StreamChunk;
    use hehe_core::Message;
    use hehe_llm::{BoxStream, CompletionRequest, CompletionResponse, LlmError, ModelInfo};

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
            Ok(CompletionResponse::new("id", "mock", Message::assistant("Hello from mock!")))
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

    #[test]
    fn test_builder_missing_llm() {
        let result = Agent::builder()
            .system_prompt("You are helpful")
            .build();

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_agent_chat() {
        let agent = Agent::builder()
            .system_prompt("You are helpful.")
            .model("mock")
            .llm(Arc::new(MockLlm))
            .build()
            .unwrap();

        let session = agent.create_session();
        let response = agent.chat(&session, "Hi").await.unwrap();

        assert_eq!(response, "Hello from mock!");
        assert_eq!(session.message_count(), 2);
    }

    #[tokio::test]
    async fn test_agent_process() {
        let agent = Agent::builder()
            .name("test-agent")
            .system_prompt("You are helpful.")
            .model("mock")
            .temperature(0.5)
            .max_iterations(5)
            .llm(Arc::new(MockLlm))
            .build()
            .unwrap();

        assert_eq!(agent.config().name, "test-agent");
        assert_eq!(agent.config().temperature, 0.5);
        assert_eq!(agent.config().max_iterations, 5);

        let session = agent.create_session();
        let response = agent.process(&session, "Hi").await.unwrap();

        assert_eq!(response.text(), "Hello from mock!");
        assert_eq!(response.iterations, 1);
    }

    #[tokio::test]
    async fn test_session_persistence() {
        let agent = Agent::builder()
            .system_prompt("You are helpful.")
            .llm(Arc::new(MockLlm))
            .build()
            .unwrap();

        let session = agent.create_session();

        agent.chat(&session, "First message").await.unwrap();
        agent.chat(&session, "Second message").await.unwrap();

        assert_eq!(session.message_count(), 4);
    }
}

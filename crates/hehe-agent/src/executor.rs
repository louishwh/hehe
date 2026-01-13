use crate::config::AgentConfig;
use crate::error::{AgentError, Result};
use crate::event::AgentEvent;
use crate::response::{AgentResponse, ToolCallRecord};
use crate::session::Session;
use hehe_core::message::{ContentBlock, ToolResult, ToolUse};
use hehe_core::{Context, Message};
use hehe_llm::{CompletionRequest, LlmProvider};
use hehe_tools::ToolExecutor;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

pub struct Executor {
    config: AgentConfig,
    llm: Arc<dyn LlmProvider>,
    tools: Option<Arc<ToolExecutor>>,
}

impl Executor {
    pub fn new(
        config: AgentConfig,
        llm: Arc<dyn LlmProvider>,
        tools: Option<Arc<ToolExecutor>>,
    ) -> Self {
        Self { config, llm, tools }
    }

    pub async fn execute(&self, session: &Session, user_input: &str) -> Result<AgentResponse> {
        let user_message = Message::user(user_input);
        session.add_message(user_message);

        let mut all_tool_calls = Vec::new();
        let mut iterations = 0;

        loop {
            iterations += 1;
            session.increment_iterations();

            if iterations > self.config.max_iterations {
                return Err(AgentError::MaxIterationsReached(self.config.max_iterations));
            }

            info!(iteration = iterations, "Starting agent loop iteration");

            let request = self.build_request(session);
            let response = self.llm.complete(request).await?;

            let tool_uses = response.message.tool_uses();

            if tool_uses.is_empty() {
                let text = response.text_content();
                session.add_message(Message::assistant(&text));

                return Ok(AgentResponse::new(session.id().clone(), text)
                    .with_tool_calls(all_tool_calls)
                    .with_iterations(iterations));
            }

            let mut assistant_content = Vec::new();
            if !response.text_content().is_empty() {
                assistant_content.push(ContentBlock::text(response.text_content()));
            }
            for tu in &tool_uses {
                assistant_content.push(ContentBlock::tool_use(ToolUse::new(
                    &tu.id,
                    &tu.name,
                    tu.input.clone(),
                )));
            }
            session.add_message(Message::new(hehe_core::Role::Assistant, assistant_content));

            let tool_results = self.execute_tools(&tool_uses).await;

            for (tu, (output, duration_ms, is_error)) in tool_uses.iter().zip(&tool_results) {
                all_tool_calls.push(ToolCallRecord {
                    id: tu.id.clone(),
                    name: tu.name.clone(),
                    input: tu.input.clone(),
                    output: output.clone(),
                    is_error: *is_error,
                    duration_ms: *duration_ms,
                });
            }

            session.increment_tool_calls(tool_results.len());

            let tool_result_content: Vec<ContentBlock> = tool_uses
                .iter()
                .zip(&tool_results)
                .map(|(tu, (output, _, is_error))| {
                    if *is_error {
                        ContentBlock::tool_result(ToolResult::error(&tu.id, output))
                    } else {
                        ContentBlock::tool_result(ToolResult::success(&tu.id, output))
                    }
                })
                .collect();

            session.add_message(Message::tool(tool_result_content));
        }
    }

    pub async fn execute_stream(
        &self,
        session: &Session,
        user_input: &str,
        tx: mpsc::Sender<AgentEvent>,
    ) -> Result<AgentResponse> {
        let _ = tx.send(AgentEvent::message_start(session.id().clone())).await;

        let result = self.execute(session, user_input).await;

        match &result {
            Ok(response) => {
                let _ = tx.send(AgentEvent::text_complete(response.text.clone())).await;
                let _ = tx.send(AgentEvent::message_end(session.id().clone())).await;
            }
            Err(e) => {
                let _ = tx.send(AgentEvent::error(e.to_string())).await;
            }
        }

        result
    }

    fn build_request(&self, session: &Session) -> CompletionRequest {
        let messages = session.last_messages(self.config.max_context_messages);

        let mut request = CompletionRequest::new(&self.config.model, messages)
            .with_system(&self.config.system_prompt)
            .with_temperature(self.config.temperature);

        if let Some(max_tokens) = self.config.max_tokens {
            request = request.with_max_tokens(max_tokens as u32);
        }

        if self.config.tools_enabled {
            if let Some(tools) = &self.tools {
                let definitions = tools.registry().definitions();
                if !definitions.is_empty() {
                    request = request.with_tools(definitions);
                }
            }
        }

        request
    }

    async fn execute_tools(
        &self,
        tool_uses: &[&ToolUse],
    ) -> Vec<(String, u64, bool)> {
        let Some(tools) = &self.tools else {
            return tool_uses
                .iter()
                .map(|tu| (format!("Tool execution not available: {}", tu.name), 0, true))
                .collect();
        };

        let ctx = Context::new().with_timeout(self.config.tool_timeout());
        let mut results = Vec::with_capacity(tool_uses.len());

        for tu in tool_uses {
            let start = Instant::now();
            debug!(tool = %tu.name, id = %tu.id, "Executing tool");

            let result = tools.execute(&ctx, &tu.name, tu.input.clone()).await;
            let duration_ms = start.elapsed().as_millis() as u64;

            match result {
                Ok(output) => {
                    info!(tool = %tu.name, duration_ms, is_error = output.is_error, "Tool completed");
                    results.push((output.content, duration_ms, output.is_error));
                }
                Err(e) => {
                    warn!(tool = %tu.name, error = %e, "Tool execution failed");
                    results.push((e.to_string(), duration_ms, true));
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use hehe_core::capability::Capabilities;
    use hehe_core::stream::StreamChunk;
    use hehe_llm::{BoxStream, CompletionResponse, LlmError, ModelInfo};

    struct MockLlm {
        responses: std::sync::Mutex<Vec<CompletionResponse>>,
    }

    impl MockLlm {
        fn new(responses: Vec<CompletionResponse>) -> Self {
            Self {
                responses: std::sync::Mutex::new(responses),
            }
        }
    }

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
            let mut responses = self.responses.lock().unwrap();
            if responses.is_empty() {
                Ok(CompletionResponse::new("id", "model", Message::assistant("Default response")))
            } else {
                Ok(responses.remove(0))
            }
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
    async fn test_executor_simple_response() {
        let config = AgentConfig::new("mock", "You are helpful.");
        let llm = Arc::new(MockLlm::new(vec![CompletionResponse::new(
            "resp-1",
            "mock",
            Message::assistant("Hello!"),
        )]));

        let executor = Executor::new(config, llm, None);
        let session = Session::new();

        let response = executor.execute(&session, "Hi").await.unwrap();

        assert_eq!(response.text(), "Hello!");
        assert_eq!(response.iterations, 1);
        assert!(!response.has_tool_calls());
    }

    #[tokio::test]
    async fn test_executor_max_iterations() {
        let config = AgentConfig::new("mock", "You are helpful.").with_max_iterations(2);

        let tool_response = Message::new(
            hehe_core::Role::Assistant,
            vec![ContentBlock::tool_use(ToolUse::new(
                "call_1",
                "test_tool",
                serde_json::json!({}),
            ))],
        );

        let llm = Arc::new(MockLlm::new(vec![
            CompletionResponse::new("resp-1", "mock", tool_response.clone()),
            CompletionResponse::new("resp-2", "mock", tool_response.clone()),
            CompletionResponse::new("resp-3", "mock", tool_response),
        ]));

        let executor = Executor::new(config, llm, None);
        let session = Session::new();

        let result = executor.execute(&session, "Hi").await;

        assert!(matches!(result, Err(AgentError::MaxIterationsReached(2))));
    }
}

use crate::error::{Result, ToolError};
use crate::registry::ToolRegistry;
use crate::traits::ToolOutput;
use hehe_core::{Context, ToolCall, ToolCallStatus};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, warn};

pub struct ToolExecutor {
    registry: Arc<ToolRegistry>,
    default_timeout: Duration,
    require_confirmation_for_dangerous: bool,
}

impl ToolExecutor {
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self {
            registry,
            default_timeout: Duration::from_secs(60),
            require_confirmation_for_dangerous: true,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    pub fn allow_dangerous_without_confirmation(mut self) -> Self {
        self.require_confirmation_for_dangerous = false;
        self
    }

    pub async fn execute(
        &self,
        ctx: &Context,
        name: &str,
        input: Value,
    ) -> Result<ToolOutput> {
        let tool = self
            .registry
            .get(name)
            .ok_or_else(|| ToolError::not_found(name))?;

        if ctx.is_cancelled() {
            return Err(ToolError::Cancelled);
        }

        tool.validate_input(&input)?;

        info!(tool = name, "Executing tool");

        let execute_timeout = ctx
            .remaining()
            .unwrap_or(self.default_timeout)
            .min(self.default_timeout);

        let result = timeout(execute_timeout, tool.execute(ctx, input)).await;

        match result {
            Ok(Ok(output)) => {
                info!(tool = name, is_error = output.is_error, "Tool execution completed");
                Ok(output)
            }
            Ok(Err(e)) => {
                warn!(tool = name, error = %e, "Tool execution failed");
                Err(e)
            }
            Err(_) => {
                warn!(tool = name, timeout_ms = ?execute_timeout.as_millis(), "Tool execution timed out");
                Err(ToolError::Timeout(execute_timeout.as_millis() as u64))
            }
        }
    }

    pub async fn execute_call(&self, ctx: &Context, call: &mut ToolCall) -> Result<ToolOutput> {
        call.start();

        match self.execute(ctx, &call.name, call.input.clone()).await {
            Ok(output) => {
                if output.is_error {
                    call.fail(&output.content);
                } else {
                    call.complete(serde_json::to_value(&output.content).unwrap_or(Value::Null));
                }
                Ok(output)
            }
            Err(e) => {
                call.fail(e.to_string());
                Err(e)
            }
        }
    }

    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }

    pub fn is_dangerous(&self, name: &str) -> bool {
        self.registry
            .get(name)
            .map(|t| t.is_dangerous())
            .unwrap_or(false)
    }

    pub fn needs_confirmation(&self, name: &str) -> bool {
        self.require_confirmation_for_dangerous && self.is_dangerous(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Tool;
    use async_trait::async_trait;
    use hehe_core::ToolDefinition;

    struct EchoTool {
        def: ToolDefinition,
    }

    impl EchoTool {
        fn new() -> Self {
            Self {
                def: ToolDefinition::new("echo", "Echoes input"),
            }
        }
    }

    #[async_trait]
    impl Tool for EchoTool {
        fn definition(&self) -> &ToolDefinition {
            &self.def
        }

        async fn execute(&self, _ctx: &Context, input: Value) -> Result<ToolOutput> {
            Ok(ToolOutput::text(input.to_string()))
        }
    }

    struct SlowTool {
        def: ToolDefinition,
    }

    impl SlowTool {
        fn new() -> Self {
            Self {
                def: ToolDefinition::new("slow", "A slow tool"),
            }
        }
    }

    #[async_trait]
    impl Tool for SlowTool {
        fn definition(&self) -> &ToolDefinition {
            &self.def
        }

        async fn execute(&self, _ctx: &Context, _input: Value) -> Result<ToolOutput> {
            tokio::time::sleep(Duration::from_secs(10)).await;
            Ok(ToolOutput::text("done"))
        }
    }

    #[tokio::test]
    async fn test_executor_execute() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(EchoTool::new())).unwrap();

        let executor = ToolExecutor::new(Arc::new(registry));
        let ctx = Context::new();

        let output = executor
            .execute(&ctx, "echo", serde_json::json!({"message": "hello"}))
            .await
            .unwrap();

        assert!(output.content.contains("hello"));
    }

    #[tokio::test]
    async fn test_executor_not_found() {
        let registry = ToolRegistry::new();
        let executor = ToolExecutor::new(Arc::new(registry));
        let ctx = Context::new();

        let result = executor.execute(&ctx, "nonexistent", Value::Null).await;
        assert!(matches!(result, Err(ToolError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_executor_timeout() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(SlowTool::new())).unwrap();

        let executor = ToolExecutor::new(Arc::new(registry))
            .with_timeout(Duration::from_millis(100));
        let ctx = Context::new();

        let result = executor.execute(&ctx, "slow", Value::Null).await;
        assert!(matches!(result, Err(ToolError::Timeout(_))));
    }

    #[tokio::test]
    async fn test_executor_execute_call() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(EchoTool::new())).unwrap();

        let executor = ToolExecutor::new(Arc::new(registry));
        let ctx = Context::new();

        let mut call = ToolCall::new("echo", serde_json::json!({"x": 1}));
        assert!(call.is_pending());

        let output = executor.execute_call(&ctx, &mut call).await.unwrap();
        
        assert!(call.is_completed());
        assert!(!output.is_error);
    }
}

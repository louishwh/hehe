use crate::error::{Result, ToolError};
use crate::traits::{Tool, ToolOutput};
use async_trait::async_trait;
use hehe_core::{Context, ToolDefinition, ToolParameter};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::time::timeout;

pub struct ExecuteShellTool {
    def: ToolDefinition,
    default_timeout: Duration,
}

impl ExecuteShellTool {
    pub fn new() -> Self {
        let def = ToolDefinition::new("execute_shell", "Execute a shell command")
            .with_required_param(
                "command",
                ToolParameter::string().with_description("The shell command to execute"),
            )
            .with_param(
                "working_dir",
                ToolParameter::string().with_description("Working directory for the command"),
            )
            .with_param(
                "timeout_ms",
                ToolParameter::integer()
                    .with_description("Timeout in milliseconds (default: 60000)")
                    .with_default(Value::Number(60000.into())),
            )
            .with_param(
                "env",
                ToolParameter::object().with_description("Environment variables to set"),
            )
            .dangerous();
        Self {
            def,
            default_timeout: Duration::from_secs(60),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }
}

impl Default for ExecuteShellTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct ExecuteShellInput {
    command: String,
    working_dir: Option<String>,
    timeout_ms: Option<u64>,
    env: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
struct ShellOutput {
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    success: bool,
}

#[async_trait]
impl Tool for ExecuteShellTool {
    fn definition(&self) -> &ToolDefinition {
        &self.def
    }

    async fn execute(&self, ctx: &Context, input: Value) -> Result<ToolOutput> {
        let input: ExecuteShellInput = serde_json::from_value(input)?;

        if ctx.is_cancelled() {
            return Err(ToolError::Cancelled);
        }

        let shell = if cfg!(target_os = "windows") {
            ("cmd", "/C")
        } else {
            ("sh", "-c")
        };

        let mut cmd = Command::new(shell.0);
        cmd.arg(shell.1).arg(&input.command);

        if let Some(dir) = &input.working_dir {
            cmd.current_dir(dir);
        }

        if let Some(env) = &input.env {
            for (key, value) in env {
                cmd.env(key, value);
            }
        }

        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        let timeout_duration = input
            .timeout_ms
            .map(Duration::from_millis)
            .unwrap_or(self.default_timeout);

        let result = timeout(timeout_duration, async {
            let mut child = cmd.spawn()?;

            let mut stdout = String::new();
            let mut stderr = String::new();

            if let Some(mut stdout_handle) = child.stdout.take() {
                stdout_handle.read_to_string(&mut stdout).await?;
            }
            if let Some(mut stderr_handle) = child.stderr.take() {
                stderr_handle.read_to_string(&mut stderr).await?;
            }

            let status = child.wait().await?;

            Ok::<_, std::io::Error>((status, stdout, stderr))
        })
        .await;

        match result {
            Ok(Ok((status, stdout, stderr))) => {
                let exit_code = status.code();
                let output = ShellOutput {
                    exit_code,
                    stdout,
                    stderr,
                    success: status.success(),
                };

                if status.success() {
                    ToolOutput::json(&output)?
                        .with_metadata("command", &input.command)
                        .with_metadata("exit_code", exit_code.unwrap_or(-1));
                    Ok(ToolOutput::json(&output)?)
                } else {
                    Ok(ToolOutput::json(&output)?.with_metadata("is_error", true))
                }
            }
            Ok(Err(e)) => Ok(ToolOutput::error(format!("Failed to execute command: {}", e))),
            Err(_) => Ok(ToolOutput::error(format!(
                "Command timed out after {}ms",
                timeout_duration.as_millis()
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_shell_echo() {
        let tool = ExecuteShellTool::new();
        let ctx = Context::new();
        let input = serde_json::json!({
            "command": "echo hello"
        });

        let output = tool.execute(&ctx, input).await.unwrap();
        assert!(!output.is_error);

        let result: ShellOutput = serde_json::from_str(&output.content).unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("hello"));
    }

    #[tokio::test]
    async fn test_execute_shell_with_env() {
        let tool = ExecuteShellTool::new();
        let ctx = Context::new();
        let input = serde_json::json!({
            "command": if cfg!(target_os = "windows") { "echo %TEST_VAR%" } else { "echo $TEST_VAR" },
            "env": {
                "TEST_VAR": "test_value"
            }
        });

        let output = tool.execute(&ctx, input).await.unwrap();
        let result: ShellOutput = serde_json::from_str(&output.content).unwrap();
        assert!(result.stdout.contains("test_value"));
    }

    #[tokio::test]
    async fn test_execute_shell_failure() {
        let tool = ExecuteShellTool::new();
        let ctx = Context::new();
        let input = serde_json::json!({
            "command": "exit 1"
        });

        let output = tool.execute(&ctx, input).await.unwrap();
        let result: ShellOutput = serde_json::from_str(&output.content).unwrap();
        assert!(!result.success);
        assert_eq!(result.exit_code, Some(1));
    }

    #[tokio::test]
    async fn test_execute_shell_timeout() {
        let tool = ExecuteShellTool::new();
        let ctx = Context::new();
        let input = serde_json::json!({
            "command": "sleep 10",
            "timeout_ms": 100
        });

        let output = tool.execute(&ctx, input).await.unwrap();
        assert!(output.content.contains("timed out"));
    }
}

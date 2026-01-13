use crate::error::Result;
use crate::traits::{Tool, ToolOutput};
use async_trait::async_trait;
use hehe_core::{Context, ToolDefinition};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;

pub struct GetSystemInfoTool {
    def: ToolDefinition,
}

impl GetSystemInfoTool {
    pub fn new() -> Self {
        let def = ToolDefinition::new("get_system_info", "Get information about the current system");
        Self { def }
    }
}

impl Default for GetSystemInfoTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize)]
struct SystemInfo {
    os: OsInfo,
    process: ProcessInfo,
    env: EnvInfo,
}

#[derive(Serialize, Deserialize)]
struct OsInfo {
    name: String,
    arch: String,
    family: String,
}

#[derive(Serialize, Deserialize)]
struct ProcessInfo {
    current_dir: Option<String>,
    exe_path: Option<String>,
    pid: u32,
}

#[derive(Serialize, Deserialize)]
struct EnvInfo {
    home: Option<String>,
    user: Option<String>,
    path: Option<String>,
}

#[async_trait]
impl Tool for GetSystemInfoTool {
    fn definition(&self) -> &ToolDefinition {
        &self.def
    }

    async fn execute(&self, _ctx: &Context, _input: Value) -> Result<ToolOutput> {
        let info = SystemInfo {
            os: OsInfo {
                name: env::consts::OS.to_string(),
                arch: env::consts::ARCH.to_string(),
                family: env::consts::FAMILY.to_string(),
            },
            process: ProcessInfo {
                current_dir: env::current_dir()
                    .ok()
                    .map(|p| p.to_string_lossy().to_string()),
                exe_path: env::current_exe()
                    .ok()
                    .map(|p| p.to_string_lossy().to_string()),
                pid: std::process::id(),
            },
            env: EnvInfo {
                home: env::var("HOME").ok().or_else(|| env::var("USERPROFILE").ok()),
                user: env::var("USER").ok().or_else(|| env::var("USERNAME").ok()),
                path: env::var("PATH").ok(),
            },
        };

        ToolOutput::json(&info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_system_info() {
        let tool = GetSystemInfoTool::new();
        let ctx = Context::new();

        let output = tool.execute(&ctx, Value::Null).await.unwrap();
        assert!(!output.is_error);

        let info: SystemInfo = serde_json::from_str(&output.content).unwrap();
        assert!(!info.os.name.is_empty());
        assert!(!info.os.arch.is_empty());
        assert!(info.process.pid > 0);
    }

    #[test]
    fn test_definition() {
        let tool = GetSystemInfoTool::new();
        assert_eq!(tool.definition().name, "get_system_info");
        assert!(!tool.definition().dangerous);
    }
}

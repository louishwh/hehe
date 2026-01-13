use crate::error::Result;
use crate::traits::{Tool, ToolOutput};
use async_trait::async_trait;
use hehe_core::Context;
use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct SandboxConfig {
    pub allowed_paths: HashSet<PathBuf>,
    pub denied_paths: HashSet<PathBuf>,
    pub allowed_hosts: HashSet<String>,
    pub denied_hosts: HashSet<String>,
    pub allow_shell: bool,
    pub allow_network: bool,
    pub max_file_size: usize,
    pub max_output_size: usize,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            allowed_paths: HashSet::new(),
            denied_paths: HashSet::new(),
            allowed_hosts: HashSet::new(),
            denied_hosts: HashSet::new(),
            allow_shell: false,
            allow_network: true,
            max_file_size: 10 * 1024 * 1024,
            max_output_size: 1024 * 1024,
        }
    }
}

impl SandboxConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allow_all() -> Self {
        Self {
            allow_shell: true,
            allow_network: true,
            ..Default::default()
        }
    }

    pub fn allow_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.allowed_paths.insert(path.into());
        self
    }

    pub fn deny_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.denied_paths.insert(path.into());
        self
    }

    pub fn allow_host(mut self, host: impl Into<String>) -> Self {
        self.allowed_hosts.insert(host.into());
        self
    }

    pub fn deny_host(mut self, host: impl Into<String>) -> Self {
        self.denied_hosts.insert(host.into());
        self
    }

    pub fn with_shell(mut self, allow: bool) -> Self {
        self.allow_shell = allow;
        self
    }

    pub fn with_network(mut self, allow: bool) -> Self {
        self.allow_network = allow;
        self
    }

    pub fn is_path_allowed(&self, path: &PathBuf) -> bool {
        for denied in &self.denied_paths {
            if path.starts_with(denied) {
                return false;
            }
        }

        if self.allowed_paths.is_empty() {
            return true;
        }

        for allowed in &self.allowed_paths {
            if path.starts_with(allowed) {
                return true;
            }
        }

        false
    }

    pub fn is_host_allowed(&self, host: &str) -> bool {
        if !self.allow_network {
            return false;
        }

        if self.denied_hosts.contains(host) {
            return false;
        }

        if self.allowed_hosts.is_empty() {
            return true;
        }

        self.allowed_hosts.contains(host)
    }
}

#[async_trait]
pub trait Sandbox: Send + Sync {
    fn config(&self) -> &SandboxConfig;

    fn check_tool(&self, tool: &dyn Tool) -> Result<()>;

    async fn execute(
        &self,
        tool: Arc<dyn Tool>,
        ctx: &Context,
        input: Value,
    ) -> Result<ToolOutput>;
}

pub struct NativeSandbox {
    config: SandboxConfig,
}

impl NativeSandbox {
    pub fn new(config: SandboxConfig) -> Self {
        Self { config }
    }

    pub fn permissive() -> Self {
        Self::new(SandboxConfig::allow_all())
    }
}

impl Default for NativeSandbox {
    fn default() -> Self {
        Self::new(SandboxConfig::default())
    }
}

#[async_trait]
impl Sandbox for NativeSandbox {
    fn config(&self) -> &SandboxConfig {
        &self.config
    }

    fn check_tool(&self, tool: &dyn Tool) -> Result<()> {
        let name = tool.name();
        
        if tool.is_dangerous() && !self.config.allow_shell {
            if name == "execute_shell" {
                return Err(crate::error::ToolError::permission_denied(
                    "Shell execution is not allowed in this sandbox",
                ));
            }
        }

        Ok(())
    }

    async fn execute(
        &self,
        tool: Arc<dyn Tool>,
        ctx: &Context,
        input: Value,
    ) -> Result<ToolOutput> {
        self.check_tool(tool.as_ref())?;
        tool.execute(ctx, input).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert!(!config.allow_shell);
        assert!(config.allow_network);
    }

    #[test]
    fn test_sandbox_config_path_check() {
        let config = SandboxConfig::new()
            .allow_path("/home/user/workspace")
            .deny_path("/home/user/workspace/secrets");

        assert!(config.is_path_allowed(&PathBuf::from("/home/user/workspace/project")));
        assert!(!config.is_path_allowed(&PathBuf::from("/home/user/workspace/secrets/key")));
        assert!(!config.is_path_allowed(&PathBuf::from("/etc/passwd")));
    }

    #[test]
    fn test_sandbox_config_host_check() {
        let config = SandboxConfig::new()
            .allow_host("api.example.com")
            .deny_host("malicious.com");

        assert!(config.is_host_allowed("api.example.com"));
        assert!(!config.is_host_allowed("malicious.com"));
        assert!(!config.is_host_allowed("other.com"));
    }

    #[test]
    fn test_sandbox_config_no_restrictions() {
        let config = SandboxConfig::allow_all();
        
        assert!(config.is_path_allowed(&PathBuf::from("/any/path")));
        assert!(config.is_host_allowed("any.host.com"));
    }
}

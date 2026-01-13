mod filesystem;
#[cfg(feature = "shell")]
mod shell;
#[cfg(feature = "http")]
mod http;
mod system;

pub use filesystem::{ListDirectoryTool, ReadFileTool, SearchFilesTool, WriteFileTool};
#[cfg(feature = "shell")]
pub use shell::ExecuteShellTool;
#[cfg(feature = "http")]
pub use http::HttpRequestTool;
pub use system::GetSystemInfoTool;

use crate::registry::ToolRegistry;
use std::sync::Arc;

pub fn register_all(registry: &mut ToolRegistry) {
    registry.register(Arc::new(ReadFileTool::new())).ok();
    registry.register(Arc::new(WriteFileTool::new())).ok();
    registry.register(Arc::new(ListDirectoryTool::new())).ok();
    registry.register(Arc::new(SearchFilesTool::new())).ok();
    registry.register(Arc::new(GetSystemInfoTool::new())).ok();

    #[cfg(feature = "shell")]
    registry.register(Arc::new(ExecuteShellTool::new())).ok();

    #[cfg(feature = "http")]
    registry.register(Arc::new(HttpRequestTool::new())).ok();
}

pub fn create_default_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    register_all(&mut registry);
    registry
}

pub mod error;
pub mod traits;
pub mod registry;
pub mod executor;
#[cfg(feature = "builtin")]
pub mod builtin;
pub mod sandbox;

pub use error::{Result, ToolError};
pub use traits::{Artifact, ArtifactData, Tool, ToolOutput};
pub use registry::ToolRegistry;
pub use executor::ToolExecutor;
pub use sandbox::{NativeSandbox, Sandbox, SandboxConfig};

#[cfg(feature = "builtin")]
pub use builtin::{
    create_default_registry, register_all, 
    ListDirectoryTool, ReadFileTool, SearchFilesTool, WriteFileTool,
    GetSystemInfoTool,
};

#[cfg(all(feature = "builtin", feature = "shell"))]
pub use builtin::ExecuteShellTool;

#[cfg(all(feature = "builtin", feature = "http"))]
pub use builtin::HttpRequestTool;

pub mod prelude {
    pub use crate::error::{Result, ToolError};
    pub use crate::traits::{Artifact, ArtifactData, Tool, ToolOutput};
    pub use crate::registry::ToolRegistry;
    pub use crate::executor::ToolExecutor;
    pub use crate::sandbox::{NativeSandbox, Sandbox, SandboxConfig};

    #[cfg(feature = "builtin")]
    pub use crate::builtin::{
        create_default_registry, register_all,
        ListDirectoryTool, ReadFileTool, SearchFilesTool, WriteFileTool,
        GetSystemInfoTool,
    };

    #[cfg(all(feature = "builtin", feature = "shell"))]
    pub use crate::builtin::ExecuteShellTool;

    #[cfg(all(feature = "builtin", feature = "http"))]
    pub use crate::builtin::HttpRequestTool;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[cfg(feature = "builtin")]
    #[test]
    fn test_create_default_registry() {
        let registry = create_default_registry();
        assert!(registry.contains("read_file"));
        assert!(registry.contains("write_file"));
        assert!(registry.contains("list_directory"));
        assert!(registry.contains("search_files"));
        assert!(registry.contains("get_system_info"));
    }

    #[cfg(all(feature = "builtin", feature = "shell"))]
    #[test]
    fn test_registry_with_shell() {
        let registry = create_default_registry();
        assert!(registry.contains("execute_shell"));
        assert!(registry.dangerous_tools().contains(&"execute_shell"));
    }

    #[cfg(feature = "builtin")]
    #[tokio::test]
    async fn test_executor_with_builtin() {
        use hehe_core::Context;

        let registry = Arc::new(create_default_registry());
        let executor = ToolExecutor::new(registry);
        let ctx = Context::new();

        let output = executor
            .execute(&ctx, "get_system_info", serde_json::Value::Null)
            .await
            .unwrap();

        assert!(!output.is_error);
        assert!(output.content.contains("os"));
    }
}

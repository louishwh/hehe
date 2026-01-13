use crate::error::{Result, ToolError};
use crate::traits::Tool;
use hehe_core::ToolDefinition;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Arc<dyn Tool>) -> Result<()> {
        let name = tool.name().to_string();
        if self.tools.contains_key(&name) {
            return Err(ToolError::AlreadyRegistered(name));
        }
        self.tools.insert(name, tool);
        Ok(())
    }

    pub fn register_boxed(&mut self, tool: Box<dyn Tool>) -> Result<()> {
        self.register(Arc::from(tool))
    }

    pub fn unregister(&mut self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.remove(name)
    }

    pub fn get(&self, name: &str) -> Option<&Arc<dyn Tool>> {
        self.tools.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    pub fn list(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .map(|t| t.definition().clone())
            .collect()
    }

    pub fn len(&self) -> usize {
        self.tools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    pub fn dangerous_tools(&self) -> Vec<&str> {
        self.tools
            .iter()
            .filter(|(_, t)| t.is_dangerous())
            .map(|(name, _)| name.as_str())
            .collect()
    }

    pub fn safe_tools(&self) -> Vec<&str> {
        self.tools
            .iter()
            .filter(|(_, t)| !t.is_dangerous())
            .map(|(name, _)| name.as_str())
            .collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::ToolOutput;
    use async_trait::async_trait;
    use hehe_core::{Context, ToolDefinition, ToolParameter};
    use serde_json::Value;

    struct MockTool {
        def: ToolDefinition,
    }

    impl MockTool {
        fn new(name: &str, dangerous: bool) -> Self {
            let mut def = ToolDefinition::new(name, format!("{} tool", name));
            if dangerous {
                def = def.dangerous();
            }
            Self { def }
        }
    }

    #[async_trait]
    impl Tool for MockTool {
        fn definition(&self) -> &ToolDefinition {
            &self.def
        }

        async fn execute(&self, _ctx: &Context, _input: Value) -> Result<ToolOutput> {
            Ok(ToolOutput::text("mock output"))
        }
    }

    #[test]
    fn test_registry_register() {
        let mut registry = ToolRegistry::new();
        
        let tool = Arc::new(MockTool::new("test_tool", false));
        registry.register(tool).unwrap();

        assert!(registry.contains("test_tool"));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_registry_duplicate() {
        let mut registry = ToolRegistry::new();
        
        registry.register(Arc::new(MockTool::new("dup", false))).unwrap();
        let result = registry.register(Arc::new(MockTool::new("dup", false)));
        
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_list() {
        let mut registry = ToolRegistry::new();
        
        registry.register(Arc::new(MockTool::new("tool_a", false))).unwrap();
        registry.register(Arc::new(MockTool::new("tool_b", true))).unwrap();

        let list = registry.list();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"tool_a"));
        assert!(list.contains(&"tool_b"));
    }

    #[test]
    fn test_registry_dangerous_tools() {
        let mut registry = ToolRegistry::new();
        
        registry.register(Arc::new(MockTool::new("safe", false))).unwrap();
        registry.register(Arc::new(MockTool::new("danger", true))).unwrap();

        let dangerous = registry.dangerous_tools();
        assert_eq!(dangerous.len(), 1);
        assert!(dangerous.contains(&"danger"));

        let safe = registry.safe_tools();
        assert_eq!(safe.len(), 1);
        assert!(safe.contains(&"safe"));
    }

    #[test]
    fn test_registry_unregister() {
        let mut registry = ToolRegistry::new();
        
        registry.register(Arc::new(MockTool::new("removable", false))).unwrap();
        assert!(registry.contains("removable"));

        let removed = registry.unregister("removable");
        assert!(removed.is_some());
        assert!(!registry.contains("removable"));
    }

    #[test]
    fn test_registry_definitions() {
        let mut registry = ToolRegistry::new();
        
        registry.register(Arc::new(MockTool::new("tool1", false))).unwrap();
        registry.register(Arc::new(MockTool::new("tool2", false))).unwrap();

        let defs = registry.definitions();
        assert_eq!(defs.len(), 2);
    }
}

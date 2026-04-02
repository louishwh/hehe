use crate::tool::{ToolDefinition, ToolParameter};

#[test]
fn test_tool_definition() {
    let tool = ToolDefinition::new("read_file", "Read a file")
        .with_required_param(
            "path",
            ToolParameter::string().with_description("File path"),
        )
        .dangerous();

    assert_eq!(tool.name, "read_file");
    assert!(tool.dangerous);

    let props = tool.parameters.properties.as_ref().unwrap();
    assert!(props.contains_key("path"));

    let required = tool.parameters.required.as_ref().unwrap();
    assert!(required.contains(&"path".to_string()));
}

#[test]
fn test_tool_definition_serde() {
    let tool = ToolDefinition::new("test", "A test tool")
        .with_required_param("input", ToolParameter::string());

    let json = serde_json::to_string(&tool).unwrap();
    let parsed: ToolDefinition = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.name, "test");
}

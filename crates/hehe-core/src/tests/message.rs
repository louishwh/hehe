use crate::message::{ContentBlock, Message, Role, ToolUse};

#[test]
fn test_message_user() {
    let msg = Message::user("Hello");
    assert_eq!(msg.role, Role::User);
    assert_eq!(msg.text_content(), "Hello");
}

#[test]
fn test_message_assistant() {
    let msg = Message::assistant("Hi there!");
    assert_eq!(msg.role, Role::Assistant);
    assert_eq!(msg.text_content(), "Hi there!");
}

#[test]
fn test_message_tool_uses() {
    let tu = ToolUse::new("call_1", "read_file", serde_json::json!({"path": "/tmp"}));
    let msg = Message::new(
        Role::Assistant,
        vec![
            ContentBlock::text("Let me read that file."),
            ContentBlock::tool_use(tu),
        ],
    );
    assert!(msg.has_tool_use());
    assert_eq!(msg.tool_uses().len(), 1);
    assert_eq!(msg.tool_uses()[0].name, "read_file");
}

#[test]
fn test_message_serde() {
    let msg = Message::user("test");
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: Message = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.text_content(), "test");
    assert_eq!(parsed.role, Role::User);
}

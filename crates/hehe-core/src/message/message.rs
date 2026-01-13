use super::content::{ContentBlock, ImageContent, ToolUse};
use super::role::Role;
use crate::types::{MessageId, Metadata, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub role: Role,
    pub content: Vec<ContentBlock>,
    pub created_at: Timestamp,
    #[serde(default, skip_serializing_if = "Metadata::is_empty")]
    pub metadata: Metadata,
}

impl Message {
    pub fn new(role: Role, content: Vec<ContentBlock>) -> Self {
        Self {
            id: MessageId::new(),
            role,
            content,
            created_at: Timestamp::now(),
            metadata: Metadata::new(),
        }
    }

    pub fn with_id(mut self, id: MessageId) -> Self {
        self.id = id;
        self
    }

    pub fn with_metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn system(text: impl Into<String>) -> Self {
        Self::new(Role::System, vec![ContentBlock::text(text)])
    }

    pub fn user(text: impl Into<String>) -> Self {
        Self::new(Role::User, vec![ContentBlock::text(text)])
    }

    pub fn assistant(text: impl Into<String>) -> Self {
        Self::new(Role::Assistant, vec![ContentBlock::text(text)])
    }

    pub fn tool(content: Vec<ContentBlock>) -> Self {
        Self::new(Role::Tool, content)
    }

    pub fn text_content(&self) -> String {
        self.content
            .iter()
            .filter_map(|b| b.as_text())
            .collect::<Vec<_>>()
            .join("")
    }

    pub fn has_tool_use(&self) -> bool {
        self.content.iter().any(|b| b.is_tool_use())
    }

    pub fn has_tool_result(&self) -> bool {
        self.content.iter().any(|b| b.is_tool_result())
    }

    pub fn tool_uses(&self) -> Vec<&ToolUse> {
        self.content
            .iter()
            .filter_map(|b| b.as_tool_use())
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn push(&mut self, block: ContentBlock) {
        self.content.push(block);
    }
}

#[derive(Default)]
pub struct MessageBuilder {
    id: Option<MessageId>,
    role: Option<Role>,
    content: Vec<ContentBlock>,
    metadata: Metadata,
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: MessageId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn role(mut self, role: Role) -> Self {
        self.role = Some(role);
        self
    }

    pub fn system(self) -> Self {
        self.role(Role::System)
    }

    pub fn user(self) -> Self {
        self.role(Role::User)
    }

    pub fn assistant(self) -> Self {
        self.role(Role::Assistant)
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.content.push(ContentBlock::text(text));
        self
    }

    pub fn image(mut self, image: ImageContent) -> Self {
        self.content.push(ContentBlock::Image(image));
        self
    }

    pub fn content(mut self, block: ContentBlock) -> Self {
        self.content.push(block);
        self
    }

    pub fn contents(mut self, blocks: Vec<ContentBlock>) -> Self {
        self.content.extend(blocks);
        self
    }

    pub fn metadata<K: Into<String>, V: Serialize>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn build(self) -> Result<Message, &'static str> {
        let role = self.role.ok_or("role is required")?;
        if self.content.is_empty() {
            return Err("content is required");
        }
        let mut msg = Message::new(role, self.content);
        if let Some(id) = self.id {
            msg.id = id;
        }
        msg.metadata = self.metadata;
        Ok(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.text_content(), "Hello");
    }

    #[test]
    fn test_message_builder() {
        let msg = MessageBuilder::new()
            .user()
            .text("Hello")
            .text(" World")
            .build()
            .unwrap();

        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.text_content(), "Hello World");
        assert_eq!(msg.content.len(), 2);
    }
}

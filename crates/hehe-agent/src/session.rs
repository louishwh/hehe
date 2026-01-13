use hehe_core::{Id, Message, Metadata, Timestamp};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionStats {
    pub message_count: usize,
    pub tool_call_count: usize,
    pub iteration_count: usize,
}

impl Default for SessionStats {
    fn default() -> Self {
        Self {
            message_count: 0,
            tool_call_count: 0,
            iteration_count: 0,
        }
    }
}

#[derive(Debug)]
struct SessionInner {
    messages: Vec<Message>,
    stats: SessionStats,
}

#[derive(Clone, Debug)]
pub struct Session {
    id: Id,
    created_at: Timestamp,
    metadata: Metadata,
    inner: Arc<RwLock<SessionInner>>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            created_at: Timestamp::now(),
            metadata: Metadata::new(),
            inner: Arc::new(RwLock::new(SessionInner {
                messages: Vec::new(),
                stats: SessionStats::default(),
            })),
        }
    }

    pub fn with_id(id: Id) -> Self {
        Self {
            id,
            created_at: Timestamp::now(),
            metadata: Metadata::new(),
            inner: Arc::new(RwLock::new(SessionInner {
                messages: Vec::new(),
                stats: SessionStats::default(),
            })),
        }
    }

    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn created_at(&self) -> &Timestamp {
        &self.created_at
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn add_message(&self, message: Message) {
        let mut inner = self.inner.write().unwrap();
        inner.stats.message_count += 1;
        inner.messages.push(message);
    }

    pub fn add_messages(&self, messages: impl IntoIterator<Item = Message>) {
        let mut inner = self.inner.write().unwrap();
        for message in messages {
            inner.stats.message_count += 1;
            inner.messages.push(message);
        }
    }

    pub fn messages(&self) -> Vec<Message> {
        self.inner.read().unwrap().messages.clone()
    }

    pub fn message_count(&self) -> usize {
        self.inner.read().unwrap().messages.len()
    }

    pub fn last_messages(&self, n: usize) -> Vec<Message> {
        let inner = self.inner.read().unwrap();
        let len = inner.messages.len();
        if n >= len {
            inner.messages.clone()
        } else {
            inner.messages[len - n..].to_vec()
        }
    }

    pub fn clear(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.messages.clear();
    }

    pub fn stats(&self) -> SessionStats {
        self.inner.read().unwrap().stats.clone()
    }

    pub fn increment_tool_calls(&self, count: usize) {
        let mut inner = self.inner.write().unwrap();
        inner.stats.tool_call_count += count;
    }

    pub fn increment_iterations(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.stats.iteration_count += 1;
    }

    pub fn truncate_messages(&self, max_messages: usize) {
        let mut inner = self.inner.write().unwrap();
        if inner.messages.len() > max_messages {
            let remove_count = inner.messages.len() - max_messages;
            inner.messages.drain(0..remove_count);
        }
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hehe_core::Role;

    #[test]
    fn test_session_new() {
        let session = Session::new();
        assert_eq!(session.message_count(), 0);
    }

    #[test]
    fn test_session_add_message() {
        let session = Session::new();
        session.add_message(Message::user("Hello"));
        session.add_message(Message::assistant("Hi there!"));

        assert_eq!(session.message_count(), 2);

        let messages = session.messages();
        assert_eq!(messages[0].role, Role::User);
        assert_eq!(messages[1].role, Role::Assistant);
    }

    #[test]
    fn test_session_last_messages() {
        let session = Session::new();
        for i in 0..10 {
            session.add_message(Message::user(format!("Message {}", i)));
        }

        let last_3 = session.last_messages(3);
        assert_eq!(last_3.len(), 3);
    }

    #[test]
    fn test_session_truncate() {
        let session = Session::new();
        for i in 0..10 {
            session.add_message(Message::user(format!("Message {}", i)));
        }

        session.truncate_messages(5);
        assert_eq!(session.message_count(), 5);
    }

    #[test]
    fn test_session_stats() {
        let session = Session::new();
        session.add_message(Message::user("Hello"));
        session.increment_tool_calls(2);
        session.increment_iterations();

        let stats = session.stats();
        assert_eq!(stats.message_count, 1);
        assert_eq!(stats.tool_call_count, 2);
        assert_eq!(stats.iteration_count, 1);
    }

    #[test]
    fn test_session_clone_shares_state() {
        let session1 = Session::new();
        let session2 = session1.clone();

        session1.add_message(Message::user("Hello"));

        assert_eq!(session2.message_count(), 1);
    }
}

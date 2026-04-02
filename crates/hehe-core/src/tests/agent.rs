use crate::message::Message;
use crate::traits::{AgentEvent, AgentResponse, Session};
use crate::types::{Id, SessionId};

#[test]
fn test_session_new() {
    let session = Session::new();
    assert_eq!(session.message_count(), 0);
}

#[test]
fn test_session_messages() {
    let mut session = Session::new();
    session.add_message(Message::user("Hello"));
    session.add_message(Message::assistant("Hi!"));
    assert_eq!(session.message_count(), 2);

    let last = session.last_messages(1);
    assert_eq!(last.len(), 1);
    assert_eq!(last[0].text_content(), "Hi!");
}

#[test]
fn test_agent_response() {
    let resp = AgentResponse::new(SessionId::new(), "Hello!");
    assert_eq!(resp.text(), "Hello!");
    assert!(!resp.has_tool_calls());
}

#[test]
fn test_agent_event_serde() {
    let event = AgentEvent::text_delta("chunk");
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("text_delta"));
    assert!(json.contains("chunk"));
}

#[test]
fn test_agent_event_is_end() {
    assert!(AgentEvent::error("oops").is_end());
    assert!(AgentEvent::MessageEnd {
        session_id: Id::new()
    }
    .is_end());
    assert!(!AgentEvent::text_delta("hi").is_end());
}

use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use futures::stream::Stream;
use hehe_agent::AgentEvent;
use hehe_core::Id;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::str::FromStr;
use tokio_stream::StreamExt;

use crate::error::{Result, ServerError};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ChatRequest {
    pub session_id: Option<String>,
    pub message: String,
    #[serde(default)]
    pub stream: bool,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub session_id: String,
    pub response: String,
    pub tool_calls: Vec<ToolCallInfo>,
    pub iterations: usize,
}

#[derive(Serialize)]
pub struct ToolCallInfo {
    pub id: String,
    pub name: String,
    pub output: String,
    pub is_error: bool,
}

pub async fn chat(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>> {
    let session_id = request.session_id.and_then(|s| Id::from_str(&s).ok());
    let session = state.get_or_create_session(session_id).await;

    let response = state
        .agent
        .process(&session, &request.message)
        .await
        .map_err(ServerError::from)?;

    Ok(Json(ChatResponse {
        session_id: session.id().to_string(),
        response: response.text,
        tool_calls: response
            .tool_calls
            .into_iter()
            .map(|tc| ToolCallInfo {
                id: tc.id,
                name: tc.name,
                output: tc.output,
                is_error: tc.is_error,
            })
            .collect(),
        iterations: response.iterations,
    }))
}

pub async fn chat_stream(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Sse<impl Stream<Item = std::result::Result<Event, Infallible>>> {
    let session_id = request.session_id.and_then(|s| Id::from_str(&s).ok());
    let session = state.get_or_create_session(session_id).await;
    let message = request.message;

    let event_stream = state.agent.chat_stream(&session, &message);

    let sse_stream = event_stream.map(|event| {
        let data = match &event {
            AgentEvent::MessageStart { session_id } => {
                serde_json::json!({
                    "type": "message_start",
                    "session_id": session_id.to_string()
                })
            }
            AgentEvent::TextDelta { delta } => {
                serde_json::json!({
                    "type": "text_delta",
                    "delta": delta
                })
            }
            AgentEvent::TextComplete { text } => {
                serde_json::json!({
                    "type": "text_complete",
                    "text": text
                })
            }
            AgentEvent::ToolUseStart { id, name, input } => {
                serde_json::json!({
                    "type": "tool_use_start",
                    "id": id,
                    "name": name,
                    "input": input
                })
            }
            AgentEvent::ToolUseEnd { id, output, is_error } => {
                serde_json::json!({
                    "type": "tool_use_end",
                    "id": id,
                    "output": output,
                    "is_error": is_error
                })
            }
            AgentEvent::Thinking { content } => {
                serde_json::json!({
                    "type": "thinking",
                    "content": content
                })
            }
            AgentEvent::MessageEnd { session_id } => {
                serde_json::json!({
                    "type": "message_end",
                    "session_id": session_id.to_string()
                })
            }
            AgentEvent::Error { message } => {
                serde_json::json!({
                    "type": "error",
                    "message": message
                })
            }
        };

        Ok(Event::default().data(data.to_string()))
    });

    Sse::new(sse_stream).keep_alive(KeepAlive::default())
}

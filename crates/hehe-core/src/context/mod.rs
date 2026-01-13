use crate::types::{AgentId, RequestId, SessionId, Timestamp};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub type TraceId = String;

#[derive(Clone)]
pub struct Context {
    pub request_id: RequestId,
    pub trace_id: Option<TraceId>,
    pub parent_span_id: Option<String>,
    pub agent_id: Option<AgentId>,
    pub session_id: Option<SessionId>,
    pub started_at: Timestamp,
    pub deadline: Option<Timestamp>,
    cancellation: CancellationToken,
    extensions: Arc<HashMap<String, String>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            request_id: RequestId::new(),
            trace_id: None,
            parent_span_id: None,
            agent_id: None,
            session_id: None,
            started_at: Timestamp::now(),
            deadline: None,
            cancellation: CancellationToken::new(),
            extensions: Arc::new(HashMap::new()),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        let deadline_ms = self.started_at.unix_millis() + timeout.as_millis() as i64;
        self.deadline = Timestamp::from_unix_millis(deadline_ms);
        self
    }

    pub fn with_deadline(mut self, deadline: Timestamp) -> Self {
        self.deadline = Some(deadline);
        self
    }

    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    pub fn with_agent(mut self, agent_id: AgentId) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    pub fn with_session(mut self, session_id: SessionId) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_cancellation(mut self, token: CancellationToken) -> Self {
        self.cancellation = token;
        self
    }

    pub fn child(&self) -> Self {
        Self {
            request_id: RequestId::new(),
            trace_id: self.trace_id.clone(),
            parent_span_id: Some(self.request_id.to_string()),
            agent_id: self.agent_id,
            session_id: self.session_id,
            started_at: Timestamp::now(),
            deadline: self.deadline,
            cancellation: self.cancellation.child_token(),
            extensions: Arc::clone(&self.extensions),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancellation.is_cancelled()
    }

    pub fn cancel(&self) {
        self.cancellation.cancel()
    }

    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation.clone()
    }

    pub fn is_timeout(&self) -> bool {
        if let Some(deadline) = self.deadline {
            Timestamp::now() > deadline
        } else {
            false
        }
    }

    pub fn is_done(&self) -> bool {
        self.is_cancelled() || self.is_timeout()
    }

    pub fn remaining(&self) -> Option<Duration> {
        self.deadline.map(|d| {
            let now = Timestamp::now().unix_millis();
            let deadline = d.unix_millis();
            if deadline > now {
                Duration::from_millis((deadline - now) as u64)
            } else {
                Duration::ZERO
            }
        })
    }

    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    pub fn get_extension(&self, key: &str) -> Option<&String> {
        self.extensions.get(key)
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("request_id", &self.request_id)
            .field("trace_id", &self.trace_id)
            .field("agent_id", &self.agent_id)
            .field("session_id", &self.session_id)
            .field("started_at", &self.started_at)
            .field("deadline", &self.deadline)
            .field("is_cancelled", &self.is_cancelled())
            .finish()
    }
}

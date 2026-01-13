use hehe_agent::{Agent, Session};
use hehe_core::Id;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub agent: Arc<Agent>,
    sessions: Arc<RwLock<HashMap<Id, Session>>>,
}

impl AppState {
    pub fn new(agent: Agent) -> Self {
        Self {
            agent: Arc::new(agent),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_or_create_session(&self, session_id: Option<Id>) -> Session {
        match session_id {
            Some(id) => {
                let sessions = self.sessions.read().await;
                if let Some(session) = sessions.get(&id) {
                    return session.clone();
                }
                drop(sessions);

                let session = Session::with_id(id.clone());
                self.sessions.write().await.insert(id, session.clone());
                session
            }
            None => {
                let session = self.agent.create_session();
                self.sessions
                    .write()
                    .await
                    .insert(session.id().clone(), session.clone());
                session
            }
        }
    }

    pub async fn get_session(&self, session_id: &Id) -> Option<Session> {
        self.sessions.read().await.get(session_id).cloned()
    }

    pub async fn remove_session(&self, session_id: &Id) -> Option<Session> {
        self.sessions.write().await.remove(session_id)
    }

    pub async fn session_count(&self) -> usize {
        self.sessions.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use hehe_agent::AgentConfig;
    use hehe_core::capability::Capabilities;
    use hehe_core::stream::StreamChunk;
    use hehe_core::Message;
    use hehe_llm::{BoxStream, CompletionRequest, CompletionResponse, LlmError, LlmProvider, ModelInfo};

    struct MockLlm;

    #[async_trait]
    impl LlmProvider for MockLlm {
        fn name(&self) -> &str { "mock" }
        fn capabilities(&self) -> &Capabilities {
            static CAPS: std::sync::OnceLock<Capabilities> = std::sync::OnceLock::new();
            CAPS.get_or_init(Capabilities::text_basic)
        }
        async fn complete(&self, _: CompletionRequest) -> std::result::Result<CompletionResponse, LlmError> {
            Ok(CompletionResponse::new("id", "mock", Message::assistant("Hi")))
        }
        async fn complete_stream(&self, _: CompletionRequest) -> std::result::Result<BoxStream<StreamChunk>, LlmError> {
            use futures::stream;
            Ok(Box::pin(stream::empty()))
        }
        async fn list_models(&self) -> std::result::Result<Vec<ModelInfo>, LlmError> { Ok(vec![]) }
        fn default_model(&self) -> &str { "mock" }
    }

    fn create_test_agent() -> Agent {
        Agent::builder()
            .system_prompt("Test")
            .llm(Arc::new(MockLlm))
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_app_state_create_session() {
        let state = AppState::new(create_test_agent());
        
        let session = state.get_or_create_session(None).await;
        assert_eq!(state.session_count().await, 1);

        let session2 = state.get_or_create_session(Some(session.id().clone())).await;
        assert_eq!(session.id(), session2.id());
        assert_eq!(state.session_count().await, 1);
    }

    #[tokio::test]
    async fn test_app_state_remove_session() {
        let state = AppState::new(create_test_agent());
        let session = state.get_or_create_session(None).await;
        
        assert!(state.remove_session(session.id()).await.is_some());
        assert_eq!(state.session_count().await, 0);
    }
}

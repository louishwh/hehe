use crate::error::Result;
use crate::types::{CompletionRequest, CompletionResponse, ModelInfo};
use async_trait::async_trait;
use futures::Stream;
use hehe_core::capability::Capabilities;
use hehe_core::stream::StreamChunk;
use std::pin::Pin;

pub type BoxStream<T> = Pin<Box<dyn Stream<Item = Result<T>> + Send>>;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;

    fn capabilities(&self) -> &Capabilities;

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;

    async fn complete_stream(&self, request: CompletionRequest) -> Result<BoxStream<StreamChunk>>;

    async fn list_models(&self) -> Result<Vec<ModelInfo>>;

    async fn health_check(&self) -> Result<()> {
        let _ = self.list_models().await?;
        Ok(())
    }

    fn default_model(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;
    use hehe_core::Message;

    struct MockProvider;

    #[async_trait]
    impl LlmProvider for MockProvider {
        fn name(&self) -> &str {
            "mock"
        }

        fn capabilities(&self) -> &Capabilities {
            static CAPS: std::sync::OnceLock<Capabilities> = std::sync::OnceLock::new();
            CAPS.get_or_init(Capabilities::text_basic)
        }

        async fn complete(&self, _request: CompletionRequest) -> Result<CompletionResponse> {
            Ok(CompletionResponse::new(
                "mock-id",
                "mock-model",
                Message::assistant("Mock response"),
            ))
        }

        async fn complete_stream(
            &self,
            _request: CompletionRequest,
        ) -> Result<BoxStream<StreamChunk>> {
            use futures::stream;
            Ok(Box::pin(stream::empty()))
        }

        async fn list_models(&self) -> Result<Vec<ModelInfo>> {
            Ok(vec![ModelInfo::new("mock-model", "mock")])
        }

        fn default_model(&self) -> &str {
            "mock-model"
        }
    }

    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockProvider;

        assert_eq!(provider.name(), "mock");
        assert!(provider.capabilities().has(&hehe_core::capability::Capability::TextInput));

        let response = provider
            .complete(CompletionRequest::new("mock-model", vec![Message::user("Hi")]))
            .await
            .unwrap();

        assert_eq!(response.text_content(), "Mock response");
    }
}

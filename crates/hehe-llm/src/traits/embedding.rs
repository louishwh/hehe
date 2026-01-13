use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    fn name(&self) -> &str;

    fn dimension(&self) -> usize;

    fn max_batch_size(&self) -> usize {
        100
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;

    async fn embed_one(&self, text: &str) -> Result<Vec<f32>> {
        let results = self.embed(&[text.to_string()]).await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| crate::error::LlmError::invalid_response("Empty embedding result"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEmbedding;

    #[async_trait]
    impl EmbeddingProvider for MockEmbedding {
        fn name(&self) -> &str {
            "mock-embedding"
        }

        fn dimension(&self) -> usize {
            3
        }

        async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
            Ok(texts.iter().map(|_| vec![0.1, 0.2, 0.3]).collect())
        }
    }

    #[tokio::test]
    async fn test_mock_embedding() {
        let provider = MockEmbedding;

        assert_eq!(provider.dimension(), 3);

        let embeddings = provider
            .embed(&["hello".to_string(), "world".to_string()])
            .await
            .unwrap();

        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 3);
    }

    #[tokio::test]
    async fn test_embed_one() {
        let provider = MockEmbedding;

        let embedding = provider.embed_one("test").await.unwrap();
        assert_eq!(embedding, vec![0.1, 0.2, 0.3]);
    }
}

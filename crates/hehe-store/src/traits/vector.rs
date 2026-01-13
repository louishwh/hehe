use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VectorRecord {
    pub id: String,
    pub vector: Vec<f32>,
    #[serde(default)]
    pub metadata: HashMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

impl VectorRecord {
    pub fn new(id: impl Into<String>, vector: Vec<f32>) -> Self {
        Self {
            id: id.into(),
            vector,
            metadata: HashMap::new(),
            content: None,
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    #[serde(default)]
    pub metadata: HashMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct VectorFilter {
    pub conditions: Vec<FilterCondition>,
}

#[derive(Clone, Debug)]
pub enum FilterCondition {
    Eq(String, Value),
    Ne(String, Value),
    Gt(String, Value),
    Gte(String, Value),
    Lt(String, Value),
    Lte(String, Value),
    In(String, Vec<Value>),
    Contains(String, String),
}

impl VectorFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn eq(mut self, field: impl Into<String>, value: impl Into<Value>) -> Self {
        self.conditions
            .push(FilterCondition::Eq(field.into(), value.into()));
        self
    }

    pub fn ne(mut self, field: impl Into<String>, value: impl Into<Value>) -> Self {
        self.conditions
            .push(FilterCondition::Ne(field.into(), value.into()));
        self
    }

    pub fn gt(mut self, field: impl Into<String>, value: impl Into<Value>) -> Self {
        self.conditions
            .push(FilterCondition::Gt(field.into(), value.into()));
        self
    }

    pub fn lt(mut self, field: impl Into<String>, value: impl Into<Value>) -> Self {
        self.conditions
            .push(FilterCondition::Lt(field.into(), value.into()));
        self
    }

    pub fn contains(mut self, field: impl Into<String>, value: impl Into<String>) -> Self {
        self.conditions
            .push(FilterCondition::Contains(field.into(), value.into()));
        self
    }

    pub fn is_empty(&self) -> bool {
        self.conditions.is_empty()
    }
}

#[derive(Clone, Debug)]
pub struct CollectionInfo {
    pub name: String,
    pub dimension: usize,
    pub count: usize,
}

#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn create_collection(&self, name: &str, dimension: usize) -> Result<()>;

    async fn delete_collection(&self, name: &str) -> Result<()>;

    async fn list_collections(&self) -> Result<Vec<CollectionInfo>>;

    async fn collection_exists(&self, name: &str) -> Result<bool>;

    async fn upsert(&self, collection: &str, records: &[VectorRecord]) -> Result<usize>;

    async fn search(
        &self,
        collection: &str,
        query: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>>;

    async fn search_with_filter(
        &self,
        collection: &str,
        query: &[f32],
        filter: &VectorFilter,
        limit: usize,
    ) -> Result<Vec<SearchResult>>;

    async fn get(&self, collection: &str, id: &str) -> Result<Option<VectorRecord>>;

    async fn delete(&self, collection: &str, ids: &[String]) -> Result<usize>;

    async fn count(&self, collection: &str) -> Result<usize>;

    fn backend_name(&self) -> &'static str;
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a * norm_b)
}

pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return f32::MAX;
    }

    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_record() {
        let record = VectorRecord::new("id1", vec![0.1, 0.2, 0.3])
            .with_metadata("key", "value")
            .with_content("some text");

        assert_eq!(record.id, "id1");
        assert_eq!(record.vector.len(), 3);
        assert!(record.metadata.contains_key("key"));
        assert_eq!(record.content, Some("some text".into()));
    }

    #[test]
    fn test_vector_filter() {
        let filter = VectorFilter::new()
            .eq("type", "article")
            .gt("score", 0.5);

        assert_eq!(filter.conditions.len(), 2);
        assert!(!filter.is_empty());
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.0001);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 0.0001);

        let d = vec![-1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &d) - (-1.0)).abs() < 0.0001);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((euclidean_distance(&a, &b) - 1.0).abs() < 0.0001);

        let c = vec![3.0, 4.0, 0.0];
        assert!((euclidean_distance(&a, &c) - 5.0).abs() < 0.0001);
    }
}

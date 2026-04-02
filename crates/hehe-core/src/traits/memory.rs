use crate::error::Result;
use crate::types::{Id, Metadata, Timestamp};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryKind {
    ShortTerm,
    LongTerm,
    System,
    Episodic,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: Id,
    pub kind: MemoryKind,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
    #[serde(default)]
    pub metadata: Metadata,
    pub created_at: Timestamp,
}

impl MemoryEntry {
    pub fn new(kind: MemoryKind, content: impl Into<String>) -> Self {
        Self {
            id: Id::new(),
            kind,
            content: content.into(),
            embedding: None,
            metadata: Metadata::new(),
            created_at: Timestamp::now(),
        }
    }

    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }

    pub fn with_metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = metadata;
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct MemoryFilter {
    pub kind: Option<MemoryKind>,
    pub query: Option<String>,
    pub limit: Option<usize>,
}

impl MemoryFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn kind(mut self, kind: MemoryKind) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

#[async_trait]
pub trait MemoryStore: Send + Sync {
    async fn store(&self, entry: MemoryEntry) -> Result<Id>;
    async fn recall(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>>;
    async fn get(&self, id: &Id) -> Result<Option<MemoryEntry>>;
    async fn delete(&self, id: &Id) -> Result<bool>;
    async fn search(&self, filter: MemoryFilter) -> Result<Vec<MemoryEntry>>;
}

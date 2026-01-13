use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub content: String,
    #[serde(default)]
    pub fields: HashMap<String, Value>,
}

impl Document {
    pub fn new(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            fields: HashMap::new(),
        }
    }

    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchHit {
    pub id: String,
    pub score: f32,
    pub content: String,
    #[serde(default)]
    pub highlights: Vec<String>,
    #[serde(default)]
    pub fields: HashMap<String, Value>,
}

#[derive(Clone, Debug, Default)]
pub struct SearchFilter {
    pub conditions: Vec<SearchCondition>,
}

#[derive(Clone, Debug)]
pub enum SearchCondition {
    Eq(String, Value),
    Range(String, Option<Value>, Option<Value>),
    In(String, Vec<Value>),
}

impl SearchFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn eq(mut self, field: impl Into<String>, value: impl Into<Value>) -> Self {
        self.conditions
            .push(SearchCondition::Eq(field.into(), value.into()));
        self
    }

    pub fn range(
        mut self,
        field: impl Into<String>,
        min: Option<Value>,
        max: Option<Value>,
    ) -> Self {
        self.conditions
            .push(SearchCondition::Range(field.into(), min, max));
        self
    }

    pub fn is_empty(&self) -> bool {
        self.conditions.is_empty()
    }
}

#[derive(Clone, Debug)]
pub struct IndexSchema {
    pub fields: Vec<IndexField>,
}

#[derive(Clone, Debug)]
pub struct IndexField {
    pub name: String,
    pub field_type: IndexFieldType,
    pub stored: bool,
    pub indexed: bool,
}

#[derive(Clone, Debug)]
pub enum IndexFieldType {
    Text,
    Keyword,
    Integer,
    Float,
    Boolean,
    Date,
}

impl IndexSchema {
    pub fn new() -> Self {
        Self { fields: vec![] }
    }

    pub fn add_text(mut self, name: impl Into<String>) -> Self {
        self.fields.push(IndexField {
            name: name.into(),
            field_type: IndexFieldType::Text,
            stored: true,
            indexed: true,
        });
        self
    }

    pub fn add_keyword(mut self, name: impl Into<String>) -> Self {
        self.fields.push(IndexField {
            name: name.into(),
            field_type: IndexFieldType::Keyword,
            stored: true,
            indexed: true,
        });
        self
    }

    pub fn add_integer(mut self, name: impl Into<String>) -> Self {
        self.fields.push(IndexField {
            name: name.into(),
            field_type: IndexFieldType::Integer,
            stored: true,
            indexed: true,
        });
        self
    }
}

impl Default for IndexSchema {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
pub trait SearchStore: Send + Sync {
    async fn create_index(&self, name: &str, schema: &IndexSchema) -> Result<()>;

    async fn delete_index(&self, name: &str) -> Result<()>;

    async fn index_exists(&self, name: &str) -> Result<bool>;

    async fn list_indexes(&self) -> Result<Vec<String>>;

    async fn index_documents(&self, index: &str, docs: &[Document]) -> Result<usize>;

    async fn delete_documents(&self, index: &str, ids: &[String]) -> Result<usize>;

    async fn search(&self, index: &str, query: &str, limit: usize) -> Result<Vec<SearchHit>>;

    async fn search_with_filter(
        &self,
        index: &str,
        query: &str,
        filter: &SearchFilter,
        limit: usize,
    ) -> Result<Vec<SearchHit>>;

    async fn count(&self, index: &str) -> Result<usize>;

    fn backend_name(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document() {
        let doc = Document::new("doc1", "Hello world")
            .with_field("category", "greeting")
            .with_field("score", 0.9);

        assert_eq!(doc.id, "doc1");
        assert_eq!(doc.content, "Hello world");
        assert_eq!(doc.fields.len(), 2);
    }

    #[test]
    fn test_index_schema() {
        let schema = IndexSchema::new()
            .add_text("title")
            .add_text("content")
            .add_keyword("category");

        assert_eq!(schema.fields.len(), 3);
    }

    #[test]
    fn test_search_filter() {
        let filter = SearchFilter::new()
            .eq("category", "article")
            .range("date", Some(Value::String("2024-01-01".into())), None);

        assert_eq!(filter.conditions.len(), 2);
    }
}

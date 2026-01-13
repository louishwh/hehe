use crate::error::{Result, StoreError};
use crate::traits::{
    cosine_similarity, CollectionInfo, SearchResult, VectorFilter, VectorRecord, VectorStore,
};
use async_trait::async_trait;
use parking_lot::RwLock;
use serde_json::Value;
use std::collections::HashMap;

struct Collection {
    dimension: usize,
    records: HashMap<String, VectorRecord>,
}

pub struct MemoryVectorStore {
    collections: RwLock<HashMap<String, Collection>>,
}

impl MemoryVectorStore {
    pub fn new() -> Self {
        Self {
            collections: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for MemoryVectorStore {
    fn default() -> Self {
        Self::new()
    }
}

fn matches_filter(record: &VectorRecord, filter: &VectorFilter) -> bool {
    use crate::traits::vector::FilterCondition;

    for condition in &filter.conditions {
        let matched = match condition {
            FilterCondition::Eq(field, value) => {
                record.metadata.get(field).map(|v| v == value).unwrap_or(false)
            }
            FilterCondition::Ne(field, value) => {
                record.metadata.get(field).map(|v| v != value).unwrap_or(true)
            }
            FilterCondition::Gt(field, value) => match (record.metadata.get(field), value) {
                (Some(Value::Number(a)), Value::Number(b)) => {
                    a.as_f64().unwrap_or(0.0) > b.as_f64().unwrap_or(0.0)
                }
                _ => false,
            },
            FilterCondition::Gte(field, value) => match (record.metadata.get(field), value) {
                (Some(Value::Number(a)), Value::Number(b)) => {
                    a.as_f64().unwrap_or(0.0) >= b.as_f64().unwrap_or(0.0)
                }
                _ => false,
            },
            FilterCondition::Lt(field, value) => match (record.metadata.get(field), value) {
                (Some(Value::Number(a)), Value::Number(b)) => {
                    a.as_f64().unwrap_or(0.0) < b.as_f64().unwrap_or(0.0)
                }
                _ => false,
            },
            FilterCondition::Lte(field, value) => match (record.metadata.get(field), value) {
                (Some(Value::Number(a)), Value::Number(b)) => {
                    a.as_f64().unwrap_or(0.0) <= b.as_f64().unwrap_or(0.0)
                }
                _ => false,
            },
            FilterCondition::In(field, values) => record
                .metadata
                .get(field)
                .map(|v| values.contains(v))
                .unwrap_or(false),
            FilterCondition::Contains(field, substr) => record
                .metadata
                .get(field)
                .and_then(|v| v.as_str())
                .map(|s| s.contains(substr))
                .unwrap_or(false),
        };

        if !matched {
            return false;
        }
    }

    true
}

#[async_trait]
impl VectorStore for MemoryVectorStore {
    async fn create_collection(&self, name: &str, dimension: usize) -> Result<()> {
        let mut collections = self.collections.write();
        if collections.contains_key(name) {
            return Err(StoreError::AlreadyExists(format!("Collection '{}'", name)));
        }
        collections.insert(
            name.to_string(),
            Collection {
                dimension,
                records: HashMap::new(),
            },
        );
        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        let mut collections = self.collections.write();
        if collections.remove(name).is_none() {
            return Err(StoreError::not_found(format!("Collection '{}'", name)));
        }
        Ok(())
    }

    async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        let collections = self.collections.read();
        Ok(collections
            .iter()
            .map(|(name, col)| CollectionInfo {
                name: name.clone(),
                dimension: col.dimension,
                count: col.records.len(),
            })
            .collect())
    }

    async fn collection_exists(&self, name: &str) -> Result<bool> {
        Ok(self.collections.read().contains_key(name))
    }

    async fn upsert(&self, collection: &str, records: &[VectorRecord]) -> Result<usize> {
        let mut collections = self.collections.write();
        let col = collections
            .get_mut(collection)
            .ok_or_else(|| StoreError::not_found(format!("Collection '{}'", collection)))?;

        let mut count = 0;
        for record in records {
            if record.vector.len() != col.dimension {
                return Err(StoreError::invalid_input(format!(
                    "Vector dimension mismatch: expected {}, got {}",
                    col.dimension,
                    record.vector.len()
                )));
            }
            col.records.insert(record.id.clone(), record.clone());
            count += 1;
        }

        Ok(count)
    }

    async fn search(
        &self,
        collection: &str,
        query: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        self.search_with_filter(collection, query, &VectorFilter::default(), limit)
            .await
    }

    async fn search_with_filter(
        &self,
        collection: &str,
        query: &[f32],
        filter: &VectorFilter,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let collections = self.collections.read();
        let col = collections
            .get(collection)
            .ok_or_else(|| StoreError::not_found(format!("Collection '{}'", collection)))?;

        if query.len() != col.dimension {
            return Err(StoreError::invalid_input(format!(
                "Query dimension mismatch: expected {}, got {}",
                col.dimension,
                query.len()
            )));
        }

        let mut scored: Vec<(String, f32, HashMap<String, Value>, Option<String>)> = col
            .records
            .values()
            .filter(|r| filter.is_empty() || matches_filter(r, filter))
            .map(|r| {
                let score = cosine_similarity(query, &r.vector);
                (r.id.clone(), score, r.metadata.clone(), r.content.clone())
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(scored
            .into_iter()
            .take(limit)
            .map(|(id, score, metadata, content)| SearchResult {
                id,
                score,
                metadata,
                content,
            })
            .collect())
    }

    async fn get(&self, collection: &str, id: &str) -> Result<Option<VectorRecord>> {
        let collections = self.collections.read();
        let col = collections
            .get(collection)
            .ok_or_else(|| StoreError::not_found(format!("Collection '{}'", collection)))?;

        Ok(col.records.get(id).cloned())
    }

    async fn delete(&self, collection: &str, ids: &[String]) -> Result<usize> {
        let mut collections = self.collections.write();
        let col = collections
            .get_mut(collection)
            .ok_or_else(|| StoreError::not_found(format!("Collection '{}'", collection)))?;

        let mut count = 0;
        for id in ids {
            if col.records.remove(id).is_some() {
                count += 1;
            }
        }

        Ok(count)
    }

    async fn count(&self, collection: &str) -> Result<usize> {
        let collections = self.collections.read();
        let col = collections
            .get(collection)
            .ok_or_else(|| StoreError::not_found(format!("Collection '{}'", collection)))?;

        Ok(col.records.len())
    }

    fn backend_name(&self) -> &'static str {
        "memory"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_collection_lifecycle() {
        let store = MemoryVectorStore::new();

        assert!(!store.collection_exists("test").await.unwrap());

        store.create_collection("test", 3).await.unwrap();
        assert!(store.collection_exists("test").await.unwrap());

        let err = store.create_collection("test", 3).await;
        assert!(err.is_err());

        store.delete_collection("test").await.unwrap();
        assert!(!store.collection_exists("test").await.unwrap());
    }

    #[tokio::test]
    async fn test_upsert_and_search() {
        let store = MemoryVectorStore::new();
        store.create_collection("docs", 3).await.unwrap();

        let records = vec![
            VectorRecord::new("doc1", vec![1.0, 0.0, 0.0])
                .with_metadata("category", "a")
                .with_content("Document one"),
            VectorRecord::new("doc2", vec![0.0, 1.0, 0.0])
                .with_metadata("category", "b")
                .with_content("Document two"),
            VectorRecord::new("doc3", vec![0.707, 0.707, 0.0])
                .with_metadata("category", "a")
                .with_content("Document three"),
        ];

        let count = store.upsert("docs", &records).await.unwrap();
        assert_eq!(count, 3);
        assert_eq!(store.count("docs").await.unwrap(), 3);

        let results = store.search("docs", &[1.0, 0.0, 0.0], 2).await.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "doc1");
        assert!((results[0].score - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_search_with_filter() {
        let store = MemoryVectorStore::new();
        store.create_collection("items", 2).await.unwrap();

        let records = vec![
            VectorRecord::new("item1", vec![1.0, 0.0]).with_metadata("type", "book"),
            VectorRecord::new("item2", vec![0.9, 0.1]).with_metadata("type", "book"),
            VectorRecord::new("item3", vec![0.8, 0.2]).with_metadata("type", "video"),
        ];
        store.upsert("items", &records).await.unwrap();

        let filter = VectorFilter::new().eq("type", "book");
        let results = store
            .search_with_filter("items", &[1.0, 0.0], &filter, 10)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        for r in &results {
            assert_eq!(r.metadata.get("type"), Some(&Value::String("book".into())));
        }
    }

    #[tokio::test]
    async fn test_get_and_delete() {
        let store = MemoryVectorStore::new();
        store.create_collection("test", 2).await.unwrap();

        store
            .upsert("test", &[VectorRecord::new("id1", vec![1.0, 0.0])])
            .await
            .unwrap();

        let record = store.get("test", "id1").await.unwrap();
        assert!(record.is_some());
        assert_eq!(record.unwrap().id, "id1");

        let deleted = store
            .delete("test", &["id1".to_string()])
            .await
            .unwrap();
        assert_eq!(deleted, 1);

        let record = store.get("test", "id1").await.unwrap();
        assert!(record.is_none());
    }

    #[tokio::test]
    async fn test_dimension_validation() {
        let store = MemoryVectorStore::new();
        store.create_collection("test", 3).await.unwrap();

        let result = store
            .upsert("test", &[VectorRecord::new("id1", vec![1.0, 0.0])])
            .await;
        assert!(result.is_err());

        let result = store.search("test", &[1.0, 0.0], 10).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_collections() {
        let store = MemoryVectorStore::new();

        store.create_collection("col1", 10).await.unwrap();
        store.create_collection("col2", 20).await.unwrap();

        let list = store.list_collections().await.unwrap();
        assert_eq!(list.len(), 2);

        let names: Vec<_> = list.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"col1"));
        assert!(names.contains(&"col2"));
    }

    #[tokio::test]
    async fn test_upsert_updates_existing() {
        let store = MemoryVectorStore::new();
        store.create_collection("test", 2).await.unwrap();

        store
            .upsert(
                "test",
                &[VectorRecord::new("id1", vec![1.0, 0.0]).with_metadata("version", 1)],
            )
            .await
            .unwrap();

        store
            .upsert(
                "test",
                &[VectorRecord::new("id1", vec![0.0, 1.0]).with_metadata("version", 2)],
            )
            .await
            .unwrap();

        assert_eq!(store.count("test").await.unwrap(), 1);

        let record = store.get("test", "id1").await.unwrap().unwrap();
        assert_eq!(record.vector, vec![0.0, 1.0]);
        assert_eq!(
            record.metadata.get("version"),
            Some(&Value::Number(2.into()))
        );
    }
}

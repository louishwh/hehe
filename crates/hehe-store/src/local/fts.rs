use crate::error::{Result, StoreError};
use crate::local::SqliteStore;
use crate::traits::{Document, IndexSchema, RelationalStore, SearchFilter, SearchHit, SearchStore};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

pub struct SqliteFtsStore {
    db: Arc<SqliteStore>,
}

impl SqliteFtsStore {
    pub fn new(db: Arc<SqliteStore>) -> Self {
        Self { db }
    }

    pub async fn from_path(path: &str) -> Result<Self> {
        let db = SqliteStore::open(path)?;
        Ok(Self { db: Arc::new(db) })
    }

    pub async fn memory() -> Result<Self> {
        let db = SqliteStore::memory()?;
        Ok(Self { db: Arc::new(db) })
    }

    fn table_name(index: &str) -> String {
        format!("fts_{}", index.replace('-', "_"))
    }
}

#[async_trait]
impl SearchStore for SqliteFtsStore {
    async fn create_index(&self, name: &str, _schema: &IndexSchema) -> Result<()> {
        let table = Self::table_name(name);

        let sql = format!(
            "CREATE VIRTUAL TABLE IF NOT EXISTS {} USING fts5(
                id,
                content,
                fields,
                tokenize='porter unicode61'
            )",
            table
        );

        self.db.execute(&sql, &[]).await?;
        Ok(())
    }

    async fn delete_index(&self, name: &str) -> Result<()> {
        let table = Self::table_name(name);
        let sql = format!("DROP TABLE IF EXISTS {}", table);
        self.db.execute(&sql, &[]).await?;
        Ok(())
    }

    async fn index_exists(&self, name: &str) -> Result<bool> {
        let table = Self::table_name(name);
        self.db.table_exists(&table).await
    }

    async fn list_indexes(&self) -> Result<Vec<String>> {
        let rows = self
            .db
            .query(
                "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE 'fts_%'",
                &[],
            )
            .await?;

        Ok(rows
            .iter()
            .filter_map(|r| r.get_str("name"))
            .map(|s| s.strip_prefix("fts_").unwrap_or(s).replace('_', "-"))
            .collect())
    }

    async fn index_documents(&self, index: &str, docs: &[Document]) -> Result<usize> {
        let table = Self::table_name(index);

        if !self.index_exists(index).await? {
            return Err(StoreError::not_found(format!("Index '{}'", index)));
        }

        let mut count = 0;
        for doc in docs {
            let fields_json = serde_json::to_string(&doc.fields)
                .map_err(|e| StoreError::Serialization(e.to_string()))?;

            self.db
                .execute(
                    &format!("DELETE FROM {} WHERE id = ?1", table),
                    &[Value::String(doc.id.clone())],
                )
                .await?;

            self.db
                .execute(
                    &format!(
                        "INSERT INTO {} (id, content, fields) VALUES (?1, ?2, ?3)",
                        table
                    ),
                    &[
                        Value::String(doc.id.clone()),
                        Value::String(doc.content.clone()),
                        Value::String(fields_json),
                    ],
                )
                .await?;

            count += 1;
        }

        Ok(count)
    }

    async fn delete_documents(&self, index: &str, ids: &[String]) -> Result<usize> {
        let table = Self::table_name(index);

        if !self.index_exists(index).await? {
            return Err(StoreError::not_found(format!("Index '{}'", index)));
        }

        let mut count = 0;
        for id in ids {
            let affected = self
                .db
                .execute(
                    &format!("DELETE FROM {} WHERE id = ?1", table),
                    &[Value::String(id.clone())],
                )
                .await?;
            count += affected as usize;
        }

        Ok(count)
    }

    async fn search(&self, index: &str, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
        self.search_with_filter(index, query, &SearchFilter::default(), limit)
            .await
    }

    async fn search_with_filter(
        &self,
        index: &str,
        query: &str,
        _filter: &SearchFilter,
        limit: usize,
    ) -> Result<Vec<SearchHit>> {
        let table = Self::table_name(index);

        if !self.index_exists(index).await? {
            return Err(StoreError::not_found(format!("Index '{}'", index)));
        }

        let escaped_query = query.replace('"', "\"\"");

        let sql = format!(
            "SELECT id, content, fields, bm25({}) as score
             FROM {} 
             WHERE {} MATCH ?1
             ORDER BY score
             LIMIT ?2",
            table, table, table
        );

        let rows = self
            .db
            .query(
                &sql,
                &[
                    Value::String(escaped_query),
                    Value::Number(limit.into()),
                ],
            )
            .await?;

        let mut results = Vec::new();
        for row in rows {
            let id = row.get_str("id").unwrap_or_default().to_string();
            let content = row.get_str("content").unwrap_or_default().to_string();
            let score = row.get_f64("score").unwrap_or(0.0) as f32;
            let fields_str = row.get_str("fields").unwrap_or("{}");
            let fields: std::collections::HashMap<String, Value> =
                serde_json::from_str(fields_str).unwrap_or_default();

            results.push(SearchHit {
                id,
                score: -score,
                content,
                highlights: vec![],
                fields,
            });
        }

        Ok(results)
    }

    async fn count(&self, index: &str) -> Result<usize> {
        let table = Self::table_name(index);

        if !self.index_exists(index).await? {
            return Err(StoreError::not_found(format!("Index '{}'", index)));
        }

        let row = self
            .db
            .query_one(&format!("SELECT COUNT(*) as cnt FROM {}", table), &[])
            .await?;

        Ok(row.and_then(|r| r.get_i64("cnt")).unwrap_or(0) as usize)
    }

    fn backend_name(&self) -> &'static str {
        "sqlite-fts5"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fts_index_lifecycle() {
        let store = SqliteFtsStore::memory().await.unwrap();

        assert!(!store.index_exists("articles").await.unwrap());

        store
            .create_index("articles", &IndexSchema::new())
            .await
            .unwrap();
        assert!(store.index_exists("articles").await.unwrap());

        let indexes = store.list_indexes().await.unwrap();
        assert!(indexes.contains(&"articles".to_string()));

        store.delete_index("articles").await.unwrap();
        assert!(!store.index_exists("articles").await.unwrap());
    }

    #[tokio::test]
    async fn test_fts_index_and_search() {
        let store = SqliteFtsStore::memory().await.unwrap();
        store
            .create_index("docs", &IndexSchema::new())
            .await
            .unwrap();

        let docs = vec![
            Document::new("doc1", "The quick brown fox jumps over the lazy dog"),
            Document::new("doc2", "A quick brown dog runs in the park"),
            Document::new("doc3", "The lazy cat sleeps all day"),
        ];

        let count = store.index_documents("docs", &docs).await.unwrap();
        assert_eq!(count, 3);
        assert_eq!(store.count("docs").await.unwrap(), 3);

        let results = store.search("docs", "quick brown", 10).await.unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.id == "doc1"));
        assert!(results.iter().any(|r| r.id == "doc2"));
    }

    #[tokio::test]
    async fn test_fts_delete_documents() {
        let store = SqliteFtsStore::memory().await.unwrap();
        store
            .create_index("test", &IndexSchema::new())
            .await
            .unwrap();

        let docs = vec![
            Document::new("id1", "content one"),
            Document::new("id2", "content two"),
        ];
        store.index_documents("test", &docs).await.unwrap();
        assert_eq!(store.count("test").await.unwrap(), 2);

        let deleted = store
            .delete_documents("test", &["id1".to_string()])
            .await
            .unwrap();
        assert_eq!(deleted, 1);
        assert_eq!(store.count("test").await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_fts_update_document() {
        let store = SqliteFtsStore::memory().await.unwrap();
        store
            .create_index("test", &IndexSchema::new())
            .await
            .unwrap();

        store
            .index_documents("test", &[Document::new("id1", "original content")])
            .await
            .unwrap();

        store
            .index_documents("test", &[Document::new("id1", "updated content")])
            .await
            .unwrap();

        assert_eq!(store.count("test").await.unwrap(), 1);

        let results = store.search("test", "updated", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "id1");
    }

    #[tokio::test]
    async fn test_fts_with_fields() {
        let store = SqliteFtsStore::memory().await.unwrap();
        store
            .create_index("articles", &IndexSchema::new())
            .await
            .unwrap();

        let doc = Document::new("art1", "An interesting article about technology")
            .with_field("author", "Alice")
            .with_field("category", "tech");

        store.index_documents("articles", &[doc]).await.unwrap();

        let results = store.search("articles", "technology", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].fields.get("author"),
            Some(&Value::String("Alice".into()))
        );
    }

    #[tokio::test]
    async fn test_fts_nonexistent_index() {
        let store = SqliteFtsStore::memory().await.unwrap();

        let result = store.search("nonexistent", "query", 10).await;
        assert!(result.is_err());

        let result = store
            .index_documents("nonexistent", &[Document::new("id", "content")])
            .await;
        assert!(result.is_err());
    }
}

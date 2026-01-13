use crate::error::Result;
use crate::traits::{CacheStore, RelationalStore, SearchStore, VectorStore};
use std::sync::Arc;

pub struct StoreRouter {
    relational: Arc<dyn RelationalStore>,
    vector: Arc<dyn VectorStore>,
    cache: Arc<dyn CacheStore>,
    search: Arc<dyn SearchStore>,
}

impl StoreRouter {
    pub fn new(
        relational: Arc<dyn RelationalStore>,
        vector: Arc<dyn VectorStore>,
        cache: Arc<dyn CacheStore>,
        search: Arc<dyn SearchStore>,
    ) -> Self {
        Self {
            relational,
            vector,
            cache,
            search,
        }
    }

    pub fn relational(&self) -> &dyn RelationalStore {
        self.relational.as_ref()
    }

    pub fn vector(&self) -> &dyn VectorStore {
        self.vector.as_ref()
    }

    pub fn cache(&self) -> &dyn CacheStore {
        self.cache.as_ref()
    }

    pub fn search(&self) -> &dyn SearchStore {
        self.search.as_ref()
    }

    pub async fn health_check(&self) -> StoreHealth {
        let relational_ok = self.relational.ping().await.is_ok();

        StoreHealth {
            relational: relational_ok,
            vector: true,
            cache: true,
            search: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StoreHealth {
    pub relational: bool,
    pub vector: bool,
    pub cache: bool,
    pub search: bool,
}

impl StoreHealth {
    pub fn is_healthy(&self) -> bool {
        self.relational && self.vector && self.cache && self.search
    }

    pub fn unhealthy_backends(&self) -> Vec<&'static str> {
        let mut unhealthy = Vec::new();
        if !self.relational {
            unhealthy.push("relational");
        }
        if !self.vector {
            unhealthy.push("vector");
        }
        if !self.cache {
            unhealthy.push("cache");
        }
        if !self.search {
            unhealthy.push("search");
        }
        unhealthy
    }
}

#[cfg(all(feature = "sqlite", feature = "memory-cache"))]
impl StoreRouter {
    pub fn local_default() -> Result<Self> {
        use crate::local::{MemoryCache, MemoryVectorStore, SqliteFtsStore, SqliteStore};

        let sqlite = Arc::new(SqliteStore::memory()?);
        let vector = Arc::new(MemoryVectorStore::new());
        let cache = Arc::new(MemoryCache::new(10000));
        let search = Arc::new(SqliteFtsStore::new(Arc::clone(&sqlite)));

        Ok(Self {
            relational: sqlite,
            vector,
            cache,
            search,
        })
    }

    pub fn local_persistent(data_dir: &str) -> Result<Self> {
        use crate::local::{MemoryCache, MemoryVectorStore, SqliteFtsStore, SqliteStore};

        std::fs::create_dir_all(data_dir)
            .map_err(|e| crate::error::StoreError::connection(e.to_string()))?;

        let db_path = format!("{}/hehe.db", data_dir);
        let sqlite = Arc::new(SqliteStore::open(&db_path)?);
        let vector = Arc::new(MemoryVectorStore::new());
        let cache = Arc::new(MemoryCache::new(10000));
        let search = Arc::new(SqliteFtsStore::new(Arc::clone(&sqlite)));

        Ok(Self {
            relational: sqlite,
            vector,
            cache,
            search,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(all(feature = "sqlite", feature = "memory-cache"))]
    #[tokio::test]
    async fn test_store_router_creation() {
        let router = StoreRouter::local_default().unwrap();

        assert_eq!(router.relational().backend_name(), "sqlite");
        assert_eq!(router.vector().backend_name(), "memory");
        assert_eq!(router.cache().backend_name(), "memory");
        assert_eq!(router.search().backend_name(), "sqlite-fts5");
    }

    #[cfg(all(feature = "sqlite", feature = "memory-cache"))]
    #[tokio::test]
    async fn test_store_router_health_check() {
        let router = StoreRouter::local_default().unwrap();
        let health = router.health_check().await;

        assert!(health.is_healthy());
        assert!(health.unhealthy_backends().is_empty());
    }
}

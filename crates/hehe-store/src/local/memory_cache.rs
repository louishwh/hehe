use crate::error::Result;
use crate::traits::CacheStore;
use async_trait::async_trait;
use moka::future::Cache;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub struct MemoryCache {
    cache: Cache<String, Vec<u8>>,
    counters: Arc<RwLock<HashMap<String, AtomicI64>>>,
}

impl MemoryCache {
    pub fn new(max_capacity: u64) -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(max_capacity)
                .time_to_idle(Duration::from_secs(3600))
                .build(),
            counters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_ttl(max_capacity: u64, default_ttl: Duration) -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(max_capacity)
                .time_to_live(default_ttl)
                .build(),
            counters: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new(10000)
    }
}

#[async_trait]
impl CacheStore for MemoryCache {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.cache.get(key).await)
    }

    async fn set(&self, key: &str, value: &[u8], _ttl: Option<Duration>) -> Result<()> {
        self.cache.insert(key.to_string(), value.to_vec()).await;
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        let existed = self.cache.contains_key(key);
        self.cache.remove(key).await;
        Ok(existed)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        Ok(self.cache.contains_key(key))
    }

    async fn mget(&self, keys: &[&str]) -> Result<Vec<Option<Vec<u8>>>> {
        let mut results = Vec::with_capacity(keys.len());
        for key in keys {
            results.push(self.cache.get(*key).await);
        }
        Ok(results)
    }

    async fn mset(&self, entries: &[(&str, &[u8])], _ttl: Option<Duration>) -> Result<()> {
        for (key, value) in entries {
            self.cache.insert((*key).to_string(), value.to_vec()).await;
        }
        Ok(())
    }

    async fn incr(&self, key: &str, delta: i64) -> Result<i64> {
        let counters = self.counters.read();
        if let Some(counter) = counters.get(key) {
            return Ok(counter.fetch_add(delta, Ordering::SeqCst) + delta);
        }
        drop(counters);

        let mut counters = self.counters.write();
        let counter = counters
            .entry(key.to_string())
            .or_insert_with(|| AtomicI64::new(0));
        Ok(counter.fetch_add(delta, Ordering::SeqCst) + delta)
    }

    async fn expire(&self, _key: &str, _ttl: Duration) -> Result<bool> {
        Ok(true)
    }

    async fn ttl(&self, _key: &str) -> Result<Option<Duration>> {
        Ok(None)
    }

    async fn keys(&self, pattern: &str) -> Result<Vec<String>> {
        let pattern = pattern.replace('*', "");
        let mut result = Vec::new();

        self.cache.run_pending_tasks().await;

        for (key, _) in self.cache.iter() {
            if pattern.is_empty() || key.contains(&pattern) {
                result.push(key.to_string());
            }
        }

        Ok(result)
    }

    async fn clear(&self) -> Result<()> {
        self.cache.invalidate_all();
        self.cache.run_pending_tasks().await;
        self.counters.write().clear();
        Ok(())
    }

    async fn len(&self) -> Result<usize> {
        self.cache.run_pending_tasks().await;
        Ok(self.cache.entry_count() as usize)
    }

    fn backend_name(&self) -> &'static str {
        "memory"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_cache_basic() {
        let cache = MemoryCache::new(100);

        cache.set("key1", b"value1", None).await.unwrap();
        cache.set("key2", b"value2", None).await.unwrap();

        let v1 = cache.get("key1").await.unwrap();
        assert_eq!(v1, Some(b"value1".to_vec()));

        let v2 = cache.get("key2").await.unwrap();
        assert_eq!(v2, Some(b"value2".to_vec()));

        let missing = cache.get("nonexistent").await.unwrap();
        assert!(missing.is_none());
    }

    #[tokio::test]
    async fn test_memory_cache_exists_delete() {
        let cache = MemoryCache::new(100);

        assert!(!cache.exists("key").await.unwrap());

        cache.set("key", b"value", None).await.unwrap();
        assert!(cache.exists("key").await.unwrap());

        let deleted = cache.delete("key").await.unwrap();
        assert!(deleted);
        assert!(!cache.exists("key").await.unwrap());

        let deleted_again = cache.delete("key").await.unwrap();
        assert!(!deleted_again);
    }

    #[tokio::test]
    async fn test_memory_cache_mget_mset() {
        let cache = MemoryCache::new(100);

        cache
            .mset(&[("a", b"1"), ("b", b"2"), ("c", b"3")], None)
            .await
            .unwrap();

        let results = cache.mget(&["a", "b", "nonexistent", "c"]).await.unwrap();
        assert_eq!(results.len(), 4);
        assert_eq!(results[0], Some(b"1".to_vec()));
        assert_eq!(results[1], Some(b"2".to_vec()));
        assert_eq!(results[2], None);
        assert_eq!(results[3], Some(b"3".to_vec()));
    }

    #[tokio::test]
    async fn test_memory_cache_incr() {
        let cache = MemoryCache::new(100);

        let v1 = cache.incr("counter", 1).await.unwrap();
        assert_eq!(v1, 1);

        let v2 = cache.incr("counter", 5).await.unwrap();
        assert_eq!(v2, 6);

        let v3 = cache.decr("counter", 2).await.unwrap();
        assert_eq!(v3, 4);
    }

    #[tokio::test]
    async fn test_memory_cache_clear() {
        let cache = MemoryCache::new(100);

        cache.set("a", b"1", None).await.unwrap();
        cache.set("b", b"2", None).await.unwrap();

        let len_before = cache.len().await.unwrap();
        assert!(len_before > 0);

        cache.clear().await.unwrap();

        let len_after = cache.len().await.unwrap();
        assert_eq!(len_after, 0);
    }

    #[tokio::test]
    async fn test_memory_cache_string_helpers() {
        let cache = MemoryCache::new(100);

        cache.set_string("msg", "hello world", None).await.unwrap();

        let value = cache.get_string("msg").await.unwrap();
        assert_eq!(value, Some("hello world".to_string()));
    }
}

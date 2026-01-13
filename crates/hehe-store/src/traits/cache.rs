use crate::error::Result;
use async_trait::async_trait;
use std::time::Duration;

#[async_trait]
pub trait CacheStore: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;

    async fn get_string(&self, key: &str) -> Result<Option<String>> {
        match self.get(key).await? {
            Some(data) => Ok(Some(
                String::from_utf8(data).map_err(|e| crate::error::StoreError::internal(e.to_string()))?,
            )),
            None => Ok(None),
        }
    }

    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<()>;

    async fn set_string(&self, key: &str, value: &str, ttl: Option<Duration>) -> Result<()> {
        self.set(key, value.as_bytes(), ttl).await
    }

    async fn delete(&self, key: &str) -> Result<bool>;

    async fn exists(&self, key: &str) -> Result<bool>;

    async fn mget(&self, keys: &[&str]) -> Result<Vec<Option<Vec<u8>>>>;

    async fn mset(&self, entries: &[(&str, &[u8])], ttl: Option<Duration>) -> Result<()>;

    async fn incr(&self, key: &str, delta: i64) -> Result<i64>;

    async fn decr(&self, key: &str, delta: i64) -> Result<i64> {
        self.incr(key, -delta).await
    }

    async fn expire(&self, key: &str, ttl: Duration) -> Result<bool>;

    async fn ttl(&self, key: &str) -> Result<Option<Duration>>;

    async fn keys(&self, pattern: &str) -> Result<Vec<String>>;

    async fn clear(&self) -> Result<()>;

    async fn len(&self) -> Result<usize>;

    fn backend_name(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCache;

    #[async_trait]
    impl CacheStore for MockCache {
        async fn get(&self, _key: &str) -> Result<Option<Vec<u8>>> {
            Ok(Some(b"test".to_vec()))
        }
        async fn set(&self, _key: &str, _value: &[u8], _ttl: Option<Duration>) -> Result<()> {
            Ok(())
        }
        async fn delete(&self, _key: &str) -> Result<bool> {
            Ok(true)
        }
        async fn exists(&self, _key: &str) -> Result<bool> {
            Ok(true)
        }
        async fn mget(&self, keys: &[&str]) -> Result<Vec<Option<Vec<u8>>>> {
            Ok(keys.iter().map(|_| Some(b"test".to_vec())).collect())
        }
        async fn mset(&self, _entries: &[(&str, &[u8])], _ttl: Option<Duration>) -> Result<()> {
            Ok(())
        }
        async fn incr(&self, _key: &str, delta: i64) -> Result<i64> {
            Ok(delta)
        }
        async fn expire(&self, _key: &str, _ttl: Duration) -> Result<bool> {
            Ok(true)
        }
        async fn ttl(&self, _key: &str) -> Result<Option<Duration>> {
            Ok(Some(Duration::from_secs(60)))
        }
        async fn keys(&self, _pattern: &str) -> Result<Vec<String>> {
            Ok(vec!["key1".into(), "key2".into()])
        }
        async fn clear(&self) -> Result<()> {
            Ok(())
        }
        async fn len(&self) -> Result<usize> {
            Ok(0)
        }
        fn backend_name(&self) -> &'static str {
            "mock"
        }
    }

    #[tokio::test]
    async fn test_cache_trait() {
        let cache = MockCache;

        let data = cache.get("key").await.unwrap();
        assert_eq!(data, Some(b"test".to_vec()));

        assert!(cache.exists("key").await.unwrap());
        assert!(cache.delete("key").await.unwrap());
    }

    #[tokio::test]
    async fn test_string_helpers() {
        let cache = MockCache;

        let s = cache.get_string("key").await.unwrap();
        assert_eq!(s, Some("test".into()));

        cache.set_string("key", "value", None).await.unwrap();
    }
}

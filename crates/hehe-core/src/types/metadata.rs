use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(flatten)]
    inner: HashMap<String, Value>,
}

impl Metadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
        }
    }

    pub fn insert<K: Into<String>, V: Serialize>(&mut self, key: K, value: V) -> Option<Value> {
        serde_json::to_value(value)
            .ok()
            .and_then(|v| self.inner.insert(key.into(), v))
    }

    pub fn insert_raw(&mut self, key: impl Into<String>, value: Value) -> Option<Value> {
        self.inner.insert(key.into(), value)
    }

    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.inner
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    pub fn get_raw(&self, key: &str) -> Option<&Value> {
        self.inner.get(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.inner.remove(key)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.inner.keys()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.inner.iter()
    }

    pub fn clear(&mut self) {
        self.inner.clear()
    }

    pub fn extend(&mut self, other: Metadata) {
        self.inner.extend(other.inner)
    }
}

impl FromIterator<(String, Value)> for Metadata {
    fn from_iter<T: IntoIterator<Item = (String, Value)>>(iter: T) -> Self {
        Self {
            inner: iter.into_iter().collect(),
        }
    }
}

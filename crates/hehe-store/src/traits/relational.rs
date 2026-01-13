use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Row {
    pub columns: Vec<String>,
    pub values: Vec<Value>,
}

impl Row {
    pub fn new(columns: Vec<String>, values: Vec<Value>) -> Self {
        Self { columns, values }
    }

    pub fn get(&self, column: &str) -> Option<&Value> {
        self.columns
            .iter()
            .position(|c| c == column)
            .and_then(|i| self.values.get(i))
    }

    pub fn get_str(&self, column: &str) -> Option<&str> {
        self.get(column).and_then(|v| v.as_str())
    }

    pub fn get_i64(&self, column: &str) -> Option<i64> {
        self.get(column).and_then(|v| v.as_i64())
    }

    pub fn get_f64(&self, column: &str) -> Option<f64> {
        self.get(column).and_then(|v| v.as_f64())
    }

    pub fn get_bool(&self, column: &str) -> Option<bool> {
        self.get(column).and_then(|v| v.as_bool())
    }

    pub fn to_map(&self) -> HashMap<String, Value> {
        self.columns
            .iter()
            .zip(self.values.iter())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct Migration {
    pub version: u32,
    pub name: String,
    pub up: String,
    pub down: Option<String>,
}

impl Migration {
    pub fn new(version: u32, name: impl Into<String>, up: impl Into<String>) -> Self {
        Self {
            version,
            name: name.into(),
            up: up.into(),
            down: None,
        }
    }

    pub fn with_down(mut self, down: impl Into<String>) -> Self {
        self.down = Some(down.into());
        self
    }
}

#[async_trait]
pub trait Transaction: Send {
    async fn execute(&mut self, sql: &str, params: &[Value]) -> Result<u64>;
    async fn query(&mut self, sql: &str, params: &[Value]) -> Result<Vec<Row>>;
    async fn query_one(&mut self, sql: &str, params: &[Value]) -> Result<Option<Row>>;
    async fn commit(self: Box<Self>) -> Result<()>;
    async fn rollback(self: Box<Self>) -> Result<()>;
}

#[async_trait]
pub trait RelationalStore: Send + Sync {
    async fn execute(&self, sql: &str, params: &[Value]) -> Result<u64>;

    async fn query(&self, sql: &str, params: &[Value]) -> Result<Vec<Row>>;

    async fn query_one(&self, sql: &str, params: &[Value]) -> Result<Option<Row>>;

    async fn begin(&self) -> Result<Box<dyn Transaction>>;

    async fn migrate(&self, migrations: &[Migration]) -> Result<()>;

    async fn ping(&self) -> Result<()>;

    async fn execute_batch(&self, statements: &[&str]) -> Result<()> {
        for stmt in statements {
            self.execute(stmt, &[]).await?;
        }
        Ok(())
    }

    async fn table_exists(&self, table: &str) -> Result<bool>;

    fn backend_name(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_access() {
        let row = Row::new(
            vec!["id".into(), "name".into(), "age".into()],
            vec![
                Value::Number(1.into()),
                Value::String("Alice".into()),
                Value::Number(30.into()),
            ],
        );

        assert_eq!(row.get_i64("id"), Some(1));
        assert_eq!(row.get_str("name"), Some("Alice"));
        assert_eq!(row.get_i64("age"), Some(30));
        assert!(row.get("unknown").is_none());
    }

    #[test]
    fn test_row_to_map() {
        let row = Row::new(
            vec!["a".into(), "b".into()],
            vec![Value::String("x".into()), Value::String("y".into())],
        );

        let map = row.to_map();
        assert_eq!(map.get("a"), Some(&Value::String("x".into())));
        assert_eq!(map.get("b"), Some(&Value::String("y".into())));
    }

    #[test]
    fn test_migration() {
        let m = Migration::new(1, "create_users", "CREATE TABLE users (id INTEGER PRIMARY KEY)")
            .with_down("DROP TABLE users");

        assert_eq!(m.version, 1);
        assert_eq!(m.name, "create_users");
        assert!(m.down.is_some());
    }
}

use crate::error::{Result, StoreError};
use crate::traits::{Migration, RelationalStore, Row, Transaction};
use async_trait::async_trait;
use parking_lot::Mutex;
use rusqlite::{params_from_iter, Connection, ToSql};
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;

pub struct SqliteStore {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA foreign_keys=ON;
             PRAGMA busy_timeout=5000;",
        )?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(
            "PRAGMA foreign_keys=ON;",
        )?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn value_to_sql(value: &Value) -> Box<dyn ToSql> {
        match value {
            Value::Null => Box::new(rusqlite::types::Null),
            Value::Bool(b) => Box::new(*b),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Box::new(i)
                } else if let Some(f) = n.as_f64() {
                    Box::new(f)
                } else {
                    Box::new(n.to_string())
                }
            }
            Value::String(s) => Box::new(s.clone()),
            Value::Array(_) | Value::Object(_) => Box::new(value.to_string()),
        }
    }

    fn row_to_values(row: &rusqlite::Row, columns: &[String]) -> Result<Vec<Value>> {
        let mut values = Vec::with_capacity(columns.len());
        for i in 0..columns.len() {
            let value: rusqlite::types::Value = row.get(i)?;
            let json_value = match value {
                rusqlite::types::Value::Null => Value::Null,
                rusqlite::types::Value::Integer(i) => Value::Number(i.into()),
                rusqlite::types::Value::Real(f) => {
                    Value::Number(serde_json::Number::from_f64(f).unwrap_or(0.into()))
                }
                rusqlite::types::Value::Text(s) => Value::String(s),
                rusqlite::types::Value::Blob(b) => {
                    Value::String(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &b))
                }
            };
            values.push(json_value);
        }
        Ok(values)
    }
}

#[async_trait]
impl RelationalStore for SqliteStore {
    async fn execute(&self, sql: &str, params: &[Value]) -> Result<u64> {
        let conn = self.conn.lock();
        let sql_params: Vec<Box<dyn ToSql>> = params.iter().map(Self::value_to_sql).collect();
        let param_refs: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        let affected = conn.execute(sql, params_from_iter(param_refs))?;
        Ok(affected as u64)
    }

    async fn query(&self, sql: &str, params: &[Value]) -> Result<Vec<Row>> {
        let conn = self.conn.lock();
        let sql_params: Vec<Box<dyn ToSql>> = params.iter().map(Self::value_to_sql).collect();
        let param_refs: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(sql)?;
        let columns: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

        let rows = stmt.query_map(params_from_iter(param_refs), |row| {
            Ok(Self::row_to_values(row, &columns))
        })?;

        let mut result = Vec::new();
        for row in rows {
            let values = row?.map_err(|e| StoreError::Query(e.to_string()))?;
            result.push(Row::new(columns.clone(), values));
        }
        Ok(result)
    }

    async fn query_one(&self, sql: &str, params: &[Value]) -> Result<Option<Row>> {
        let rows = self.query(sql, params).await?;
        Ok(rows.into_iter().next())
    }

    async fn begin(&self) -> Result<Box<dyn Transaction>> {
        Err(StoreError::internal(
            "SQLite transactions not yet implemented in async context",
        ))
    }

    async fn migrate(&self, migrations: &[Migration]) -> Result<()> {
        let conn = self.conn.lock();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS _migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        )?;

        let applied: std::collections::HashSet<u32> = {
            let mut stmt = conn.prepare("SELECT version FROM _migrations")?;
            let rows = stmt.query_map([], |row| row.get(0))?;
            rows.filter_map(|r| r.ok()).collect()
        };

        for migration in migrations {
            if applied.contains(&migration.version) {
                continue;
            }

            tracing::info!(
                version = migration.version,
                name = %migration.name,
                "Applying migration"
            );

            conn.execute_batch(&migration.up)?;

            conn.execute(
                "INSERT INTO _migrations (version, name) VALUES (?1, ?2)",
                rusqlite::params![migration.version, migration.name],
            )?;
        }

        Ok(())
    }

    async fn ping(&self) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute_batch("SELECT 1")?;
        Ok(())
    }

    async fn table_exists(&self, table: &str) -> Result<bool> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
        )?;
        let count: i64 = stmt.query_row([table], |row| row.get(0))?;
        Ok(count > 0)
    }

    fn backend_name(&self) -> &'static str {
        "sqlite"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sqlite_basic_operations() {
        let store = SqliteStore::memory().unwrap();

        store
            .execute(
                "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT, value REAL)",
                &[],
            )
            .await
            .unwrap();

        let affected = store
            .execute(
                "INSERT INTO test (name, value) VALUES (?1, ?2)",
                &[Value::String("alice".into()), serde_json::json!(1.5)],
            )
            .await
            .unwrap();
        assert_eq!(affected, 1);

        store
            .execute(
                "INSERT INTO test (name, value) VALUES (?1, ?2)",
                &[Value::String("bob".into()), serde_json::json!(2.5)],
            )
            .await
            .unwrap();

        let rows = store.query("SELECT * FROM test ORDER BY id", &[]).await.unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].get_str("name"), Some("alice"));
        assert_eq!(rows[1].get_str("name"), Some("bob"));
    }

    #[tokio::test]
    async fn test_sqlite_query_one() {
        let store = SqliteStore::memory().unwrap();

        store
            .execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)", &[])
            .await
            .unwrap();

        store
            .execute(
                "INSERT INTO users (name) VALUES (?1)",
                &[Value::String("test".into())],
            )
            .await
            .unwrap();

        let row = store
            .query_one("SELECT * FROM users WHERE name = ?1", &[Value::String("test".into())])
            .await
            .unwrap();
        assert!(row.is_some());
        assert_eq!(row.unwrap().get_str("name"), Some("test"));

        let empty = store
            .query_one(
                "SELECT * FROM users WHERE name = ?1",
                &[Value::String("nonexistent".into())],
            )
            .await
            .unwrap();
        assert!(empty.is_none());
    }

    #[tokio::test]
    async fn test_sqlite_migration() {
        let store = SqliteStore::memory().unwrap();

        let migrations = vec![
            Migration::new(1, "create_users", "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)"),
            Migration::new(2, "add_email", "ALTER TABLE users ADD COLUMN email TEXT"),
        ];

        store.migrate(&migrations).await.unwrap();

        assert!(store.table_exists("users").await.unwrap());
        assert!(store.table_exists("_migrations").await.unwrap());

        store.migrate(&migrations).await.unwrap();

        let rows = store.query("SELECT * FROM _migrations", &[]).await.unwrap();
        assert_eq!(rows.len(), 2);
    }

    #[tokio::test]
    async fn test_sqlite_ping() {
        let store = SqliteStore::memory().unwrap();
        assert!(store.ping().await.is_ok());
    }

    #[tokio::test]
    async fn test_sqlite_table_exists() {
        let store = SqliteStore::memory().unwrap();

        assert!(!store.table_exists("nonexistent").await.unwrap());

        store.execute("CREATE TABLE exists_test (id INTEGER)", &[]).await.unwrap();

        assert!(store.table_exists("exists_test").await.unwrap());
    }

    #[tokio::test]
    async fn test_sqlite_null_handling() {
        let store = SqliteStore::memory().unwrap();

        store
            .execute("CREATE TABLE nullable (id INTEGER, data TEXT)", &[])
            .await
            .unwrap();

        store
            .execute(
                "INSERT INTO nullable (id, data) VALUES (?1, ?2)",
                &[Value::Number(1.into()), Value::Null],
            )
            .await
            .unwrap();

        let rows = store.query("SELECT * FROM nullable", &[]).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get("data"), Some(&Value::Null));
    }
}

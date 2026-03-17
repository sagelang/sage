//! Sync adapters for async persistence backends.
//!
//! These wrappers bridge the async `sage_persistence` stores to the sync
//! `CheckpointStore` trait used in generated code. They use `block_on()`
//! internally, which is safe because agent code runs on the tokio runtime.

use crate::persistence::CheckpointStore;
use serde_json::Value;
use std::collections::HashMap;

// Import the async trait to access its methods
use sage_persistence::CheckpointStore as AsyncCheckpointStore;

/// Sync wrapper for SQLite checkpoint store.
#[cfg(feature = "persistence-sqlite")]
pub struct SyncSqliteStore {
    inner: sage_persistence::SqliteStore,
}

#[cfg(feature = "persistence-sqlite")]
impl SyncSqliteStore {
    /// Open or create a SQLite checkpoint database.
    ///
    /// # Errors
    /// Returns an error if the database cannot be opened or created.
    pub fn open(path: &str) -> Result<Self, sage_persistence::PersistenceError> {
        let inner = sage_persistence::SqliteStore::open(path)?;
        Ok(Self { inner })
    }

    /// Create an in-memory SQLite store for testing.
    pub fn in_memory() -> Result<Self, sage_persistence::PersistenceError> {
        let inner = sage_persistence::SqliteStore::in_memory()?;
        Ok(Self { inner })
    }
}

#[cfg(feature = "persistence-sqlite")]
impl CheckpointStore for SyncSqliteStore {
    fn save_sync(&self, agent_key: &str, field: &str, value: Value) {
        // Use tokio's block_in_place to allow blocking in async context
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                if let Err(e) = self.inner.save(agent_key, field, value).await {
                    eprintln!("Persistence error (save): {e}");
                }
            })
        });
    }

    fn load_sync(&self, agent_key: &str, field: &str) -> Option<Value> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                match self.inner.load(agent_key, field).await {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Persistence error (load): {e}");
                        None
                    }
                }
            })
        })
    }

    fn load_all_sync(&self, agent_key: &str) -> HashMap<String, Value> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                match self.inner.load_all(agent_key).await {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("Persistence error (load_all): {e}");
                        HashMap::new()
                    }
                }
            })
        })
    }

    fn save_all_sync(&self, agent_key: &str, fields: &HashMap<String, Value>) {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                if let Err(e) = self.inner.save_all(agent_key, fields).await {
                    eprintln!("Persistence error (save_all): {e}");
                }
            })
        });
    }

    fn exists_sync(&self, agent_key: &str) -> bool {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                match self.inner.exists(agent_key).await {
                    Ok(b) => b,
                    Err(e) => {
                        eprintln!("Persistence error (exists): {e}");
                        false
                    }
                }
            })
        })
    }
}

/// Sync wrapper for PostgreSQL checkpoint store.
#[cfg(feature = "persistence-postgres")]
pub struct SyncPostgresStore {
    inner: sage_persistence::PostgresStore,
}

#[cfg(feature = "persistence-postgres")]
impl SyncPostgresStore {
    /// Connect to a PostgreSQL database.
    ///
    /// The connection string should be a standard PostgreSQL URL:
    /// `postgres://user:password@host/database`
    ///
    /// # Errors
    /// Returns an error if the connection cannot be established.
    pub fn connect(url: &str) -> Result<Self, sage_persistence::PersistenceError> {
        // We need to run connect in a runtime since it's async
        let inner = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(sage_persistence::PostgresStore::connect(url))
        })?;
        Ok(Self { inner })
    }
}

#[cfg(feature = "persistence-postgres")]
impl CheckpointStore for SyncPostgresStore {
    fn save_sync(&self, agent_key: &str, field: &str, value: Value) {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                if let Err(e) = self.inner.save(agent_key, field, value).await {
                    eprintln!("Persistence error (save): {e}");
                }
            })
        });
    }

    fn load_sync(&self, agent_key: &str, field: &str) -> Option<Value> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                match self.inner.load(agent_key, field).await {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Persistence error (load): {e}");
                        None
                    }
                }
            })
        })
    }

    fn load_all_sync(&self, agent_key: &str) -> HashMap<String, Value> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                match self.inner.load_all(agent_key).await {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("Persistence error (load_all): {e}");
                        HashMap::new()
                    }
                }
            })
        })
    }

    fn save_all_sync(&self, agent_key: &str, fields: &HashMap<String, Value>) {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                if let Err(e) = self.inner.save_all(agent_key, fields).await {
                    eprintln!("Persistence error (save_all): {e}");
                }
            })
        });
    }

    fn exists_sync(&self, agent_key: &str) -> bool {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                match self.inner.exists(agent_key).await {
                    Ok(b) => b,
                    Err(e) => {
                        eprintln!("Persistence error (exists): {e}");
                        false
                    }
                }
            })
        })
    }
}

/// Sync wrapper for file-based checkpoint store.
#[cfg(feature = "persistence-file")]
pub struct SyncFileStore {
    inner: sage_persistence::FileStore,
}

#[cfg(feature = "persistence-file")]
impl SyncFileStore {
    /// Create a new file store in the given directory.
    ///
    /// # Errors
    /// Returns an error if the directory cannot be created.
    pub fn open(path: &str) -> Result<Self, sage_persistence::PersistenceError> {
        let inner = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(sage_persistence::FileStore::open(path))
        })?;
        Ok(Self { inner })
    }
}

#[cfg(feature = "persistence-file")]
impl CheckpointStore for SyncFileStore {
    fn save_sync(&self, agent_key: &str, field: &str, value: Value) {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                if let Err(e) = self.inner.save(agent_key, field, value).await {
                    eprintln!("Persistence error (save): {e}");
                }
            })
        });
    }

    fn load_sync(&self, agent_key: &str, field: &str) -> Option<Value> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                match self.inner.load(agent_key, field).await {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Persistence error (load): {e}");
                        None
                    }
                }
            })
        })
    }

    fn load_all_sync(&self, agent_key: &str) -> HashMap<String, Value> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                match self.inner.load_all(agent_key).await {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("Persistence error (load_all): {e}");
                        HashMap::new()
                    }
                }
            })
        })
    }

    fn save_all_sync(&self, agent_key: &str, fields: &HashMap<String, Value>) {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                if let Err(e) = self.inner.save_all(agent_key, fields).await {
                    eprintln!("Persistence error (save_all): {e}");
                }
            })
        });
    }

    fn exists_sync(&self, agent_key: &str) -> bool {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                match self.inner.exists(agent_key).await {
                    Ok(b) => b,
                    Err(e) => {
                        eprintln!("Persistence error (exists): {e}");
                        false
                    }
                }
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "persistence-sqlite")]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn sqlite_sync_adapter_save_load() {
        let store = SyncSqliteStore::in_memory().unwrap();
        store.save_sync("agent1", "count", serde_json::json!(42));
        let loaded = store.load_sync("agent1", "count");
        assert_eq!(loaded, Some(serde_json::json!(42)));
    }

    #[cfg(feature = "persistence-sqlite")]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn sqlite_sync_adapter_load_missing() {
        let store = SyncSqliteStore::in_memory().unwrap();
        let loaded = store.load_sync("agent1", "nonexistent");
        assert_eq!(loaded, None);
    }

    #[cfg(feature = "persistence-sqlite")]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn sqlite_sync_adapter_exists() {
        let store = SyncSqliteStore::in_memory().unwrap();
        assert!(!store.exists_sync("agent1"));
        store.save_sync("agent1", "field", serde_json::json!("value"));
        assert!(store.exists_sync("agent1"));
    }

    #[cfg(feature = "persistence-sqlite")]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn sqlite_sync_adapter_save_all_load_all() {
        let store = SyncSqliteStore::in_memory().unwrap();
        let mut fields = HashMap::new();
        fields.insert("a".to_string(), serde_json::json!(1));
        fields.insert("b".to_string(), serde_json::json!(2));

        store.save_all_sync("agent1", &fields);

        let loaded = store.load_all_sync("agent1");
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded.get("a"), Some(&serde_json::json!(1)));
        assert_eq!(loaded.get("b"), Some(&serde_json::json!(2)));
    }
}

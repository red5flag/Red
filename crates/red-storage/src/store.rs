//! Storage — the main sled-backed storage service for Red.
//!
//! Provides typed access to sled trees with proper key encoding,
//! batched writes, and cursor-based pagination.

use crate::keys::{encode_uuid, MessageKey, TreeName};
use crate::migration::{Migration, MigrationRunner, SchemaVersion};
use red_core::{CoreError, CoreResult};
use sled::{Batch, Db, Tree};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

/// The main storage service backed by sled.
pub struct Storage {
    db: Db,
    db_path: PathBuf,
}

/// Current schema version — increment when adding new migrations.
const CURRENT_SCHEMA_VERSION: SchemaVersion = 1;

/// All forward migrations from version 0 to CURRENT_SCHEMA_VERSION.
fn migrations() -> Vec<Migration> {
    vec![
        // v0 -> v1: Initial structured tree layout
        Migration {
            from: 0,
            to: 1,
            name: "initial_tree_layout",
            run: |db| {
                // Ensure all trees exist by opening them
                for tree in [
                    TreeName::Portfolios,
                    TreeName::Assets,
                    TreeName::AssetsByGroup,
                    TreeName::AssetsByStatus,
                    TreeName::Channels,
                    TreeName::Messages,
                    TreeName::MessagesByChannel,
                    TreeName::Transactions,
                    TreeName::TransactionsByStatus,
                    TreeName::Documents,
                    TreeName::DocumentsByAsset,
                    TreeName::Notifications,
                    TreeName::SyncState,
                    TreeName::PendingOps,
                    TreeName::Credentials,
                ] {
                    db.open_tree(tree.as_str())
                        .map_err(|e| CoreError::Migration {
                            message: e.to_string(),
                        })?;
                }
                Ok(())
            },
        },
    ]
}

impl Storage {
    /// Open or create a storage instance at the given path.
    /// Runs any pending migrations automatically.
    pub fn open(path: impl Into<PathBuf>) -> CoreResult<Arc<Self>> {
        let path = path.into();
        std::fs::create_dir_all(&path).map_err(|e| CoreError::Storage {
            message: e.to_string(),
        })?;

        let db = sled::Config::default()
            .path(&path)
            .cache_capacity(64 * 1024 * 1024) // 64MB cache
            .flush_every_ms(Some(1000)) // Flush at most every 1s
            .open()
            .map_err(|e| CoreError::Storage {
                message: e.to_string(),
            })?;

        let storage = Self {
            db: db.clone(),
            db_path: path.clone(),
        };

        // Run migrations
        let runner = MigrationRunner::new(db, path);
        let version = runner.run(&migrations())?;
        info!("Storage opened at version {}", version);

        Ok(Arc::new(storage))
    }

    /// Get a sled tree by name.
    pub fn tree(&self, name: TreeName) -> CoreResult<Tree> {
        self.db
            .open_tree(name.as_str())
            .map_err(|e| CoreError::Storage {
                message: e.to_string(),
            })
    }

    /// Get the raw sled database (for advanced operations).
    pub fn raw_db(&self) -> &Db {
        &self.db
    }

    // ─────────────────────────────────────────────────────
    // Generic key-value operations
    // ─────────────────────────────────────────────────────

    /// Insert a key-value pair into a specific tree.
    pub fn insert(&self, tree: TreeName, key: &[u8], value: &[u8]) -> CoreResult<()> {
        self.tree(tree)?
            .insert(key, value)
            .map_err(|e| CoreError::Storage {
                message: e.to_string(),
            })?;
        Ok(())
    }

    /// Get a value by key from a specific tree.
    pub fn get(&self, tree: TreeName, key: &[u8]) -> CoreResult<Option<sled::IVec>> {
        self.tree(tree)?
            .get(key)
            .map_err(|e| CoreError::Storage {
                message: e.to_string(),
            })
    }

    /// Delete a key from a specific tree.
    pub fn remove(&self, tree: TreeName, key: &[u8]) -> CoreResult<()> {
        self.tree(tree)?
            .remove(key)
            .map_err(|e| CoreError::Storage {
                message: e.to_string(),
            })?;
        Ok(())
    }

    /// Apply a batch of operations atomically.
    pub fn apply_batch(&self, tree: TreeName, batch: Batch) -> CoreResult<()> {
        self.tree(tree)?
            .apply_batch(batch)
            .map_err(|e| CoreError::Storage {
                message: e.to_string(),
            })?;
        Ok(())
    }

    // ─────────────────────────────────────────────────────
    // Message operations (ordered compound keys)
    // ─────────────────────────────────────────────────────

    /// Insert a message with an ordered compound key.
    /// The message is stored in both the `messages` tree (ordered by channel+time)
    /// and the `messages_by_channel` tree (for direct lookup).
    pub fn insert_message(
        &self,
        key: &MessageKey,
        value: &[u8],
    ) -> CoreResult<()> {
        let encoded_key = key.encode();

        // Primary tree: ordered by channel_id | timestamp | message_id
        self.insert(TreeName::Messages, &encoded_key, value)?;

        // Secondary tree: for direct message lookup by channel + message_id
        let secondary_key =
            crate::keys::encode_compound_uuid(key.channel_id, key.message_id);
        self.insert(TreeName::MessagesByChannel, &secondary_key, value)?;

        Ok(())
    }

    /// Get the latest N messages for a channel (reverse range scan).
    pub fn get_latest_messages(
        &self,
        channel_id: uuid::Uuid,
        limit: usize,
    ) -> CoreResult<Vec<(MessageKey, Vec<u8>)>> {
        let prefix = MessageKey::channel_prefix(channel_id);
        let upper = MessageKey::channel_max_bound(channel_id);

        let tree = self.tree(TreeName::Messages)?;
        let mut results = Vec::with_capacity(limit);

        // Reverse scan from the upper bound backwards
        for item in tree.range::<&[u8], _>(..=&upper[..]).rev().take(limit) {
            let (k, v) = item.map_err(|e| CoreError::Storage {
                message: e.to_string(),
            })?;

            // Stop if we've left the channel prefix
            if !k.starts_with(&prefix) {
                break;
            }

            if let Some(mk) = MessageKey::decode(&k) {
                results.push((mk, v.to_vec()));
            }
        }

        Ok(results)
    }

    /// Get messages before a given timestamp (exclusive, for backward pagination).
    pub fn get_messages_before(
        &self,
        channel_id: uuid::Uuid,
        before: chrono::DateTime<chrono::Utc>,
        limit: usize,
    ) -> CoreResult<Vec<(MessageKey, Vec<u8>)>> {
        let prefix = MessageKey::channel_prefix(channel_id);
        // Use timestamp - 1ms for exclusive upper bound
        let upper_ts = before - chrono::Duration::milliseconds(1);
        let upper = MessageKey::channel_upper_bound(channel_id, upper_ts);

        let tree = self.tree(TreeName::Messages)?;
        let mut results = Vec::with_capacity(limit);

        for item in tree.range::<&[u8], _>(..=&upper[..]).rev().take(limit) {
            let (k, v) = item.map_err(|e| CoreError::Storage {
                message: e.to_string(),
            })?;

            if !k.starts_with(&prefix) {
                break;
            }

            if let Some(mk) = MessageKey::decode(&k) {
                results.push((mk, v.to_vec()));
            }
        }

        Ok(results)
    }

    // ─────────────────────────────────────────────────────
    // Portfolio operations
    // ─────────────────────────────────────────────────────

    /// Save a portfolio by UUID key.
    pub fn save_portfolio(&self, id: uuid::Uuid, value: &[u8]) -> CoreResult<()> {
        self.insert(TreeName::Portfolios, &encode_uuid(id), value)
    }

    /// Load a portfolio by UUID key.
    pub fn load_portfolio(&self, id: uuid::Uuid) -> CoreResult<Option<Vec<u8>>> {
        Ok(self.get(TreeName::Portfolios, &encode_uuid(id))?.map(|v| v.to_vec()))
    }

    /// Load all portfolios (full scan — use sparingly).
    pub fn load_all_portfolios(&self) -> CoreResult<Vec<(uuid::Uuid, Vec<u8>)>> {
        let tree = self.tree(TreeName::Portfolios)?;
        let mut results = Vec::new();
        for item in tree.iter() {
            let (k, v) = item.map_err(|e| CoreError::Storage {
                message: e.to_string(),
            })?;
            if k.len() == 16 {
                if let Ok(id) = uuid::Uuid::from_slice(&k) {
                    results.push((id, v.to_vec()));
                }
            }
        }
        Ok(results)
    }

    /// Delete a portfolio by UUID key.
    pub fn delete_portfolio(&self, id: uuid::Uuid) -> CoreResult<()> {
        self.remove(TreeName::Portfolios, &encode_uuid(id))
    }

    // ─────────────────────────────────────────────────────
    // Transaction operations
    // ─────────────────────────────────────────────────────

    /// Save a transaction by UUID key.
    pub fn save_transaction(&self, id: uuid::Uuid, value: &[u8]) -> CoreResult<()> {
        self.insert(TreeName::Transactions, &encode_uuid(id), value)
    }

    /// Load a transaction by UUID key.
    pub fn load_transaction(&self, id: uuid::Uuid) -> CoreResult<Option<Vec<u8>>> {
        Ok(self
            .get(TreeName::Transactions, &encode_uuid(id))?
            .map(|v| v.to_vec()))
    }

    /// Index a transaction by status for filtered queries.
    pub fn index_transaction_by_status(
        &self,
        status: &str,
        id: uuid::Uuid,
    ) -> CoreResult<()> {
        self.insert(
            TreeName::TransactionsByStatus,
            &crate::keys::encode_str_uuid(status, id),
            &[],
        )
    }

    /// Get all transaction IDs for a given status.
    pub fn get_transactions_by_status(
        &self,
        status: &str,
    ) -> CoreResult<Vec<uuid::Uuid>> {
        let tree = self.tree(TreeName::TransactionsByStatus)?;
        let prefix = status.as_bytes();
        let mut results = Vec::new();

        for item in tree.scan_prefix(prefix) {
            let (k, _) = item.map_err(|e| CoreError::Storage {
                message: e.to_string(),
            })?;
            // Key is `status | uuid` — extract the UUID (last 16 bytes)
            if k.len() >= 16 {
                let uuid_bytes = &k[k.len() - 16..];
                if let Ok(id) = uuid::Uuid::from_slice(uuid_bytes) {
                    results.push(id);
                }
            }
        }
        Ok(results)
    }

    // ─────────────────────────────────────────────────────
    // Sync state
    // ─────────────────────────────────────────────────────

    /// Get or set sync state for an entity type.
    pub fn set_sync_state(&self, entity: &str, value: &[u8]) -> CoreResult<()> {
        self.insert(TreeName::SyncState, entity.as_bytes(), value)
    }

    pub fn get_sync_state(&self, entity: &str) -> CoreResult<Option<Vec<u8>>> {
        Ok(self
            .get(TreeName::SyncState, entity.as_bytes())?
            .map(|v| v.to_vec()))
    }

    // ─────────────────────────────────────────────────────
    // Pending operations queue
    // ─────────────────────────────────────────────────────

    /// Enqueue a pending operation with a timestamp-ordered key.
    pub fn enqueue_pending_op(
        &self,
        timestamp: chrono::DateTime<chrono::Utc>,
        id: uuid::Uuid,
        value: &[u8],
    ) -> CoreResult<()> {
        let mut key = Vec::with_capacity(24);
        key.extend_from_slice(&crate::keys::encode_timestamp(timestamp));
        key.extend_from_slice(&encode_uuid(id));
        self.insert(TreeName::PendingOps, &key, value)
    }

    /// Get the oldest pending operations (FIFO order by timestamp).
    pub fn get_pending_ops(&self, limit: usize) -> CoreResult<Vec<(Vec<u8>, Vec<u8>)>> {
        let tree = self.tree(TreeName::PendingOps)?;
        let mut results = Vec::with_capacity(limit);
        for item in tree.iter().take(limit) {
            let (k, v) = item.map_err(|e| CoreError::Storage {
                message: e.to_string(),
            })?;
            results.push((k.to_vec(), v.to_vec()));
        }
        Ok(results)
    }

    /// Remove a pending operation after it's been processed.
    pub fn dequeue_pending_op(&self, key: &[u8]) -> CoreResult<()> {
        self.remove(TreeName::PendingOps, key)
    }

    // ─────────────────────────────────────────────────────
    // Maintenance
    // ─────────────────────────────────────────────────────

    /// Flush all pending writes to disk.
    pub fn flush(&self) -> CoreResult<()> {
        self.db
            .flush()
            .map_err(|e| CoreError::Storage {
                message: e.to_string(),
            })?;
        Ok(())
    }

    /// Get the approximate size of the database in bytes.
    pub fn size_on_disk(&self) -> CoreResult<u64> {
        Ok(self.db.size_on_disk().unwrap_or(0))
    }

    /// Get the current schema version.
    pub fn schema_version(&self) -> SchemaVersion {
        let runner = MigrationRunner::new(self.db.clone(), self.db_path.clone());
        runner.current_version()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use uuid::Uuid;
    use chrono::Utc;

    fn open_test_storage() -> (Arc<Storage>, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let storage = Storage::open(dir.path()).unwrap();
        (storage, dir)
    }

    #[test]
    fn test_open_and_migrate() {
        let (storage, _dir) = open_test_storage();
        assert_eq!(storage.schema_version(), CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn test_portfolio_save_load() {
        let (storage, _dir) = open_test_storage();
        let id = Uuid::new_v4();
        let data = b"test portfolio data";

        storage.save_portfolio(id, data).unwrap();
        let loaded = storage.load_portfolio(id).unwrap();
        assert_eq!(loaded, Some(data.to_vec()));
    }

    #[test]
    fn test_portfolio_delete() {
        let (storage, _dir) = open_test_storage();
        let id = Uuid::new_v4();
        let data = b"test";

        storage.save_portfolio(id, data).unwrap();
        storage.delete_portfolio(id).unwrap();
        assert_eq!(storage.load_portfolio(id).unwrap(), None);
    }

    #[test]
    fn test_message_insert_and_latest() {
        let (storage, _dir) = open_test_storage();
        let channel_id = Uuid::new_v4();

        // Insert 3 messages with different timestamps
        for i in 0..3 {
            let ts = Utc::now() + chrono::Duration::seconds(i);
            let msg_id = Uuid::new_v4();
            let key = MessageKey::new(channel_id, ts, msg_id);
            let data = format!("message {}", i);
            storage.insert_message(&key, data.as_bytes()).unwrap();
        }

        // Get latest 2 — should be messages 2 and 1 (reverse order)
        let results = storage.get_latest_messages(channel_id, 2).unwrap();
        assert_eq!(results.len(), 2);
        // Latest first
        assert_eq!(results[0].1, b"message 2".to_vec());
        assert_eq!(results[1].1, b"message 1".to_vec());
    }

    #[test]
    fn test_messages_before_timestamp() {
        let (storage, _dir) = open_test_storage();
        let channel_id = Uuid::new_v4();
        let base_ts = Utc::now();

        // Insert messages at t, t+1, t+2
        for i in 0..3 {
            let ts = base_ts + chrono::Duration::seconds(i);
            let msg_id = Uuid::new_v4();
            let key = MessageKey::new(channel_id, ts, msg_id);
            storage.insert_message(&key, format!("msg {}", i).as_bytes()).unwrap();
        }

        // Get messages before t+2 (should get msg 1 and msg 0)
        let before = base_ts + chrono::Duration::seconds(2);
        let results = storage.get_messages_before(channel_id, before, 10).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].1, b"msg 1".to_vec());
        assert_eq!(results[1].1, b"msg 0".to_vec());
    }

    #[test]
    fn test_transaction_status_index() {
        let (storage, _dir) = open_test_storage();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();

        storage.index_transaction_by_status("pending", id1).unwrap();
        storage.index_transaction_by_status("pending", id2).unwrap();
        storage.index_transaction_by_status("approved", id3).unwrap();

        let pending = storage.get_transactions_by_status("pending").unwrap();
        assert_eq!(pending.len(), 2);
        assert!(pending.contains(&id1));
        assert!(pending.contains(&id2));

        let approved = storage.get_transactions_by_status("approved").unwrap();
        assert_eq!(approved.len(), 1);
        assert!(approved.contains(&id3));
    }

    #[test]
    fn test_pending_ops_fifo() {
        let (storage, _dir) = open_test_storage();
        let base_ts = Utc::now();

        for i in 0..3 {
            let ts = base_ts + chrono::Duration::seconds(i);
            let id = Uuid::new_v4();
            storage.enqueue_pending_op(ts, id, format!("op {}", i).as_bytes()).unwrap();
        }

        let ops = storage.get_pending_ops(10).unwrap();
        assert_eq!(ops.len(), 3);
        // FIFO order — earliest first
        assert_eq!(ops[0].1, b"op 0".to_vec());
        assert_eq!(ops[1].1, b"op 1".to_vec());
        assert_eq!(ops[2].1, b"op 2".to_vec());
    }

    #[test]
    fn test_sync_state() {
        let (storage, _dir) = open_test_storage();
        storage.set_sync_state("portfolios", b"synced at noon").unwrap();
        let state = storage.get_sync_state("portfolios").unwrap();
        assert_eq!(state, Some(b"synced at noon".to_vec()));
    }

    #[test]
    fn test_load_all_portfolios() {
        let (storage, _dir) = open_test_storage();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        storage.save_portfolio(id1, b"data1").unwrap();
        storage.save_portfolio(id2, b"data2").unwrap();

        let all = storage.load_all_portfolios().unwrap();
        assert_eq!(all.len(), 2);
    }
}

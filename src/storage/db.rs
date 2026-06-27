use crate::models::{Message, Portfolio};
use crate::utils::crypto;
use dashmap::DashMap;
use std::sync::{Arc, OnceLock};
use uuid::Uuid;

const DB_PATH: &str = "./data/farley";

/// Global singleton: local sled DB + DashMap write-through cache for portfolios
/// and encrypted, compressed messages.
pub struct DataStore {
    db: sled::Db,
    portfolio_cache: DashMap<Uuid, Portfolio>,
    message_cache: DashMap<Uuid, Message>,
}

static STORE: OnceLock<Arc<DataStore>> = OnceLock::new();

/// Initialise (or retrieve) the global store. Call once at server start.
pub fn data_store() -> Arc<DataStore> {
    STORE
        .get_or_init(|| Arc::new(DataStore::open().expect("Failed to open local DB")))
        .clone()
}

/// Convenience accessor for the portfolio subset of the global store.
pub fn portfolio_store() -> Arc<DataStore> {
    data_store()
}

impl DataStore {
    fn open() -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(DB_PATH)?;
        let db = sled::open(DB_PATH)?;

        let store = Self {
            db,
            portfolio_cache: DashMap::new(),
            message_cache: DashMap::new(),
        };

        store.warm_portfolio_cache()?;
        store.warm_message_cache()?;
        Ok(store)
    }

    fn warm_portfolio_cache(&self) -> Result<(), Box<dyn std::error::Error>> {
        for item in self.db.iter() {
            let (k, v) = item?;
            // Portfolio keys are UUID strings; skip the message keyspace.
            if let Ok(id) = Uuid::parse_str(std::str::from_utf8(&k).unwrap_or("")) {
                if let Ok(p) = serde_json::from_slice::<Portfolio>(&v) {
                    self.portfolio_cache.insert(id, p);
                }
            }
        }
        Ok(())
    }

    fn warm_message_cache(&self) -> Result<(), Box<dyn std::error::Error>> {
        let key = crypto::storage_key();
        for item in self.db.iter() {
            let (k, v) = item?;
            // Message keys are prefixed with "msg:".
            if let Some(id_str) = k.strip_prefix(b"msg:") {
                if let Ok(id) = Uuid::parse_str(std::str::from_utf8(id_str).unwrap_or("")) {
                    if let Ok(m) = crypto::decompress_and_decrypt::<Message>(&v, &key) {
                        self.message_cache.insert(id, m);
                    }
                }
            }
        }
        Ok(())
    }

    // Portfolios

    /// Persist a single portfolio (write-through to DB + update cache).
    pub fn save_portfolio(&self, portfolio: &Portfolio) -> Result<(), Box<dyn std::error::Error>> {
        let key = portfolio.id.to_string();
        let value = serde_json::to_vec(portfolio)?;
        self.db.insert(key.as_bytes(), value)?;
        self.portfolio_cache.insert(portfolio.id, portfolio.clone());
        Ok(())
    }

    /// Load a single portfolio by ID (cache-first).
    pub fn load_portfolio(&self, id: Uuid) -> Option<Portfolio> {
        if let Some(p) = self.portfolio_cache.get(&id) {
            return Some(p.clone());
        }
        let key = id.to_string();
        self.db
            .get(key.as_bytes())
            .ok()
            .flatten()
            .and_then(|v| serde_json::from_slice::<Portfolio>(&v).ok())
            .inspect(|p| {
                self.portfolio_cache.insert(p.id, p.clone());
            })
    }

    /// Load all portfolios for a given owner (from cache).
    pub fn load_portfolios_for_owner(&self, owner_id: Uuid) -> Vec<Portfolio> {
        self.portfolio_cache
            .iter()
            .filter(|e| e.owner_id == owner_id)
            .map(|e| e.value().clone())
            .collect()
    }

    /// Load all portfolios.
    pub fn load_all_portfolios(&self) -> Vec<Portfolio> {
        self.portfolio_cache.iter().map(|e| e.value().clone()).collect()
    }

    /// Delete a portfolio from DB and cache.
    pub fn delete_portfolio(&self, id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        let key = id.to_string();
        self.db.remove(key.as_bytes())?;
        self.portfolio_cache.remove(&id);
        Ok(())
    }

    // Messages

    /// Persist a single message: compress and encrypt before writing to the local DB.
    pub fn save_message(&self, message: &Message) -> Result<(), Box<dyn std::error::Error>> {
        let key = crypto::storage_key();
        let id_key = format!("msg:{}", message.id);
        let value = crypto::compress_and_encrypt(message, &key)?;
        self.db.insert(id_key.as_bytes(), value)?;
        self.message_cache.insert(message.id, message.clone());
        Ok(())
    }

    /// Load a single message by ID (cache-first).
    pub fn load_message(&self, id: Uuid) -> Option<Message> {
        if let Some(m) = self.message_cache.get(&id) {
            return Some(m.clone());
        }
        let key = crypto::storage_key();
        let id_key = format!("msg:{}", id);
        self.db
            .get(id_key.as_bytes())
            .ok()
            .flatten()
            .and_then(|v| crypto::decompress_and_decrypt::<Message>(&v, &key).ok())
            .inspect(|m| {
                self.message_cache.insert(m.id, m.clone());
            })
    }

    /// Load all messages.
    pub fn load_all_messages(&self) -> Vec<Message> {
        self.message_cache.iter().map(|e| e.value().clone()).collect()
    }

    /// Delete a message from DB and cache.
    pub fn delete_message(&self, id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        let id_key = format!("msg:{}", id);
        self.db.remove(id_key.as_bytes())?;
        self.message_cache.remove(&id);
        Ok(())
    }
}

// Backwards-compatible re-exports for the existing portfolio-only API.
impl DataStore {
    pub fn save(&self, portfolio: &Portfolio) -> Result<(), Box<dyn std::error::Error>> {
        self.save_portfolio(portfolio)
    }

    pub fn load(&self, id: Uuid) -> Option<Portfolio> {
        self.load_portfolio(id)
    }

    pub fn load_all_for_owner(&self, owner_id: Uuid) -> Vec<Portfolio> {
        self.load_portfolios_for_owner(owner_id)
    }
}

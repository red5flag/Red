use crate::models::{Message, Organization, Portfolio};
use crate::stores::credentials::CredentialStore;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use rocksdb::{ColumnFamilyDescriptor, Options, DB};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, OnceLock};
use uuid::Uuid;

const ROCKS_DB_PATH: &str = "./data/farley_rocks";

/// Column family names for the different entity types.
/// This keeps the keyspace isolated and allows per-entity tuning.
const CF_PORTFOLIOS: &str = "portfolios";
const CF_ORGANIZATIONS: &str = "organizations";
const CF_MESSAGES: &str = "messages";
const CF_CREDENTIALS: &str = "credentials";
const CF_USERS: &str = "users";

/// Global singleton RocksDB instance.
static ROCKS_STORE: OnceLock<Arc<RocksDataStore>> = OnceLock::new();

/// Initialise (or retrieve) the global RocksDB-backed store.
pub fn rocks_data_store() -> Arc<RocksDataStore> {
    ROCKS_STORE
        .get_or_init(|| Arc::new(RocksDataStore::open().expect("Failed to open RocksDB")))
        .clone()
}

/// Convenience accessor for the portfolio subset of the global store.
pub fn rocks_portfolio_store() -> Arc<RocksDataStore> {
    rocks_data_store()
}

/// Convenience accessor for the message subset of the global store.
pub fn rocks_message_store() -> Arc<RocksDataStore> {
    rocks_data_store()
}

/// A container returned by the export endpoint: decompressed and decrypted
/// JSON for a single user.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserDataExport {
    pub user_id: Uuid,
    pub portfolios: Vec<Portfolio>,
    pub organizations: Vec<Organization>,
    pub messages: Vec<Message>,
    pub credentials: Option<CredentialStore>,
}

/// Wraps the on-disk encryption key and salt. The key stored here is derived
/// from the user's password with Argon2. It is never persisted to disk.
pub struct UserDataKey {
    pub key: [u8; 32],
    pub salt: [u8; 16],
}

/// Storage wrapper that compresses with zstd and encrypts with
/// ChaCha20-Poly1305 before writing to RocksDB.
pub struct RocksDataStore {
    db: DB,
}

impl RocksDataStore {
    /// Open the database, creating required column families.
    fn open() -> Result<Self, Box<dyn std::error::Error>> {
        Self::open_at(ROCKS_DB_PATH)
    }

    fn open_at<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(path.as_ref())?;

        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        let cfs = vec![
            ColumnFamilyDescriptor::new(CF_PORTFOLIOS, Options::default()),
            ColumnFamilyDescriptor::new(CF_ORGANIZATIONS, Options::default()),
            ColumnFamilyDescriptor::new(CF_MESSAGES, Options::default()),
            ColumnFamilyDescriptor::new(CF_CREDENTIALS, Options::default()),
            ColumnFamilyDescriptor::new(CF_USERS, Options::default()),
        ];

        let db = DB::open_cf_descriptors(&opts, path, cfs)?;
        Ok(Self { db })
    }

    /// Derive a 32-byte ChaCha20 key from a password and salt.
    /// If the password is empty, the data is still encrypted with a key derived
    /// from the empty string so the storage layer is always encrypted.
    fn derive_key(password: &str, salt: &[u8; 16]) -> [u8; 32] {
        let salt_str = SaltString::encode_b64(salt).unwrap_or_else(|_| SaltString::from_b64("AAAAAAAAAAAAAAAAAAAAAA").unwrap());
        let argon2 = Argon2::default();
        let mut key = [0u8; 32];
        if let Ok(hash) = argon2.hash_password(password.as_bytes(), &salt_str) {
            let b64 = hash.hash;
            if let Some(hash_bytes) = b64 {
                let bytes = hash_bytes.as_bytes();
                let len = bytes.len().min(32);
                key[..len].copy_from_slice(&bytes[..len]);
            }
        }
        key
    }

    /// Generate a fresh random 16-byte salt.
    pub fn generate_salt() -> [u8; 16] {
        let mut salt = [0u8; 16];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut salt);
        salt
    }

    /// Build a UserDataKey from a password and salt.
    pub fn user_key(password: &str, salt: [u8; 16]) -> UserDataKey {
        UserDataKey {
            key: Self::derive_key(password, &salt),
            salt,
        }
    }

    /// Compress then encrypt a serializable value.
    fn pack<T: Serialize>(value: &T, key: &UserDataKey) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let json = serde_json::to_vec(value)?;
        let compressed = zstd::encode_all(&json[..], 0)?;
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&key.key));
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = cipher.encrypt(&nonce, compressed.as_ref()).map_err(|e| e.to_string())?;
        // Store: [nonce (12 bytes)][ciphertext]
        let mut packed = Vec::with_capacity(nonce.len() + ciphertext.len());
        packed.extend_from_slice(&nonce);
        packed.extend_from_slice(&ciphertext);
        Ok(packed)
    }

    /// Decrypt then decompress a packed value.
    fn unpack<T: for<'de> Deserialize<'de>>(packed: &[u8], key: &UserDataKey) -> Result<T, Box<dyn std::error::Error>> {
        if packed.len() < 12 {
            return Err("Packed value too short".into());
        }
        let nonce = Nonce::from_slice(&packed[..12]);
        let ciphertext = &packed[12..];
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&key.key));
        let compressed = cipher.decrypt(nonce, ciphertext).map_err(|e| e.to_string())?;
        let mut json = zstd::decode_all(&compressed[..])?;
        let value = serde_json::from_slice(&mut json)?;
        Ok(value)
    }

    /// Key layout: `{user_id}:{entity_id}`.
    fn entity_key(user_id: Uuid, entity_id: Uuid) -> String {
        format!("{}:{}", user_id, entity_id)
    }

    /// Key layout for user-scoped metadata: `{user_id}:meta`.
    fn user_meta_key(user_id: Uuid) -> String {
        format!("{}:meta", user_id)
    }

    /// Persist a portfolio for a user.
    pub fn save_portfolio(
        &self,
        user_id: Uuid,
        portfolio: &Portfolio,
        key: &UserDataKey,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cf = self.db.cf_handle(CF_PORTFOLIOS).ok_or("Missing portfolios column family")?;
        let k = Self::entity_key(user_id, portfolio.id);
        let v = self.pack(portfolio, key)?;
        self.db.put_cf(&cf, k.as_bytes(), &v)?;
        Ok(())
    }

    /// Load a portfolio by user + portfolio id.
    pub fn load_portfolio(
        &self,
        user_id: Uuid,
        portfolio_id: Uuid,
        key: &UserDataKey,
    ) -> Option<Portfolio> {
        let cf = self.db.cf_handle(CF_PORTFOLIOS)?;
        let k = Self::entity_key(user_id, portfolio_id);
        self.db
            .get_cf(&cf, k.as_bytes())
            .ok()
            .flatten()
            .and_then(|v| self.unpack(&v, key).ok())
    }

    /// Load all portfolios for a user.
    pub fn load_portfolios_for_user(
        &self,
        user_id: Uuid,
        key: &UserDataKey,
    ) -> Result<Vec<Portfolio>, Box<dyn std::error::Error>> {
        let cf = self.db.cf_handle(CF_PORTFOLIOS).ok_or("Missing portfolios column family")?;
        let prefix = format!("{}:", user_id);
        let mut portfolios = Vec::new();
        let iter = self.db.iterator_cf(&cf, rocksdb::IteratorMode::From(prefix.as_bytes(), rocksdb::Direction::Forward));
        for item in iter {
            let (k, v) = item?;
            if !std::str::from_utf8(&k).unwrap_or("").starts_with(&prefix) {
                break;
            }
            // Skip the meta key if it ever ends up here.
            if k.ends_with(b":meta") {
                continue;
            }
            if let Ok(p) = self.unpack::<Portfolio>(&v, key) {
                portfolios.push(p);
            }
        }
        Ok(portfolios)
    }

    /// Delete a portfolio for a user.
    pub fn delete_portfolio(
        &self,
        user_id: Uuid,
        portfolio_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cf = self.db.cf_handle(CF_PORTFOLIOS).ok_or("Missing portfolios column family")?;
        let k = Self::entity_key(user_id, portfolio_id);
        self.db.delete_cf(&cf, k.as_bytes())?;
        Ok(())
    }

    /// Persist an organization for a user.
    pub fn save_organization(
        &self,
        user_id: Uuid,
        organization: &Organization,
        key: &UserDataKey,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cf = self.db.cf_handle(CF_ORGANIZATIONS).ok_or("Missing organizations column family")?;
        let k = Self::entity_key(user_id, organization.id);
        let v = self.pack(organization, key)?;
        self.db.put_cf(&cf, k.as_bytes(), &v)?;
        Ok(())
    }

    /// Load all organizations for a user.
    pub fn load_organizations_for_user(
        &self,
        user_id: Uuid,
        key: &UserDataKey,
    ) -> Result<Vec<Organization>, Box<dyn std::error::Error>> {
        let cf = self.db.cf_handle(CF_ORGANIZATIONS).ok_or("Missing organizations column family")?;
        let prefix = format!("{}:", user_id);
        let mut orgs = Vec::new();
        let iter = self.db.iterator_cf(&cf, rocksdb::IteratorMode::From(prefix.as_bytes(), rocksdb::Direction::Forward));
        for item in iter {
            let (k, v) = item?;
            if !std::str::from_utf8(&k).unwrap_or("").starts_with(&prefix) {
                break;
            }
            if k.ends_with(b":meta") {
                continue;
            }
            if let Ok(o) = self.unpack::<Organization>(&v, key) {
                orgs.push(o);
            }
        }
        Ok(orgs)
    }

    /// Delete an organization for a user.
    pub fn delete_organization(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cf = self.db.cf_handle(CF_ORGANIZATIONS).ok_or("Missing organizations column family")?;
        let k = Self::entity_key(user_id, organization_id);
        self.db.delete_cf(&cf, k.as_bytes())?;
        Ok(())
    }

    /// Persist a message.
    pub fn save_message(
        &self,
        user_id: Uuid,
        message: &Message,
        key: &UserDataKey,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cf = self.db.cf_handle(CF_MESSAGES).ok_or("Missing messages column family")?;
        let k = Self::entity_key(user_id, message.id);
        let v = self.pack(message, key)?;
        self.db.put_cf(&cf, k.as_bytes(), &v)?;
        Ok(())
    }

    /// Load all messages for a user.
    pub fn load_messages_for_user(
        &self,
        user_id: Uuid,
        key: &UserDataKey,
    ) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
        let cf = self.db.cf_handle(CF_MESSAGES).ok_or("Missing messages column family")?;
        let prefix = format!("{}:", user_id);
        let mut messages = Vec::new();
        let iter = self.db.iterator_cf(&cf, rocksdb::IteratorMode::From(prefix.as_bytes(), rocksdb::Direction::Forward));
        for item in iter {
            let (k, v) = item?;
            if !std::str::from_utf8(&k).unwrap_or("").starts_with(&prefix) {
                break;
            }
            if k.ends_with(b":meta") {
                continue;
            }
            if let Ok(m) = self.unpack::<Message>(&v, key) {
                messages.push(m);
            }
        }
        Ok(messages)
    }

    /// Persist a user's credential store.
    pub fn save_credentials(
        &self,
        user_id: Uuid,
        credentials: &CredentialStore,
        key: &UserDataKey,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cf = self.db.cf_handle(CF_CREDENTIALS).ok_or("Missing credentials column family")?;
        let k = Self::user_meta_key(user_id);
        let v = self.pack(credentials, key)?;
        self.db.put_cf(&cf, k.as_bytes(), &v)?;
        Ok(())
    }

    /// Load a user's credential store.
    pub fn load_credentials(
        &self,
        user_id: Uuid,
        key: &UserDataKey,
    ) -> Option<CredentialStore> {
        let cf = self.db.cf_handle(CF_CREDENTIALS).ok_or("Missing credentials column family").ok()?;
        let k = Self::user_meta_key(user_id);
        self.db
            .get_cf(&cf, k.as_bytes())
            .ok()
            .flatten()
            .and_then(|v| self.unpack(&v, key).ok())
    }

    /// Store the encryption salt for a user so the same key can be re-derived later.
    /// Key: `users:{user_id}:salt`.
    pub fn save_user_salt(
        &self,
        user_id: Uuid,
        salt: &[u8; 16],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cf = self.db.cf_handle(CF_USERS).ok_or("Missing users column family")?;
        let k = format!("{}:salt", user_id);
        self.db.put_cf(&cf, k.as_bytes(), salt)?;
        Ok(())
    }

    /// Load the encryption salt for a user.
    pub fn load_user_salt(&self, user_id: Uuid) -> Option<[u8; 16]> {
        let cf = self.db.cf_handle(CF_USERS).ok()?;
        let k = format!("{}:salt", user_id);
        self.db
            .get_cf(&cf, k.as_bytes())
            .ok()
            .flatten()
            .and_then(|v| {
                if v.len() == 16 {
                    let mut salt = [0u8; 16];
                    salt.copy_from_slice(&v);
                    Some(salt)
                } else {
                    None
                }
            })
    }

    /// Export every decryptable entity for a user.
    pub fn export_user_data(
        &self,
        user_id: Uuid,
        key: &UserDataKey,
    ) -> Result<UserDataExport, Box<dyn std::error::Error>> {
        let portfolios = self.load_portfolios_for_user(user_id, key)?;
        let organizations = self.load_organizations_for_user(user_id, key)?;
        let messages = self.load_messages_for_user(user_id, key)?;
        let credentials = self.load_credentials(user_id, key);
        Ok(UserDataExport {
            user_id,
            portfolios,
            organizations,
            messages,
            credentials,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn round_trip_portfolio() {
        let dir = "./test_data_rocks";
        let _ = fs::remove_dir_all(dir);
        let store = RocksDataStore::open_at(dir).unwrap();
        let key = RocksDataStore::user_key("password", RocksDataStore::generate_salt());
        let p = Portfolio::default();
        let user_id = Uuid::new_v4();
        store.save_portfolio(user_id, &p, &key).unwrap();
        let loaded = store.load_portfolio(user_id, p.id, &key).unwrap();
        assert_eq!(loaded.id, p.id);
        let _ = fs::remove_dir_all(dir);
    }
}

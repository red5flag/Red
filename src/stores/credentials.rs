use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use base64::Engine as _;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn default_true() -> bool {
    true
}

// Obfuscation key for locally-stored remembered passwords.
// This is compile-time static obfuscation, not a user-provided secret.
#[cfg(any(feature = "ssr", feature = "hydrate"))]
const REMEMBERED_PASSWORD_KEY: [u8; 32] = *b"farley-local-remember-password!!";

#[cfg(any(feature = "ssr", feature = "hydrate"))]
pub fn encrypt_remembered_password(plain: &str) -> Option<String> {
    use chacha20poly1305::aead::{Aead, KeyInit};
    use chacha20poly1305::{ChaCha20Poly1305, Nonce};
    if plain.is_empty() {
        return None;
    }
    let cipher = ChaCha20Poly1305::new_from_slice(&REMEMBERED_PASSWORD_KEY).ok()?;
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, plain.as_bytes()).ok()?;
    let mut out = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Some(base64::engine::general_purpose::STANDARD.encode(&out))
}

#[cfg(any(feature = "ssr", feature = "hydrate"))]
pub fn decrypt_remembered_password(encoded: &str) -> Option<String> {
    use chacha20poly1305::aead::{Aead, KeyInit};
    use chacha20poly1305::{ChaCha20Poly1305, Nonce};
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .ok()?;
    if bytes.len() < 12 {
        // Fall back to treating the value as a legacy plaintext password.
        return Some(encoded.to_string());
    }
    let (nonce_bytes, ciphertext) = bytes.split_at(12);
    let cipher = ChaCha20Poly1305::new_from_slice(&REMEMBERED_PASSWORD_KEY).ok()?;
    let nonce = Nonce::from_slice(nonce_bytes);
    match cipher.decrypt(nonce, ciphertext) {
        Ok(plain) => String::from_utf8(plain).ok(),
        Err(_) => {
            // Legacy plaintext fallback.
            Some(encoded.to_string())
        }
    }
}

#[cfg(not(any(feature = "ssr", feature = "hydrate")))]
fn encrypt_remembered_password(_plain: &str) -> Option<String> {
    None
}

#[cfg(not(any(feature = "ssr", feature = "hydrate")))]
fn decrypt_remembered_password(encoded: &str) -> Option<String> {
    Some(encoded.to_string())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredCredential {
    pub username: String,
    pub password_hash: String,
    pub display_name: String,
    pub email: String,
    pub validated: bool,
    pub totp_secret: Option<String>,
    pub totp_enabled: bool,
    pub email_2fa_enabled: bool,
    #[serde(default = "default_true")]
    pub store_local: bool,
    #[serde(default)]
    pub store_cloud: bool,
    // Plain-text password retained only when the user opts in to "Remember Password".
    // This is intentionally stored locally for convenience in this dev/test build.
    #[serde(default)]
    pub remembered_password: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CredentialStore {
    pub credentials: HashMap<String, StoredCredential>,
}

impl CredentialStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize an empty credential store (no default users).
    pub fn with_defaults() -> Self {
        Self::new()
    }

    /// Load credentials from localStorage (hydrate only), plain text for development
    #[cfg(feature = "hydrate")]
    pub fn load_from_local_storage() -> Self {
        use web_sys::window;
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(json)) = storage.get_item("farley_credentials") {
                    if let Ok(store) = serde_json::from_str::<Self>(&json) {
                        return store;
                    }
                }
            }
        }
        Self::new()
    }

    /// Save credentials to localStorage (hydrate only), plain text for development
    #[cfg(feature = "hydrate")]
    pub fn save_to_local_storage(&self) {
        use web_sys::window;
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(json) = serde_json::to_string(self) {
                    let _ = storage.set_item("farley_credentials", &json);
                }
            }
        }
    }

    /// Merge locally stored credentials into this store (hydrate only)
    #[cfg(feature = "hydrate")]
    pub fn merge_from_local_storage(&mut self) {
        let stored = Self::load_from_local_storage();
        for (username, cred) in stored.credentials {
            // Only add credentials that don't already exist locally,
            // so the default validated red user isn't overwritten by stale storage.
            self.credentials.entry(username).or_insert(cred);
        }
    }

    /// Hash a password using argon2
    pub fn hash_password(password: &str) -> Result<String, String> {
        let mut salt_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut salt_bytes);
        let salt = SaltString::encode_b64(&salt_bytes)
            .map_err(|e| format!("Failed to create salt: {}", e))?;
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| format!("Failed to hash password: {}", e))
    }

    /// Verify a password against a stored hash
    pub fn verify_password(password: &str, hash: &str) -> bool {
        match PasswordHash::new(hash) {
            Ok(parsed_hash) => Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok(),
            Err(_) => false,
        }
    }

    /// Add a user with username and password (password will be hashed)
    pub fn add_user(
        &mut self,
        username: &str,
        password: &str,
        display_name: &str,
        email: &str,
        validated: bool,
        store_local: bool,
        store_cloud: bool,
        remembered_password: Option<String>,
    ) {
        if let Ok(hash) = Self::hash_password(password) {
            let remembered_password = remembered_password
                .as_deref()
                .and_then(encrypt_remembered_password);
            self.credentials.insert(
                username.to_string(),
                StoredCredential {
                    username: username.to_string(),
                    password_hash: hash,
                    display_name: display_name.to_string(),
                    email: email.to_string(),
                    validated,
                    totp_secret: None,
                    totp_enabled: false,
                    email_2fa_enabled: false,
                    store_local,
                    store_cloud,
                    remembered_password,
                },
            );
        }
    }

    /// Save (or update) a password for a user in the credential store.
    /// If the user already exists, only the password hash is updated.
    /// If the user does not exist, a new entry is created with default values.
    /// `remembered_password` is the optional plain-text password kept when the user opts in to Remember Password.
    pub fn save_password(
        &mut self,
        username: &str,
        password: &str,
        display_name: &str,
        email: &str,
        remembered_password: Option<String>,
    ) {
        if let Ok(hash) = Self::hash_password(password) {
            let encrypted_remembered = remembered_password
                .as_deref()
                .and_then(encrypt_remembered_password);
            if let Some(cred) = self.credentials.get_mut(username) {
                cred.password_hash = hash;
                if encrypted_remembered.is_some() {
                    cred.remembered_password = encrypted_remembered;
                }
            } else {
                self.credentials.insert(
                    username.to_string(),
                    StoredCredential {
                        username: username.to_string(),
                        password_hash: hash,
                        display_name: display_name.to_string(),
                        email: email.to_string(),
                        validated: false,
                        totp_secret: None,
                        totp_enabled: false,
                        email_2fa_enabled: false,
                        store_local: true,
                        store_cloud: false,
                        remembered_password: encrypted_remembered,
                    },
                );
            }
        }
    }

    /// Verify credentials against stored users
    pub fn verify(&self, username: &str, password: &str) -> Option<&StoredCredential> {
        self.credentials
            .get(username)
            .filter(|cred| Self::verify_password(password, &cred.password_hash))
    }

    /// Check if a user exists
    pub fn user_exists(&self, username: &str) -> bool {
        self.credentials.contains_key(username)
    }

    /// Register a new user. Returns Ok if successful, Err with message if user exists.
    pub fn register_user(
        &mut self,
        username: &str,
        password: &str,
        display_name: &str,
        email: &str,
        store_local: bool,
        store_cloud: bool,
    ) -> Result<(), String> {
        if self.user_exists(username) {
            return Err(format!("Username '{}' is already taken", username));
        }
        if password.len() < 3 {
            return Err("Password must be at least 3 characters".to_string());
        }
        if username.trim().is_empty() {
            return Err("Username is required".to_string());
        }
        if !email.contains('@') {
            return Err("A valid email is required".to_string());
        }
        self.add_user(
            username,
            password,
            display_name,
            email,
            false,
            store_local,
            store_cloud,
            Some(password.to_string()),
        );
        Ok(())
    }

    /// Mark a user as validated
    pub fn mark_validated(&mut self, username: &str) {
        if let Some(cred) = self.credentials.get_mut(username) {
            cred.validated = true;
        }
    }

    /// Check if a user's email has been validated
    pub fn is_validated(&self, username: &str) -> bool {
        self.credentials
            .get(username)
            .map(|c| c.validated)
            .unwrap_or(false)
    }

    /// Verify password without checking validation status
    pub fn verify_password_only(&self, username: &str, password: &str) -> bool {
        self.credentials
            .get(username)
            .map(|cred| Self::verify_password(password, &cred.password_hash))
            .unwrap_or(false)
    }

    /// Set both local and cloud storage preferences for a user
    pub fn set_storage_options(&mut self, username: &str, store_local: bool, store_cloud: bool) {
        if let Some(cred) = self.credentials.get_mut(username) {
            cred.store_local = store_local;
            cred.store_cloud = store_cloud;
        }
    }

    /// Get local and cloud storage preferences for a user, defaulting to (true, false).
    pub fn get_storage_options(&self, username: &str) -> (bool, bool) {
        self.credentials
            .get(username)
            .map(|cred| (cred.store_local, cred.store_cloud))
            .unwrap_or((true, false))
    }
}

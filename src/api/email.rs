use cfg_if::cfg_if;
use serde::{Deserialize, Serialize};

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use std::collections::VecDeque;
        use std::sync::OnceLock;
        use tokio::sync::Mutex as TokioMutex;
        use argon2::{
            password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
            Argon2,
        };
        use rand::RngCore;

        const VALIDATED_USERS_FILE: &str = "data/validated_users.json";
        const PENDING_REGS_FILE: &str = "data/pending_registrations.json";

        static EMAIL_QUEUE: OnceLock<TokioMutex<VecDeque<ValidationEmail>>> = OnceLock::new();
        static PENDING_REGISTRATIONS: OnceLock<TokioMutex<Vec<PendingRegistration>>> = OnceLock::new();
        static VALIDATED_USERS: OnceLock<TokioMutex<Vec<ValidatedUser>>> = OnceLock::new();

        fn email_queue() -> &'static TokioMutex<VecDeque<ValidationEmail>> {
            EMAIL_QUEUE.get_or_init(|| TokioMutex::new(VecDeque::new()))
        }

        fn pending_registrations() -> &'static TokioMutex<Vec<PendingRegistration>> {
            PENDING_REGISTRATIONS.get_or_init(|| {
                let users = load_from_file::<PendingRegistration>(PENDING_REGS_FILE);
                TokioMutex::new(users)
            })
        }

        fn validated_users() -> &'static TokioMutex<Vec<ValidatedUser>> {
            VALIDATED_USERS.get_or_init(|| {
                let users = load_from_file::<ValidatedUser>(VALIDATED_USERS_FILE);
                TokioMutex::new(users)
            })
        }

        fn load_from_file<T: for<'de> serde::Deserialize<'de>>(path: &str) -> Vec<T> {
            std::fs::create_dir_all("data").ok();
            match std::fs::read_to_string(path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => Vec::new(),
            }
        }

        fn save_to_file<T: serde::Serialize>(path: &str, data: &[T]) {
            std::fs::create_dir_all("data").ok();
            if let Ok(json) = serde_json::to_string_pretty(data) {
                let _ = std::fs::write(path, json);
            }
        }

        pub fn hash_password(password: &str) -> Result<String, String> {
            let mut salt_bytes = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut salt_bytes);
            let salt = SaltString::encode_b64(&salt_bytes)
                .map_err(|e| format!("Failed to create salt: {}", e))?;
            Argon2::default()
                .hash_password(password.as_bytes(), &salt)
                .map(|hash| hash.to_string())
                .map_err(|e| format!("Failed to hash password: {}", e))
        }

        pub async fn enqueue_email(email: ValidationEmail) {
            email_queue().lock().await.push_back(email);
        }

        pub async fn get_pending_emails() -> Vec<ValidationEmail> {
            email_queue().lock().await.iter().cloned().collect()
        }

        pub async fn add_pending_registration(
            username: String,
            password: String,
            email: String,
        ) -> Result<ValidationEmail, String> {
            {
                let pending = pending_registrations().lock().await;
                if pending.iter().any(|p| p.username == username) {
                    return Err(format!("Username '{}' is already pending validation", username));
                }
            }
            {
                let validated = validated_users().lock().await;
                if validated.iter().any(|v| v.username == username) {
                    return Err(format!("Username '{}' is already taken", username));
                }
            }

            let password_hash = hash_password(&password)?;
            let validation_email = ValidationEmail::new(email.clone(), username.clone());

            let pending_reg = PendingRegistration {
                username: username.clone(),
                password_hash,
                email: email.clone(),
                validation_token: validation_email.validation_token.clone(),
                created_at: chrono::Utc::now().to_rfc3339(),
            };

            {
                let mut pending = pending_registrations().lock().await;
                pending.push(pending_reg);
                save_to_file(PENDING_REGS_FILE, &pending);
            }
            enqueue_email(validation_email.clone()).await;

            Ok(validation_email)
        }

        pub async fn validate_user_by_token(token: &str) -> Result<ValidatedUser, String> {
            // Check if already validated (e.g. user clicked the link twice)
            let already_validated: Option<ValidatedUser> = {
                let validated = validated_users().lock().await;
                validated.iter().find(|v| token.starts_with(&format!("val_{}_", v.username))).cloned()
            };
            if let Some(v) = already_validated {
                // Remove the email from queue if present
                remove_email_by_token(token).await;
                return Ok(v);
            }

            let reg = {
                let mut pending = pending_registrations().lock().await;
                let pos = pending
                    .iter()
                    .position(|p| p.validation_token == token)
                    .ok_or("Invalid or expired validation token")?;
                let reg = pending.remove(pos);
                save_to_file(PENDING_REGS_FILE, &pending);
                reg
            };

            let validated = ValidatedUser {
                username: reg.username.clone(),
                password_hash: reg.password_hash,
                display_name: reg.username.clone(),
                email: reg.email.clone(),
                validated_at: chrono::Utc::now().to_rfc3339(),
                totp_secret: None,
                totp_enabled: false,
                email_2fa_enabled: false,
                phone_number: None,
                phone_2fa_enabled: false,
            };

            {
                let mut validated_list = validated_users().lock().await;
                validated_list.push(validated.clone());
                save_to_file(VALIDATED_USERS_FILE, &validated_list);
            }

            // Remove the email from the in-memory queue
            remove_email_by_token(token).await;

            Ok(validated)
        }

        pub async fn remove_email_by_token(token: &str) {
            let mut q = email_queue().lock().await;
            q.retain(|e| e.validation_token != token);
        }

        pub async fn check_validated_user(username: &str, password: &str) -> Option<ValidatedUser> {
            let validated = validated_users().lock().await;
            for user in validated.iter() {
                if user.username == username {
                    if let Ok(parsed_hash) = PasswordHash::new(&user.password_hash) {
                        if Argon2::default()
                            .verify_password(password.as_bytes(), &parsed_hash)
                            .is_ok()
                        {
                            return Some(user.clone());
                        }
                    }
                    return None;
                }
            }
            None
        }

        pub async fn get_pending_registrations() -> Vec<PendingRegistration> {
            pending_registrations().lock().await.clone()
        }

        /// Generate a new TOTP secret and provisioning URI for Google Authenticator.
        pub fn generate_totp_secret(username: &str, issuer: &str) -> Result<(String, String), String> {
            use totp_rs::{TOTP, Secret, Algorithm};
            let secret = Secret::generate_secret();
            let secret_bytes = secret.to_bytes().map_err(|e| e.to_string())?;
            let totp = TOTP::new(
                Algorithm::SHA1,
                6,
                1,
                30,
                secret_bytes,
                Some(issuer.to_string()),
                username.to_string(),
            ).map_err(|e| e.to_string())?;
            let secret_base32 = totp.get_secret_base32();
            let uri = totp.get_url();
            Ok((secret_base32, uri))
        }

        /// Verify a TOTP code against a base32 secret.
        pub fn verify_totp_code(secret_base32: &str, code: &str) -> bool {
            use totp_rs::{TOTP, Secret, Algorithm};
            let secret = match Secret::Encoded(secret_base32.to_string()).to_bytes() {
                Ok(s) => s,
                Err(_) => return false,
            };
            let totp = match TOTP::new(Algorithm::SHA1, 6, 1, 30, secret, None, "".to_string()) {
                Ok(t) => t,
                Err(_) => return false,
            };
            totp.check_current(code).unwrap_or(false)
        }

        /// Generate a 6-digit email 2FA code.
        pub fn generate_email_2fa_code() -> String {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            format!("{:06}", rng.gen_range(0..1_000_000u32))
        }

        /// Enqueue a 2FA email with a 6-digit code.
        pub async fn enqueue_2fa_email(email: String, username: String, code: String) {
            let subject = format!("Red - Your sign-in code for '{}'", username);
            let body = format!(
                "Your Red sign-in code is: {}\n\nThis code is valid for 10 minutes.\n\nIf you did not request this code, please ignore this email.\n\n- Red Team",
                code
            );
            let email = ValidationEmail {
                to: email,
                username,
                validation_token: format!("2fa_{}_{}", code, chrono::Utc::now().timestamp()),
                subject,
                body,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            enqueue_email(email).await;
        }

        /// Find a validated user by username.
        pub async fn get_validated_user(username: &str) -> Option<ValidatedUser> {
            validated_users().lock().await.iter().find(|u| u.username == username).cloned()
        }

        /// Update a validated user's 2FA settings.
        pub async fn update_user_2fa(user: ValidatedUser) -> Result<(), String> {
            let mut validated = validated_users().lock().await;
            let pos = validated.iter().position(|u| u.username == user.username)
                .ok_or("User not found")?;
            validated[pos] = user;
            save_to_file(VALIDATED_USERS_FILE, &validated);
            Ok(())
        }

        /// Generate a 6-digit SMS 2FA code.
        pub fn generate_phone_2fa_code() -> String {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            format!("{:06}", rng.gen_range(0..1_000_000u32))
        }

        /// Enqueue a simulated phone/SMS 2FA email (stored as email since no SMS gateway is integrated).
        pub async fn enqueue_phone_2fa_email(email: String, username: String, phone: String, code: String) {
            let subject = format!("Red - Phone verification code for '{}'", username);
            let body = format!(
                "Your Red phone 2FA sign-in code is: {}\n\nThis code was sent to: {}\n\nValid for 10 minutes.\n\n- Red Team",
                code, phone
            );
            let email = ValidationEmail {
                to: email,
                username,
                validation_token: format!("phone2fa_{}_{}", code, chrono::Utc::now().timestamp()),
                subject,
                body,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            enqueue_email(email).await;
        }

        /// Find a validated user by email or by phone number.
        pub async fn find_validated_user_by_email_or_phone(email: &str, phone: &str) -> Option<ValidatedUser> {
            let validated = validated_users().lock().await;
            validated.iter().find(|u| {
                (!email.is_empty() && u.email.eq_ignore_ascii_case(email))
                    || (!phone.is_empty() && u.phone_number.as_deref().unwrap_or("") == phone)
            }).cloned()
        }

        /// Enqueue a recovery email with a 6-digit code.
        pub async fn enqueue_recovery_email(email: String, username: String, code: String) {
            let subject = format!("Red - Account recovery code for '{}'", username);
            let body = format!(
                "Your Red account recovery code is: {}\n\nThis code is valid for 10 minutes.\n\nIf you did not request this code, please ignore this email.\n\n- Red Team",
                code
            );
            let username_clone = username.clone();
            let email = ValidationEmail {
                to: email,
                username,
                validation_token: format!("recover_{}_{}", username_clone, code),
                subject,
                body,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            enqueue_email(email).await;
        }

        /// Generate a secure random recovery passkey (temporary login password).
        pub fn generate_recovery_passkey() -> String {
            use rand::{thread_rng, Rng};
            const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
            let mut rng = thread_rng();
            (0..24)
                .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
                .collect()
        }

        /// Enqueue an account recovery email containing the temporary passkey and 2FA code.
        pub async fn enqueue_account_recovery_email(
            email: String,
            username: String,
            passkey: String,
            two_fa_code: String,
        ) {
            let subject = format!("Red - Account recovery for '{}'", username);
            let body = format!(
                "You requested account recovery for '{}'.\n\nTemporary login passkey: {}\n2FA verification code: {}\n\nLog in with the passkey, then enter the 2FA code when prompted.\n\nIf you did not request this, please ignore this email.\n\n- Red Team",
                username, passkey, two_fa_code
            );
            let email = ValidationEmail {
                to: email,
                username: username.clone(),
                validation_token: format!("recover_{}_{}", username, two_fa_code),
                subject,
                body,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            enqueue_email(email).await;
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationEmail {
    pub to: String,
    pub username: String,
    pub validation_token: String,
    pub subject: String,
    pub body: String,
    pub timestamp: String,
}

impl ValidationEmail {
    pub fn new(to: String, username: String) -> Self {
        let token = format!("val_{}_{}", username, chrono::Utc::now().timestamp());
        Self {
            to: to.clone(),
            username: username.clone(),
            validation_token: token.clone(),
            subject: format!("Red - Validate your email for user '{}'", username),
            body: format!(
                "Welcome to Red!\n\nPlease validate your email address for username '{}'.\n\nYour validation token is: {}\n\nIf you did not create an account, please ignore this email.\n\n- Red Team",
                username, token
            ),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Send a validation email (SSR: stores in memory queue for testing; CSR: no-op)
pub async fn send_validation_email(
    to: String,
    username: String,
) -> Result<ValidationEmail, String> {
    let email = ValidationEmail::new(to, username);
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            enqueue_email(email.clone()).await;
            Ok(email)
        } else {
            Ok(email)
        }
    }
}

/// Get all pending validation emails.
pub async fn get_validation_emails() -> Vec<ValidationEmail> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            get_pending_emails().await
        } else {
            Vec::new()
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingRegistration {
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub validation_token: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatedUser {
    pub username: String,
    pub password_hash: String,
    pub display_name: String,
    pub email: String,
    pub validated_at: String,
    #[serde(default)]
    pub totp_secret: Option<String>,
    #[serde(default)]
    pub totp_enabled: bool,
    #[serde(default)]
    pub email_2fa_enabled: bool,
    #[serde(default)]
    pub phone_number: Option<String>,
    #[serde(default)]
    pub phone_2fa_enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignupResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidateRequest {
    pub token: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidateResponse {
    pub success: bool,
    pub message: String,
    pub username: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub requires_totp: bool,
    pub requires_email_2fa: bool,
    #[serde(default)]
    pub requires_phone_2fa: bool,
    pub totp_uri: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifyTotpRequest {
    pub username: String,
    pub code: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifyTotpResponse {
    pub success: bool,
    pub message: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifyEmail2faRequest {
    pub username: String,
    pub code: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifyEmail2faResponse {
    pub success: bool,
    pub message: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnableTotpRequest {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnableTotpResponse {
    pub success: bool,
    pub message: String,
    pub secret: Option<String>,
    pub uri: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfirmTotpRequest {
    pub username: String,
    pub code: String,
    pub secret: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfirmTotpResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToggleEmail2faRequest {
    pub username: String,
    pub password: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToggleEmail2faResponse {
    pub success: bool,
    pub message: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TogglePhone2faRequest {
    pub username: String,
    pub password: String,
    pub enabled: bool,
    pub phone_number: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TogglePhone2faResponse {
    pub success: bool,
    pub message: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifyPhone2faRequest {
    pub username: String,
    pub code: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifyPhone2faResponse {
    pub success: bool,
    pub message: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncCredentialsRequest {
    pub username: String,
    pub credentials: crate::stores::credentials::CredentialStore,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncCredentialsResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoadCredentialsRequest {
    pub username: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoadCredentialsResponse {
    pub success: bool,
    pub message: String,
    pub credentials: Option<crate::stores::credentials::CredentialStore>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecoverRequest {
    pub email: String,
    pub phone: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecoverResponse {
    pub success: bool,
    pub message: String,
    pub username: Option<String>,
    pub recovery_code: Option<String>,
    pub passkey: Option<String>,
    pub two_fa_code: Option<String>,
}

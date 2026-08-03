use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use axum::extract::Query;
        use axum::Json as AxumJson;
        use axum::http::StatusCode;
        use crate::api::email;
        use crate::api::email::{SignupRequest, SignupResponse, ValidateRequest, ValidateResponse, LoginRequest, LoginResponse,
            VerifyTotpRequest, VerifyTotpResponse, VerifyEmail2faRequest, VerifyEmail2faResponse,
            EnableTotpRequest, EnableTotpResponse, ConfirmTotpRequest, ConfirmTotpResponse,
            ToggleEmail2faRequest, ToggleEmail2faResponse,
            TogglePhone2faRequest, TogglePhone2faResponse,
            VerifyPhone2faRequest, VerifyPhone2faResponse,
            SyncCredentialsRequest, SyncCredentialsResponse, LoadCredentialsRequest, LoadCredentialsResponse,
            RecoverRequest, RecoverResponse};

        // API: POST /api/signup
        pub async fn api_signup(
            axum::Json(req): axum::Json<SignupRequest>,
        ) -> Result<AxumJson<SignupResponse>, (StatusCode, String)> {
            if req.username.trim().is_empty() {
                return Ok(AxumJson(SignupResponse {
                    success: false,
                    message: "Username is required".to_string(),
                }));
            }
            if req.password.len() < 3 {
                return Ok(AxumJson(SignupResponse {
                    success: false,
                    message: "Password must be at least 3 characters".to_string(),
                }));
            }
            if !req.email.contains('@') {
                return Ok(AxumJson(SignupResponse {
                    success: false,
                    message: "A valid email is required".to_string(),
                }));
            }

            match email::add_pending_registration(req.username, req.password, req.email).await {
                Ok(_) => Ok(AxumJson(SignupResponse {
                    success: true,
                    message: "Account created! Check your email for the validation token.".to_string(),
                })),
                Err(e) => Ok(AxumJson(SignupResponse {
                    success: false,
                    message: e,
                })),
            }
        }

        // API: POST /api/validate
        pub async fn api_validate(
            axum::Json(req): axum::Json<ValidateRequest>,
        ) -> Result<AxumJson<ValidateResponse>, (StatusCode, String)> {
            match email::validate_user_by_token(&req.token).await {
                Ok(user) => Ok(AxumJson(ValidateResponse {
                    success: true,
                    message: format!("User '{}' validated successfully", user.username),
                    username: Some(user.username),
                })),
                Err(e) => Ok(AxumJson(ValidateResponse {
                    success: false,
                    message: e,
                    username: None,
                })),
            }
        }

        // API: POST /api/login
        pub async fn api_login(
            axum::Json(req): axum::Json<LoginRequest>,
        ) -> Result<AxumJson<LoginResponse>, (StatusCode, String)> {
            match email::check_validated_user(&req.username, &req.password).await {
                Some(user) => {
                    let requires_totp = user.totp_enabled;
                    let requires_email_2fa = user.email_2fa_enabled;
                    let requires_phone_2fa = user.phone_2fa_enabled;
                    if requires_email_2fa {
                        let code = email::generate_email_2fa_code();
                        email::enqueue_2fa_email(user.email.clone(), user.username.clone(), code).await;
                    }
                    if requires_phone_2fa {
                        let code = email::generate_phone_2fa_code();
                        let phone = user.phone_number.clone().unwrap_or_default();
                        email::enqueue_phone_2fa_email(user.email.clone(), user.username.clone(), phone, code).await;
                    }
                    let needs_2fa = requires_totp || requires_email_2fa || requires_phone_2fa;
                    Ok(AxumJson(LoginResponse {
                        success: !needs_2fa,
                        message: if needs_2fa {
                            "Two-factor authentication required".to_string()
                        } else {
                            "Login successful".to_string()
                        },
                        display_name: if needs_2fa { None } else { Some(user.display_name) },
                        email: if needs_2fa { None } else { Some(user.email) },
                        requires_totp,
                        requires_email_2fa,
                        requires_phone_2fa,
                        totp_uri: None,
                    }))
                }
                None => Ok(AxumJson(LoginResponse {
                    success: false,
                    message: "Invalid username or password, or email not yet validated".to_string(),
                    display_name: None,
                    email: None,
                    requires_totp: false,
                    requires_email_2fa: false,
                    requires_phone_2fa: false,
                    totp_uri: None,
                })),
            }
        }

        // API: POST /api/verify_totp
        pub async fn api_verify_totp(
            axum::Json(req): axum::Json<VerifyTotpRequest>,
        ) -> Result<AxumJson<VerifyTotpResponse>, (StatusCode, String)> {
            match email::get_validated_user(&req.username).await {
                Some(user) if user.totp_enabled => {
                    if let Some(ref secret) = user.totp_secret {
                        if email::verify_totp_code(secret, &req.code) {
                            Ok(AxumJson(VerifyTotpResponse {
                                success: true,
                                message: "TOTP verified".to_string(),
                                display_name: Some(user.display_name),
                                email: Some(user.email),
                            }))
                        } else {
                            Ok(AxumJson(VerifyTotpResponse {
                                success: false,
                                message: "Invalid TOTP code".to_string(),
                                display_name: None,
                                email: None,
                            }))
                        }
                    } else {
                        Ok(AxumJson(VerifyTotpResponse {
                            success: false,
                            message: "TOTP not configured".to_string(),
                            display_name: None,
                            email: None,
                        }))
                    }
                }
                _ => Ok(AxumJson(VerifyTotpResponse {
                    success: false,
                    message: "User not found or TOTP not enabled".to_string(),
                    display_name: None,
                    email: None,
                })),
            }
        }

        // API: POST /api/verify_email_2fa
        pub async fn api_verify_email_2fa(
            axum::Json(req): axum::Json<VerifyEmail2faRequest>,
        ) -> Result<AxumJson<VerifyEmail2faResponse>, (StatusCode, String)> {
            let emails = email::get_pending_emails().await;
            let valid = emails.iter().any(|e| {
                e.username == req.username && e.validation_token.starts_with("2fa_") && e.body.contains(&req.code)
            });
            if valid {
                if let Some(user) = email::get_validated_user(&req.username).await {
                    Ok(AxumJson(VerifyEmail2faResponse {
                        success: true,
                        message: "Email 2FA verified".to_string(),
                        display_name: Some(user.display_name),
                        email: Some(user.email),
                    }))
                } else {
                    Ok(AxumJson(VerifyEmail2faResponse {
                        success: false,
                        message: "User not found".to_string(),
                        display_name: None,
                        email: None,
                    }))
                }
            } else {
                Ok(AxumJson(VerifyEmail2faResponse {
                    success: false,
                    message: "Invalid email 2FA code".to_string(),
                    display_name: None,
                    email: None,
                }))
            }
        }

        // API: POST /api/enable_totp
        pub async fn api_enable_totp(
            axum::Json(req): axum::Json<EnableTotpRequest>,
        ) -> Result<AxumJson<EnableTotpResponse>, (StatusCode, String)> {
            match email::check_validated_user(&req.username, &req.password).await {
                Some(user) => {
                    match email::generate_totp_secret(&user.username, "Red") {
                        Ok((secret, uri)) => Ok(AxumJson(EnableTotpResponse {
                            success: true,
                            message: "Scan the QR code or enter the secret in Google Authenticator".to_string(),
                            secret: Some(secret),
                            uri: Some(uri),
                        })),
                        Err(e) => Ok(AxumJson(EnableTotpResponse {
                            success: false,
                            message: e,
                            secret: None,
                            uri: None,
                        })),
                    }
                }
                None => Ok(AxumJson(EnableTotpResponse {
                    success: false,
                    message: "Invalid username or password".to_string(),
                    secret: None,
                    uri: None,
                })),
            }
        }

        // API: POST /api/confirm_totp
        pub async fn api_confirm_totp(
            axum::Json(req): axum::Json<ConfirmTotpRequest>,
        ) -> Result<AxumJson<ConfirmTotpResponse>, (StatusCode, String)> {
            if email::verify_totp_code(&req.secret, &req.code) {
                if let Some(mut user) = email::get_validated_user(&req.username).await {
                    user.totp_secret = Some(req.secret);
                    user.totp_enabled = true;
                    match email::update_user_2fa(user).await {
                        Ok(_) => Ok(AxumJson(ConfirmTotpResponse {
                            success: true,
                            message: "TOTP enabled".to_string(),
                        })),
                        Err(e) => Ok(AxumJson(ConfirmTotpResponse {
                            success: false,
                            message: e,
                        })),
                    }
                } else {
                    Ok(AxumJson(ConfirmTotpResponse {
                        success: false,
                        message: "User not found".to_string(),
                    }))
                }
            } else {
                Ok(AxumJson(ConfirmTotpResponse {
                    success: false,
                    message: "Invalid TOTP code".to_string(),
                }))
            }
        }

        // API: POST /api/toggle_email_2fa
        pub async fn api_toggle_email_2fa(
            axum::Json(req): axum::Json<ToggleEmail2faRequest>,
        ) -> Result<AxumJson<ToggleEmail2faResponse>, (StatusCode, String)> {
            match email::check_validated_user(&req.username, &req.password).await {
                Some(mut user) => {
                    user.email_2fa_enabled = req.enabled;
                    match email::update_user_2fa(user).await {
                        Ok(_) => Ok(AxumJson(ToggleEmail2faResponse {
                            success: true,
                            message: format!("Email 2FA {}", if req.enabled { "enabled" } else { "disabled" }),
                            enabled: req.enabled,
                        })),
                        Err(e) => Ok(AxumJson(ToggleEmail2faResponse {
                            success: false,
                            message: e,
                            enabled: !req.enabled,
                        })),
                    }
                }
                None => Ok(AxumJson(ToggleEmail2faResponse {
                    success: false,
                    message: "Invalid username or password".to_string(),
                    enabled: false,
                })),
            }
        }

        // API: POST /api/toggle_phone_2fa
        pub async fn api_toggle_phone_2fa(
            axum::Json(req): axum::Json<TogglePhone2faRequest>,
        ) -> Result<AxumJson<TogglePhone2faResponse>, (StatusCode, String)> {
            match email::check_validated_user(&req.username, &req.password).await {
                Some(mut user) => {
                    user.phone_2fa_enabled = req.enabled;
                    if req.enabled && !req.phone_number.trim().is_empty() {
                        user.phone_number = Some(req.phone_number.clone());
                    }
                    if !req.enabled {
                        user.phone_number = None;
                    }
                    match email::update_user_2fa(user).await {
                        Ok(_) => Ok(AxumJson(TogglePhone2faResponse {
                            success: true,
                            message: format!("Phone 2FA {}", if req.enabled { "enabled" } else { "disabled" }),
                            enabled: req.enabled,
                        })),
                        Err(e) => Ok(AxumJson(TogglePhone2faResponse {
                            success: false,
                            message: e,
                            enabled: !req.enabled,
                        })),
                    }
                }
                None => Ok(AxumJson(TogglePhone2faResponse {
                    success: false,
                    message: "Invalid username or password".to_string(),
                    enabled: false,
                })),
            }
        }

        // API: POST /api/verify_phone_2fa
        pub async fn api_verify_phone_2fa(
            axum::Json(req): axum::Json<VerifyPhone2faRequest>,
        ) -> Result<AxumJson<VerifyPhone2faResponse>, (StatusCode, String)> {
            let emails = email::get_pending_emails().await;
            let valid = emails.iter().any(|e| {
                e.username == req.username && e.validation_token.starts_with("phone2fa_") && e.body.contains(&req.code)
            });
            if valid {
                if let Some(user) = email::get_validated_user(&req.username).await {
                    Ok(AxumJson(VerifyPhone2faResponse {
                        success: true,
                        message: "Phone 2FA verified".to_string(),
                        display_name: Some(user.display_name),
                        email: Some(user.email),
                    }))
                } else {
                    Ok(AxumJson(VerifyPhone2faResponse {
                        success: false,
                        message: "User not found".to_string(),
                        display_name: None,
                        email: None,
                    }))
                }
            } else {
                Ok(AxumJson(VerifyPhone2faResponse {
                    success: false,
                    message: "Invalid phone 2FA code".to_string(),
                    display_name: None,
                    email: None,
                }))
            }
        }

        // API: POST /api/recover
        pub async fn api_recover(
            axum::Json(req): axum::Json<RecoverRequest>,
        ) -> Result<AxumJson<RecoverResponse>, (StatusCode, String)> {
            let email = req.email.trim().to_string();
            let phone = req.phone.trim().to_string();

            if email.is_empty() && phone.is_empty() {
                return Ok(AxumJson(RecoverResponse {
                    success: false,
                    message: "Please provide an email or phone number".to_string(),
                    username: None,
                    recovery_code: None,
                    passkey: None,
                    two_fa_code: None,
                }));
            }

            match email::find_validated_user_by_email_or_phone(&email, &phone).await {
                Some(mut user) => {
                    let passkey = email::generate_recovery_passkey();
                    let two_fa_code = email::generate_email_2fa_code();

                    let new_hash = match email::hash_password(&passkey) {
                        Ok(h) => h,
                        Err(e) => return Ok(AxumJson(RecoverResponse {
                            success: false,
                            message: e,
                            username: None,
                            recovery_code: None,
                            passkey: None,
                            two_fa_code: None,
                        })),
                    };

                    user.password_hash = new_hash;
                    user.email_2fa_enabled = true;
                    if !phone.is_empty() && user.phone_number.is_none() {
                        user.phone_number = Some(phone);
                    }

                    if let Err(e) = email::update_user_2fa(user.clone()).await {
                        return Ok(AxumJson(RecoverResponse {
                            success: false,
                            message: e,
                            username: None,
                            recovery_code: None,
                            passkey: None,
                            two_fa_code: None,
                        }));
                    }

                    email::enqueue_account_recovery_email(
                        user.email.clone(),
                        user.username.clone(),
                        passkey.clone(),
                        two_fa_code.clone(),
                    ).await;

                    Ok(AxumJson(RecoverResponse {
                        success: true,
                        message: format!(
                            "Recovery email sent to '{}'. Use the temporary passkey and 2FA code to log in.",
                            user.email
                        ),
                        username: Some(user.username),
                        recovery_code: Some(two_fa_code.clone()),
                        passkey: Some(passkey),
                        two_fa_code: Some(two_fa_code),
                    }))
                }
                None => {
                    Ok(AxumJson(RecoverResponse {
                        success: false,
                        message: "No account found with that email or phone".to_string(),
                        username: None,
                        recovery_code: None,
                        passkey: None,
                        two_fa_code: None,
                    }))
                }
            }
        }

        // API: GET /api/stats
        pub async fn api_stats() -> Result<AxumJson<serde_json::Value>, (StatusCode, String)> {
            Ok(AxumJson(serde_json::json!({
                "status": "ok",
                "users": 1,
            })))
        }

        // API: POST /api/credentials/sync
        pub async fn api_sync_credentials(
            axum::Json(req): axum::Json<SyncCredentialsRequest>,
        ) -> Result<AxumJson<SyncCredentialsResponse>, (StatusCode, String)> {
            let db = crate::storage::data_store();
            match db.save_credentials(&req.username, &req.credentials) {
                Ok(_) => Ok(AxumJson(SyncCredentialsResponse {
                    success: true,
                    message: "Credentials synced to cloud".to_string(),
                })),
                Err(e) => Ok(AxumJson(SyncCredentialsResponse {
                    success: false,
                    message: format!("Failed to sync credentials: {}", e),
                })),
            }
        }

        // API: GET /api/credentials/sync
        pub async fn api_load_credentials(
            Query(req): Query<LoadCredentialsRequest>,
        ) -> Result<AxumJson<LoadCredentialsResponse>, (StatusCode, String)> {
            let db = crate::storage::data_store();
            match db.load_credentials(&req.username) {
                Some(credentials) => Ok(AxumJson(LoadCredentialsResponse {
                    success: true,
                    message: "Credentials loaded from cloud".to_string(),
                    credentials: Some(credentials),
                })),
                None => Ok(AxumJson(LoadCredentialsResponse {
                    success: false,
                    message: "No cloud credentials found".to_string(),
                    credentials: None,
                })),
            }
        }
    }
}

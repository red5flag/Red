use cfg_if::cfg_if;
use leptos::prelude::*;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use axum::extract::Query;
        use axum::response::Html;
        use axum::Json as AxumJson;
        use axum::http::StatusCode;
        use serde::Deserialize;
        use crate::api::email;
        use crate::api::email::{SignupRequest, SignupResponse, ValidateRequest, ValidateResponse, LoginRequest, LoginResponse,
            VerifyTotpRequest, VerifyTotpResponse, VerifyEmail2faRequest, VerifyEmail2faResponse,
            EnableTotpRequest, EnableTotpResponse, ConfirmTotpRequest, ConfirmTotpResponse,
            ToggleEmail2faRequest, ToggleEmail2faResponse,
            SyncCredentialsRequest, SyncCredentialsResponse, LoadCredentialsRequest, LoadCredentialsResponse};

        #[derive(Deserialize)]
        pub struct EmailValidQuery {
            pub token: Option<String>,
        }

        pub async fn email_valid_page(Query(q): Query<EmailValidQuery>) -> Html<String> {
            let pending = email::get_pending_registrations().await;

            // Re-enqueue emails from pending registrations if queue is empty (e.g. after server restart)
            let emails = email::get_validation_emails().await;
            if emails.is_empty() && !pending.is_empty() {
                for p in &pending {
                    let email_obj = email::ValidationEmail::new(p.email.clone(), p.username.clone());
                    email::enqueue_email(email_obj).await;
                }
            }

            // If token is provided, validate the user
            let validation_result: Option<Result<String, String>> = if let Some(token) = &q.token {
                Some(email::validate_user_by_token(token).await
                    .map(|u| u.username)
                    .map_err(|e| e))
            } else {
                None
            };

            // Refresh emails after validation (may have removed the validated one)
            let emails = email::get_validation_emails().await;

            let mut html = String::from(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1"/>
    <title>Farley - Email Validation</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #1a1a2e; color: #e0e0e0; margin: 0; padding: 40px; max-width: 900px; }
        h1 { color: #64ffda; }
        h2 { color: #64ffda; margin-top: 32px; }
        .email-card { background: #16213e; border-radius: 8px; padding: 20px; margin: 16px 0; border: 1px solid #233; }
        .email-to { color: #64ffda; font-weight: bold; }
        .email-subject { font-size: 16px; margin: 8px 0; }
        .email-body { white-space: pre-wrap; background: #0f1623; padding: 16px; border-radius: 4px; font-family: monospace; font-size: 13px; color: #a0a0b0; }
        .email-token { color: #ffd166; font-size: 12px; }
        .email-timestamp { color: #888; font-size: 12px; }
        .validate-btn { display: inline-block; background: #64ffda; color: #1a1a2e; padding: 8px 16px; border-radius: 4px; text-decoration: none; font-weight: bold; margin-top: 8px; cursor: pointer; border: none; font-size: 14px; }
        .validate-btn:hover { opacity: 0.9; }
        .validated-banner { background: #1a3a2e; border: 2px solid #64ffda; border-radius: 8px; padding: 16px; margin: 16px 0; }
        .validated-banner h2 { margin: 0 0 8px 0; }
        .error-banner { background: #3a1a1a; border: 2px solid #ff6b6b; border-radius: 8px; padding: 16px; margin: 16px 0; }
        .error-banner h2 { color: #ff6b6b; margin: 0 0 8px 0; }
        .empty { color: #888; font-style: italic; }
        .pending-card { background: #16213e; border-radius: 8px; padding: 16px; margin: 8px 0; border: 1px solid #233; display: flex; justify-content: space-between; align-items: center; }
        .pending-info { flex: 1; }
        .pending-username { color: #64ffda; font-weight: bold; }
        .pending-email { color: #a0a0b0; font-size: 13px; }
        .tick-btn { background: #64ffda; color: #1a1a2e; border: none; border-radius: 50%; width: 36px; height: 36px; font-size: 18px; cursor: pointer; display: flex; align-items: center; justify-content: center; margin-left: 12px; text-decoration: none; }
        .tick-btn:hover { opacity: 0.9; }
        .info-box { background: #0f1623; border-radius: 8px; padding: 16px; margin: 16px 0; border: 1px solid #233; font-size: 13px; color: #a0a0b0; }
    </style>
</head>
<body>
    <h1>Farley Email Validation - Test Inbox</h1>
    <p>This page shows pending validation emails and registrations for testing purposes.</p>
"#);

            // Show validation result if token was provided
            if let Some(result) = validation_result {
                match result {
                    Ok(username) => {
                        html.push_str(&format!(
                            r#"<div class="validated-banner">
    <h2>&#9989; Email Validated Successfully!</h2>
    <p>User <strong>{}</strong> has been validated and can now sign in.</p>
    <a class="validate-btn" href="/">Go to Login</a>
</div>
"#, username));
                    }
                    Err(e) => {
                        html.push_str(&format!(
                            r#"<div class="error-banner">
    <h2>&#10060; Validation Failed</h2>
    <p>{}</p>
</div>
"#, e));
                    }
                }
            }

            // Show pending registrations with tick buttons
            if !pending.is_empty() {
                html.push_str(&format!("<h2>Pending Registrations ({})</h2>\n", pending.len()));
                html.push_str("<p>Click the &#10003; button to validate a user and allow them to sign in.</p>\n");
                for p in &pending {
                    html.push_str(&format!(
                        r#"<div class="pending-card">
    <div class="pending-info">
        <div class="pending-username">&#128100; {}</div>
        <div class="pending-email">&#128231; {}</div>
        <div class="email-token">Token: {}</div>
    </div>
    <a class="tick-btn" href="/emailvalid?token={}" title="Validate user">&#10003;</a>
</div>
"#,
                        p.username,
                        p.email,
                        p.validation_token,
                        p.validation_token
                    ));
                }
            }

            // Show emails
            if emails.is_empty() {
                html.push_str("<h2>Validation Emails</h2>\n");
                html.push_str("<p class=\"empty\">No validation emails.</p>\n");
            } else {
                html.push_str(&format!("<h2>Validation Emails ({})</h2>\n", emails.len()));
                for e in &emails {
                    html.push_str(&format!(
                        r#"<div class="email-card">
    <div class="email-to">To: {}</div>
    <div class="email-subject">{}</div>
    <div class="email-timestamp">{}</div>
    <div class="email-token">Token: {}</div>
    <div class="email-body">{}</div>
    <a class="validate-btn" href="/emailvalid?token={}">&#10003; Validate</a>
</div>
"#,
                        e.to,
                        e.subject,
                        e.timestamp,
                        e.validation_token,
                        e.body,
                        e.validation_token
                    ));
                }
            }

            html.push_str("<div class=\"info-box\">\n<p><strong>How it works:</strong></p>\n<ol>\n");
            html.push_str("<li>User signs up on the login page with username, email, and password</li>\n");
            html.push_str("<li>A validation email appears here with a &#10003; Validate button</li>\n");
            html.push_str("<li>Click &#10003; to validate the user - they can then sign in with their credentials</li>\n");
            html.push_str("</ol>\n</div>\n");

            html.push_str("</body>\n</html>");
            Html(html)
        }

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
                    message: "Account created! Check /emailvalid to validate your email.".to_string(),
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
                    if requires_email_2fa {
                        let code = email::generate_email_2fa_code();
                        email::enqueue_2fa_email(user.email.clone(), user.username.clone(), code).await;
                    }
                    Ok(AxumJson(LoginResponse {
                        success: !requires_totp && !requires_email_2fa,
                        message: if requires_totp || requires_email_2fa {
                            "Two-factor authentication required".to_string()
                        } else {
                            "Login successful".to_string()
                        },
                        display_name: if requires_totp || requires_email_2fa { None } else { Some(user.display_name) },
                        email: if requires_totp || requires_email_2fa { None } else { Some(user.email) },
                        requires_totp,
                        requires_email_2fa,
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
                    match email::generate_totp_secret(&user.username, "Farley") {
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

#[component]
pub fn EmailValidPage() -> impl IntoView {
    view! {
        <div class="email-valid-page">
            <h1>"Email Validation"</h1>
            <p>"Email validation is available at the /emailvalid endpoint on the server."</p>
        </div>
    }
}

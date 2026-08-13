//! Typed error hierarchy for the Red application core.
//!
//! These errors are designed to be:
//! - Safe to serialize across IPC (no secrets leaked)
//! - Ergonomic with `?` operator via `thiserror`
//! - Granular enough for the frontend to show appropriate UI

use thiserror::Error;

/// Top-level error type for all core operations.
#[derive(Debug, Clone, Error, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "detail")]
pub enum CoreError {
    #[error("storage error: {message}")]
    Storage { message: String },

    #[error("network error: {message}")]
    Network { message: String },

    #[error("document error: {message}")]
    Document { message: String },

    #[error("crypto error: {message}")]
    Crypto { message: String },

    #[error("transaction error: {message}")]
    Transaction { message: String },

    #[error("not found: {entity} with id {id}")]
    NotFound { entity: String, id: String },

    #[error("validation error: {message}")]
    Validation { message: String },

    #[error("migration error: {message}")]
    Migration { message: String },

    #[error("permission denied: {message}")]
    Permission { message: String },

    #[error("internal error: {message}")]
    Internal { message: String },
}

impl From<Box<dyn std::error::Error>> for CoreError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        CoreError::Internal {
            message: e.to_string(),
        }
    }
}

impl From<serde_json::Error> for CoreError {
    fn from(e: serde_json::Error) -> Self {
        CoreError::Internal {
            message: format!("serialization error: {}", e),
        }
    }
}

impl From<std::io::Error> for CoreError {
    fn from(e: std::io::Error) -> Self {
        CoreError::Storage {
            message: e.to_string(),
        }
    }
}

/// Convenience Result alias.
pub type CoreResult<T> = Result<T, CoreError>;

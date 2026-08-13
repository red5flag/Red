//! Service abstraction traits for web/Tauri compatibility.
//!
//! These traits allow the Leptos frontend to call the same logical operations
//! regardless of whether it's running in:
//! - Web mode (HTTP calls to Axum server functions)
//! - Tauri mode (IPC calls to native Rust core)
//!
//! The frontend calls trait methods; the implementation is selected at compile time
//! via `#[cfg]` or at runtime via platform detection.

use crate::ipc::{
    GetMessagesRequest, MarkChannelReadRequest, PageResponse, QueryAssetsRequest,
    SendMessageRequest, TransactionActionRequest,
};
use crate::CoreResult;
use serde_json::Value;

/// Asset query service — fetches bounded result sets of assets.
pub trait AssetService: Send + Sync {
    fn query_assets(&self, request: QueryAssetsRequest) -> CoreResult<PageResponse<Value>>;
}

/// Channel/message service — manages channels, messages, and unread state.
pub trait MessageService: Send + Sync {
    fn get_channels(&self) -> CoreResult<Vec<Value>>;
    fn get_messages(&self, request: GetMessagesRequest) -> CoreResult<Vec<Value>>;
    fn send_message(&self, request: SendMessageRequest) -> CoreResult<Value>;
    fn mark_channel_read(&self, request: MarkChannelReadRequest) -> CoreResult<()>;
}

/// Transaction service — client-side validation and state transitions.
/// Server-side verification remains authoritative.
pub trait TransactionService: Send + Sync {
    fn perform_action(&self, request: TransactionActionRequest) -> CoreResult<()>;
}

/// Storage service — simple key-value persistence.
pub trait StorageService: Send + Sync {
    fn get(&self, key: &str) -> CoreResult<Option<String>>;
    fn set(&self, key: &str, value: &str) -> CoreResult<()>;
    fn delete(&self, key: &str) -> CoreResult<()>;
}

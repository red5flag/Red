//! Typed IPC request/response structures for Tauri commands.
//!
//! These types define the contract between the Leptos frontend and the
//! native Rust core. Each structure is serializable and carries enough
//! type information to avoid ambiguity on either side.
//!
//! Design principles:
//! - Bounded result sets (pagination via cursors, not full dumps)
//! - Batch-friendly (one command processes many items)
//! - No raw binary in JSON (use filesystem paths or Tauri asset protocol)
//! - Explicit error propagation via CoreError

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─────────────────────────────────────────────────────
// Pagination
// ─────────────────────────────────────────────────────

/// Cursor-based pagination request.
/// The cursor is an opaque string that the native core interprets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRequest {
    /// Maximum number of items to return.
    pub limit: usize,
    /// Opaque cursor from a previous response, or None for the first page.
    pub cursor: Option<String>,
}

/// Cursor-based pagination response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageResponse<T> {
    pub items: Vec<T>,
    /// Cursor for the next page, or None if no more items.
    pub next_cursor: Option<String>,
    /// Total count if known, or None if expensive to compute.
    pub total: Option<usize>,
}

// ─────────────────────────────────────────────────────
// Asset queries
// ─────────────────────────────────────────────────────

/// Sort options for asset queries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssetSort {
    Recent,
    Oldest,
    HighestValue,
    LowestValue,
    ByName,
    ByGroup,
}

/// Filter parameters for asset queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetFilter {
    pub portfolio_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub asset_type: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
}

/// Request to query assets with filtering, sorting, and pagination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAssetsRequest {
    pub filter: AssetFilter,
    pub sort: AssetSort,
    pub ascending: bool,
    pub page: PageRequest,
}

// ─────────────────────────────────────────────────────
// Channel/Message service
// ─────────────────────────────────────────────────────

/// Request to fetch messages for a channel with pagination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMessagesRequest {
    pub channel_id: Uuid,
    /// Fetch messages before this timestamp (for backward pagination).
    pub before: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: usize,
}

/// Request to send a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub channel_id: Uuid,
    pub content: String,
    pub content_type: String,
    pub idempotency_key: Option<String>,
}

/// Request to mark a channel as read.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkChannelReadRequest {
    pub channel_id: Uuid,
    pub last_read_message_id: Option<Uuid>,
}

// ─────────────────────────────────────────────────────
// Transaction service
// ─────────────────────────────────────────────────────

/// Transaction state transitions that the native core can perform.
/// Server-side verification remains authoritative.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionAction {
    Submit,
    Approve,
    Reject,
    Withdraw,
    Execute,
    Lock,
}

/// Request to perform a transaction action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionActionRequest {
    pub transaction_id: Uuid,
    pub action: TransactionAction,
    pub comment: Option<String>,
    pub idempotency_key: Option<String>,
}

// ─────────────────────────────────────────────────────
// Storage / key-value
// ─────────────────────────────────────────────────────

/// Simple key-value get request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageGetRequest {
    pub key: String,
}

/// Simple key-value set request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSetRequest {
    pub key: String,
    pub value: String,
}

// ─────────────────────────────────────────────────────
// Document processing
// ─────────────────────────────────────────────────────

/// Request to convert images to PDF.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertImagesToPdfRequest {
    /// Filesystem paths to image files (native side reads them).
    pub image_paths: Vec<String>,
    pub output_path: String,
}

/// Request to generate a report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateReportRequest {
    pub report_type: String,
    pub portfolio_id: Option<Uuid>,
    pub format: ReportFormat,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReportFormat {
    Pdf,
    Html,
    Csv,
}

// ─────────────────────────────────────────────────────
// Network / sync
// ─────────────────────────────────────────────────────

/// Network connection state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionState {
    Connected,
    Disconnected,
    Connecting,
    Error,
}

/// Sync state for a given entity type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub entity: String,
    pub last_synced_at: Option<chrono::DateTime<chrono::Utc>>,
    pub pending_count: usize,
    pub connection_state: ConnectionState,
}

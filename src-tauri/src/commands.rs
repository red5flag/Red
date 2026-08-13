// Tauri IPC commands — typed, secure interfaces for the Leptos frontend
//
// These commands use red-core IPC types for type-safe communication.
// Each command is an explicit typed action — no generic "execute" interface.

use crate::state::AppState;
use red_core::ipc::{
    ConvertImagesToPdfRequest, GenerateReportRequest, GetMessagesRequest, MarkChannelReadRequest,
    PageResponse, QueryAssetsRequest, SendMessageRequest, StorageGetRequest, StorageSetRequest,
    TransactionActionRequest,
};
use red_core::{CoreError, CoreResult};
use red_storage::keys::TreeName;
use std::sync::Arc;
use tauri::State;

// ─────────────────────────────────────────────────────
// Health / metadata
// ─────────────────────────────────────────────────────

#[tauri::command]
pub fn ping() -> String {
    "pong".to_string()
}

#[derive(serde::Serialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub platform: String,
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        name: "Red".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
    }
}

// ─────────────────────────────────────────────────────
// Storage (simple key-value via sled)
// ─────────────────────────────────────────────────────

#[tauri::command]
pub fn storage_get(
    request: StorageGetRequest,
    state: State<'_, Arc<AppState>>,
) -> CoreResult<Option<String>> {
    let val = state.storage.get(TreeName::Meta, request.key.as_bytes())?;
    Ok(val.map(|v| String::from_utf8_lossy(&v).to_string()))
}

#[tauri::command]
pub fn storage_set(
    request: StorageSetRequest,
    state: State<'_, Arc<AppState>>,
) -> CoreResult<()> {
    state.storage.insert(
        TreeName::Meta,
        request.key.as_bytes(),
        request.value.as_bytes(),
    )
}

#[tauri::command]
pub fn storage_delete(key: String, state: State<'_, Arc<AppState>>) -> CoreResult<()> {
    state.storage.remove(TreeName::Meta, key.as_bytes())
}

// ─────────────────────────────────────────────────────
// Asset queries (stub — implemented in Milestone 4)
// ─────────────────────────────────────────────────────

#[tauri::command]
pub fn query_assets(
    _request: QueryAssetsRequest,
    _state: State<'_, Arc<AppState>>,
) -> CoreResult<PageResponse<serde_json::Value>> {
    // TODO M4: Implement native asset query with sled indexes
    Ok(PageResponse {
        items: Vec::new(),
        next_cursor: None,
        total: Some(0),
    })
}

// ─────────────────────────────────────────────────────
// Channel/Message service (stub — implemented in Milestone 5)
// ─────────────────────────────────────────────────────

#[tauri::command]
pub fn get_channels(_state: State<'_, Arc<AppState>>) -> CoreResult<Vec<serde_json::Value>> {
    // TODO M5: Implement channel list from sled
    Ok(Vec::new())
}

#[tauri::command]
pub fn get_messages(
    _request: GetMessagesRequest,
    _state: State<'_, Arc<AppState>>,
) -> CoreResult<Vec<serde_json::Value>> {
    // TODO M5: Implement message retrieval with ordered compound keys
    Ok(Vec::new())
}

#[tauri::command]
pub fn send_message(
    _request: SendMessageRequest,
    _state: State<'_, Arc<AppState>>,
) -> CoreResult<serde_json::Value> {
    // TODO M5: Implement message insertion with ordered key
    Err(CoreError::Internal {
        message: "send_message not yet implemented".to_string(),
    })
}

#[tauri::command]
pub fn mark_channel_read(
    _request: MarkChannelReadRequest,
    _state: State<'_, Arc<AppState>>,
) -> CoreResult<()> {
    // TODO M5: Implement unread state tracking
    Ok(())
}

// ─────────────────────────────────────────────────────
// Transaction service (stub — implemented in Milestone 8)
// ─────────────────────────────────────────────────────

#[tauri::command]
pub fn transaction_action(
    _request: TransactionActionRequest,
    _state: State<'_, Arc<AppState>>,
) -> CoreResult<()> {
    // TODO M8: Implement client-side validation + pending queue
    Err(CoreError::Internal {
        message: "transaction_action not yet implemented".to_string(),
    })
}

// ─────────────────────────────────────────────────────
// Document processing (stub — implemented in Milestone 7)
// ─────────────────────────────────────────────────────

#[tauri::command]
pub fn convert_images_to_pdf(
    _request: ConvertImagesToPdfRequest,
    _state: State<'_, Arc<AppState>>,
) -> CoreResult<String> {
    // TODO M7: Implement native PDF generation with printpdf
    Err(CoreError::Internal {
        message: "convert_images_to_pdf not yet implemented".to_string(),
    })
}

#[tauri::command]
pub fn generate_report(
    _request: GenerateReportRequest,
    _state: State<'_, Arc<AppState>>,
) -> CoreResult<String> {
    // TODO M7: Implement native report generation
    Err(CoreError::Internal {
        message: "generate_report not yet implemented".to_string(),
    })
}

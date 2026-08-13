use std::sync::Arc;

/// Portfolio and message persistence: sled (server/Tauri-side) + DashMap in-memory cache.
///
/// Only compiled when the `ssr` or `tauri` feature is active. The WASM/hydrate client
/// never touches this module – it reads state from the Leptos AppStore which
/// the server pre-populates via `load_portfolios` / `save_portfolio` and
/// `load_messages` / `save_message`.
#[cfg(any(feature = "ssr", feature = "tauri"))]
pub mod db;

#[cfg(any(feature = "ssr", feature = "tauri"))]
pub use db::{data_store, portfolio_store, DataStore};

#[cfg(any(feature = "ssr", feature = "tauri"))]
pub fn message_store() -> Arc<DataStore> {
    data_store()
}

use std::sync::Arc;

/// Portfolio and message persistence: RocksDB (server-side) + DashMap in-memory cache.
///
/// Only compiled when the `ssr` feature is active. The WASM/hydrate client
/// never touches this module – it reads state from the Leptos AppStore which
/// the server pre-populates via `load_portfolios` / `save_portfolio` and
/// `load_messages` / `save_message`.
#[cfg(feature = "ssr")]
pub mod db;

#[cfg(feature = "ssr")]
pub use db::{data_store, portfolio_store, DataStore};

#[cfg(feature = "ssr")]
pub fn message_store() -> Arc<DataStore> {
    data_store()
}

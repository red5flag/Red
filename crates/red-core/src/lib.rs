//! Red Core — shared domain models, typed IPC request/response structures,
//! and application-level error types.
//!
//! This crate is the shared foundation between:
//! - The Leptos frontend (farley lib)
//! - The Tauri native core (src-tauri)
//! - Future native service crates (red-storage, red-network, etc.)
//!
//! All types here are serializable (serde) and framework-agnostic — no Leptos,
//! no Tauri, no web-sys dependencies.

pub mod error;
pub mod ipc;
pub mod service;

pub use error::{CoreError, CoreResult};

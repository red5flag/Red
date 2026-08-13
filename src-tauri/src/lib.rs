// Red — Tauri 2 native application library
// This crate provides the native Rust core that runs behind the Leptos UI
// when Red is built as an Android/desktop app via Tauri 2.

use std::sync::Arc;
use tauri::Manager;

mod commands;
mod state;

pub use state::AppState;

/// Initialize the Tauri application state and register all IPC commands.
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Initialize native application state
            let state = AppState::new(app)?;
            app.manage(Arc::new(state));

            tracing::info!("Red Tauri application initialized");

            // Load the Leptos frontend
            let window = app.get_webview_window("main").expect("main window");
            let _ = window.eval(
                r#"
                if (typeof window.__LEPTOS_INIT === 'undefined') {
                    window.__LEPTOS_INIT = true;
                }
                "#,
            );

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::get_app_info,
            commands::storage_get,
            commands::storage_set,
            commands::storage_delete,
            commands::query_assets,
            commands::get_channels,
            commands::get_messages,
            commands::send_message,
            commands::mark_channel_read,
            commands::transaction_action,
            commands::convert_images_to_pdf,
            commands::generate_report,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Red Tauri application");
}

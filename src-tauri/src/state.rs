// Native application state managed by Tauri
use red_storage::Storage;
use std::path::PathBuf;
use std::sync::Arc;

pub struct AppState {
    /// sled-backed storage service (with schema migrations applied)
    pub storage: Arc<Storage>,
    /// Path to document storage directory
    pub docs_path: PathBuf,
}

impl AppState {
    pub fn new(app: &tauri::App) -> Result<Self, Box<dyn std::error::Error>> {
        let app_data_dir = app
            .path()
            .app_data_dir()
            .expect("failed to resolve app data dir");

        let db_path = app_data_dir.join("sled");
        let docs_path = app_data_dir.join("documents");

        std::fs::create_dir_all(&db_path)?;
        std::fs::create_dir_all(&docs_path)?;

        tracing::info!("Database path: {:?}", db_path);
        tracing::info!("Documents path: {:?}", docs_path);

        // Open storage with automatic migrations
        let storage = Storage::open(&db_path)?;

        Ok(Self { storage, docs_path })
    }
}

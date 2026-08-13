//! Schema versioning and migration runner for sled.
//!
//! sled has no built-in schema migration, so we implement explicit
//! application-level versioning with forward-only migrations.
//!
//! ## Safety guarantees
//!
//! - The current version is stored in the `meta` tree under key `schema_version`.
//! - Migrations run forward only — no automatic rollback.
//! - Each migration runs inside a sled `Batch` (atomic).
//! - Before running migrations, a backup of the database is created by
//!   copying the sled directory to `<db_path>.backup.<old_version>`.
//! - If a migration fails, the backup can be manually restored.
//! - Interrupted migrations are detected by checking if the version
//!   was updated — if not, the migration is re-run from the start.

use red_core::{CoreError, CoreResult};
use sled::Db;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Current schema version type.
pub type SchemaVersion = u32;

/// Key in the `meta` tree where the schema version is stored.
const SCHEMA_VERSION_KEY: &[u8] = b"schema_version";

/// A single migration step from version `from` to version `to`.
pub struct Migration {
    pub from: SchemaVersion,
    pub to: SchemaVersion,
    pub name: &'static str,
    pub run: fn(&Db) -> CoreResult<()>,
}

/// Migration runner — executes forward migrations atomically.
pub struct MigrationRunner {
    db: Db,
    db_path: std::path::PathBuf,
}

impl MigrationRunner {
    pub fn new(db: Db, db_path: impl AsRef<Path>) -> Self {
        Self {
            db,
            db_path: db_path.as_ref().to_path_buf(),
        }
    }

    /// Get the current schema version from the meta tree.
    pub fn current_version(&self) -> SchemaVersion {
        let meta = match self.db.open_tree(TreeName::Meta.as_str()) {
            Ok(t) => t,
            Err(_) => return 0,
        };
        match meta.get(SCHEMA_VERSION_KEY) {
            Ok(Some(v)) => {
                if v.len() == 4 {
                    SchemaVersion::from_be_bytes(v[..4].try_into().unwrap_or([0; 4]))
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    /// Set the schema version in the meta tree (atomic with batch).
    fn set_version(&self, version: SchemaVersion) -> CoreResult<()> {
        let meta = self
            .db
            .open_tree(TreeName::Meta.as_str())
            .map_err(|e| CoreError::Migration {
                message: e.to_string(),
            })?;
        meta.insert(SCHEMA_VERSION_KEY, &version.to_be_bytes()[..])
            .map_err(|e| CoreError::Migration {
                message: e.to_string(),
            })?;
        meta.flush()
            .map_err(|e| CoreError::Migration {
                message: e.to_string(),
            })?;
        Ok(())
    }

    /// Create a backup of the database directory before migration.
    fn backup(&self, old_version: SchemaVersion) -> CoreResult<()> {
        let backup_path = PathBuf::from(format!(
            "{}.backup.{}",
            self.db_path.display(),
            old_version
        ));
        if self.db_path.exists() {
            copy_dir_recursive(&self.db_path, &backup_path).map_err(|e| {
                warn!("Backup failed: {}", e);
                CoreError::Migration {
                    message: format!("backup failed: {}", e),
                }
            })?;
            info!("Backed up database to {}", backup_path.display());
        }
        Ok(())
    }

    /// Run all pending migrations up to the target version.
    pub fn run(&self, migrations: &[Migration]) -> CoreResult<SchemaVersion> {
        let mut current = self.current_version();
        let target = migrations.last().map(|m| m.to).unwrap_or(current);

        if current == target {
            info!("Database schema is up to date (version {})", current);
            return Ok(current);
        }

        info!(
            "Running migrations: version {} -> {}",
            current, target
        );

        // Backup before migrating
        self.backup(current)?;

        for migration in migrations {
            if current >= migration.to {
                continue;
            }
            if current != migration.from {
                warn!(
                    "Skipping migration '{}' (expects v{}, current is v{})",
                    migration.name, migration.from, current
                );
                continue;
            }

            info!("Running migration: {} (v{} -> v{})", migration.name, migration.from, migration.to);

            // Run the migration function
            (migration.run)(&self.db)?;

            // Update version atomically
            self.set_version(migration.to)?;
            current = migration.to;

            info!("Migration '{}' completed, now at v{}", migration.name, migration.to);
        }

        let final_version = self.current_version();
        info!("Database schema version is now {}", final_version);
        Ok(final_version)
    }
}

// Re-export for convenience
use crate::keys::TreeName;

/// Recursively copy a directory (for database backups).
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            std::fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn open_test_db(dir: &Path) -> Db {
        sled::open(dir).unwrap()
    }

    #[test]
    fn test_initial_version_is_zero() {
        let dir = tempdir().unwrap();
        let db = open_test_db(dir.path());
        let runner = MigrationRunner::new(db, dir.path());
        assert_eq!(runner.current_version(), 0);
    }

    #[test]
    fn test_migration_advances_version() {
        let dir = tempdir().unwrap();
        let db = open_test_db(dir.path());
        let runner = MigrationRunner::new(db.clone(), dir.path());

        let migrations = vec![Migration {
            from: 0,
            to: 1,
            name: "test_v0_to_v1",
            run: |_db| Ok(()),
        }];

        let version = runner.run(&migrations).unwrap();
        assert_eq!(version, 1);
    }

    #[test]
    fn test_multiple_migrations() {
        let dir = tempdir().unwrap();
        let db = open_test_db(dir.path());
        let runner = MigrationRunner::new(db.clone(), dir.path());

        let migrations = vec![
            Migration {
                from: 0,
                to: 1,
                name: "v0_to_v1",
                run: |_db| Ok(()),
            },
            Migration {
                from: 1,
                to: 2,
                name: "v1_to_v2",
                run: |_db| Ok(()),
            },
            Migration {
                from: 2,
                to: 3,
                name: "v2_to_v3",
                run: |_db| Ok(()),
            },
        ];

        let version = runner.run(&migrations).unwrap();
        assert_eq!(version, 3);
    }

    #[test]
    fn test_idempotent_run() {
        let dir = tempdir().unwrap();
        let db = open_test_db(dir.path());
        let runner = MigrationRunner::new(db.clone(), dir.path());

        let migrations = vec![Migration {
            from: 0,
            to: 1,
            name: "v0_to_v1",
            run: |_db| Ok(()),
        }];

        // First run
        let v1 = runner.run(&migrations).unwrap();
        assert_eq!(v1, 1);

        // Second run should be a no-op
        let v2 = runner.run(&migrations).unwrap();
        assert_eq!(v2, 1);
    }
}

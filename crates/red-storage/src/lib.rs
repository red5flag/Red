//! Red Storage — sled-backed local storage with schema versioning,
//! ordered compound keys, and efficient indexes.
//!
//! ## Tree Layout
//!
//! sled trees (column families) isolate different entity types:
//!
//! | Tree | Key Pattern | Purpose |
//! |------|-------------|---------|
//! | `meta` | `schema_version` | Storage version metadata |
//! | `portfolios` | `uuid` | Portfolio records |
//! | `assets` | `portfolio_id\|uuid` | Asset records, ordered by portfolio |
//! | `assets_by_group` | `group_id\|uuid` | Index: assets by group |
//! | `assets_by_status` | `status\|uuid` | Index: assets by status |
//! | `channels` | `uuid` | Channel records |
//! | `messages` | `channel_id\|timestamp\|message_id` | Messages, ordered for range queries |
//! | `messages_by_channel` | `channel_id\|message_id` | Secondary index for direct lookup |
//! | `transactions` | `uuid` | Transaction records |
//! | `transactions_by_status` | `status\|uuid` | Index: transactions by status |
//! | `documents` | `uuid` | Document metadata |
//! | `documents_by_asset` | `asset_id\|uuid` | Index: documents by asset |
//! | `notifications` | `uuid` | Notification records |
//! | `sync_state` | `entity_type` | Sync state per entity type |
//! | `pending_ops` | `timestamp\|uuid` | Pending operations queue, ordered by time |
//! | `credentials` | `username` | User credentials |
//!
//! ## Key Encoding
//!
//! Keys are binary-encoded for compactness and ordering:
//! - UUIDs: 16 bytes big-endian
//! - Timestamps: 8 bytes big-endian (for correct ordering)
//! - Compound keys: fields concatenated with separator bytes
//!
//! ## Message Key Design
//!
//! Message keys use `channel_id (16) | timestamp (8 BE) | message_id (16)`
//! so loading the latest N messages for a channel is a bounded reverse
//! range scan from `channel_id | MAX_TS | MAX_UUID` backwards.

pub mod keys;
pub mod migration;
pub mod store;

pub use keys::{MessageKey, TreeName};
pub use migration::{Migration, MigrationRunner, SchemaVersion};
pub use store::Storage;

//! Key encoding utilities for sled trees.
//!
//! Keys are binary-encoded for compactness and correct ordering.
//! All multi-field keys use big-endian encoding so lexicographic byte
//! ordering matches natural numeric ordering.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// sled tree names (column families).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TreeName {
    Meta,
    Portfolios,
    Assets,
    AssetsByGroup,
    AssetsByStatus,
    Channels,
    Messages,
    MessagesByChannel,
    Transactions,
    TransactionsByStatus,
    Documents,
    DocumentsByAsset,
    Notifications,
    SyncState,
    PendingOps,
    Credentials,
}

impl TreeName {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Meta => "meta",
            Self::Portfolios => "portfolios",
            Self::Assets => "assets",
            Self::AssetsByGroup => "assets_by_group",
            Self::AssetsByStatus => "assets_by_status",
            Self::Channels => "channels",
            Self::Messages => "messages",
            Self::MessagesByChannel => "messages_by_channel",
            Self::Transactions => "transactions",
            Self::TransactionsByStatus => "transactions_by_status",
            Self::Documents => "documents",
            Self::DocumentsByAsset => "documents_by_asset",
            Self::Notifications => "notifications",
            Self::SyncState => "sync_state",
            Self::PendingOps => "pending_ops",
            Self::Credentials => "credentials",
        }
    }
}

/// Encode a UUID as 16 big-endian bytes.
pub fn encode_uuid(id: Uuid) -> [u8; 16] {
    // UUID bytes are already in big-endian network order.
    // For Uuid v7 (timestamp-based), the first 8 bytes are a millisecond
    // timestamp in big-endian, which gives natural time ordering.
    *id.as_bytes()
}

/// Encode a timestamp as 8 big-endian bytes (milliseconds since epoch).
pub fn encode_timestamp(ts: DateTime<Utc>) -> [u8; 8] {
    let millis = ts.timestamp_millis();
    millis.to_be_bytes()
}

/// Maximum timestamp value (for range query upper bounds).
pub fn max_timestamp() -> [u8; 8] {
    i64::MAX.to_be_bytes()
}

/// Maximum UUID value (for range query upper bounds).
pub fn max_uuid() -> [u8; 16] {
    [0xFF; 16]
}

/// A compound key for messages: `channel_id | timestamp | message_id`.
///
/// This ordering allows efficient "latest N messages for channel X" queries
/// via a reverse range scan from `channel_id | MAX_TS | MAX_UUID` backwards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageKey {
    pub channel_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub message_id: Uuid,
}

impl MessageKey {
    pub fn new(channel_id: Uuid, timestamp: DateTime<Utc>, message_id: Uuid) -> Self {
        Self {
            channel_id,
            timestamp,
            message_id,
        }
    }

    /// Encode as bytes for sled: `channel_id (16) | timestamp_ms (8 BE) | message_id (16)`.
    pub fn encode(&self) -> Vec<u8> {
        let mut key = Vec::with_capacity(40);
        key.extend_from_slice(self.channel_id.as_bytes());
        key.extend_from_slice(&encode_timestamp(self.timestamp));
        key.extend_from_slice(self.message_id.as_bytes());
        key
    }

    /// Encode a prefix for a channel (all messages for this channel).
    pub fn channel_prefix(channel_id: Uuid) -> [u8; 16] {
        *channel_id.as_bytes()
    }

    /// Encode an upper bound for "latest messages before `before`" queries.
    pub fn channel_upper_bound(channel_id: Uuid, before: DateTime<Utc>) -> Vec<u8> {
        let mut key = Vec::with_capacity(40);
        key.extend_from_slice(channel_id.as_bytes());
        key.extend_from_slice(&encode_timestamp(before));
        key.extend_from_slice(&max_uuid());
        key
    }

    /// Encode the absolute upper bound for a channel (for "latest N" queries).
    pub fn channel_max_bound(channel_id: Uuid) -> Vec<u8> {
        let mut key = Vec::with_capacity(40);
        key.extend_from_slice(channel_id.as_bytes());
        key.extend_from_slice(&max_timestamp());
        key.extend_from_slice(&max_uuid());
        key
    }

    /// Decode from sled key bytes.
    pub fn decode(data: &[u8]) -> Option<Self> {
        if data.len() != 40 {
            return None;
        }
        let channel_id = Uuid::from_slice(&data[0..16]).ok()?;
        let ts_millis = i64::from_be_bytes(data[16..24].try_into().ok()?);
        let timestamp = DateTime::<Utc>::from_timestamp_millis(ts_millis)?;
        let message_id = Uuid::from_slice(&data[24..40]).ok()?;
        Some(Self {
            channel_id,
            timestamp,
            message_id,
        })
    }
}

/// Encode a compound key: `prefix_uuid | suffix_uuid`.
pub fn encode_compound_uuid(prefix: Uuid, suffix: Uuid) -> Vec<u8> {
    let mut key = Vec::with_capacity(32);
    key.extend_from_slice(prefix.as_bytes());
    key.extend_from_slice(suffix.as_bytes());
    key
}

/// Encode a compound key: `prefix_str | suffix_uuid`.
pub fn encode_str_uuid(prefix: &str, suffix: Uuid) -> Vec<u8> {
    let mut key = Vec::with_capacity(prefix.len() + 16);
    key.extend_from_slice(prefix.as_bytes());
    key.extend_from_slice(suffix.as_bytes());
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_key_roundtrip() {
        let channel_id = Uuid::new_v4();
        let timestamp = Utc::now();
        let message_id = Uuid::new_v4();
        let key = MessageKey::new(channel_id, timestamp, message_id);
        let encoded = key.encode();
        let decoded = MessageKey::decode(&encoded).unwrap();
        // Encoding truncates to millisecond precision, so compare at ms level
        assert_eq!(key.channel_id, decoded.channel_id);
        assert_eq!(
            key.timestamp.timestamp_millis(),
            decoded.timestamp.timestamp_millis()
        );
        assert_eq!(key.message_id, decoded.message_id);
    }

    #[test]
    fn test_message_key_ordering() {
        let channel_id = Uuid::new_v4();
        let id = Uuid::new_v4();
        let ts1 = Utc::now();
        let ts2 = ts1 + chrono::Duration::seconds(1);

        let key1 = MessageKey::new(channel_id, ts1, id).encode();
        let key2 = MessageKey::new(channel_id, ts2, id).encode();

        // Earlier timestamp should sort before later timestamp
        assert!(key1 < key2);
    }

    #[test]
    fn test_channel_prefix_is_prefix_of_full_key() {
        let channel_id = Uuid::new_v4();
        let prefix = MessageKey::channel_prefix(channel_id);
        let full_key = MessageKey::new(channel_id, Utc::now(), Uuid::new_v4()).encode();

        assert!(full_key.starts_with(&prefix));
    }

    #[test]
    fn test_max_bound_is_upper_bound() {
        let channel_id = Uuid::new_v4();
        let max_bound = MessageKey::channel_max_bound(channel_id);
        let any_key = MessageKey::new(channel_id, Utc::now(), Uuid::new_v4()).encode();

        assert!(any_key < max_bound);
    }
}

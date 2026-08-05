//! Messages sent from an untrusted client to the server.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Metadata required for every client message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientEnvelope {
    /// Wire protocol version used to encode the message.
    pub protocol_version: u16,
    /// Idempotency key for this command.
    pub command_id: Uuid,
    /// Last room sequence observed by the client, when available.
    pub room_sequence: Option<u64>,
    /// The player's intention.
    pub message: ClientMessage,
}

/// Client message variants will be added with the first room commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientMessage {}

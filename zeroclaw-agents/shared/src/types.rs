//! x402 message types for P2P agent communication.
//!
//! x402 is a lightweight agent-to-agent messaging protocol built on top of
//! standard WebSocket connections. Each message is JSON-encoded and signed
//! with the sender's Ethereum private key so recipients can verify origin.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Protocol version — bump when breaking changes land.
pub const X402_VERSION: &str = "x402/1.0";

/// Discriminator for all message types the protocol supports.
/// Extend this as new flows are added (e.g. TaskFailed, Heartbeat).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum X402MessageType {
    /// Requester → executor: "please do this work"
    TaskRequest,
    /// Executor → requester: "work is done, here's the result"
    TaskComplete,
    /// Any agent → any agent: generic notification
    Notification,
}

/// A signed x402 envelope.
///
/// Wire format (JSON):
/// ```json
/// {
///   "version": "x402/1.0",
///   "from":    "0xABCD…",
///   "to":      "0x1234…",
///   "message_id": "uuid-v4",
///   "timestamp":  1710000000,
///   "message_type": "TASK_REQUEST",
///   "payload": { … },
///   "signature": "0xhex…"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct X402Message {
    /// Protocol version string ("x402/1.0")
    pub version: String,
    /// Sender's checksummed Ethereum address ("0x…")
    pub from: String,
    /// Recipient's checksummed Ethereum address ("0x…")
    pub to: String,
    /// Unique message ID (UUID v4)
    pub message_id: String,
    /// Unix timestamp (seconds) at creation
    pub timestamp: u64,
    /// Discriminator for payload shape
    pub message_type: X402MessageType,
    /// Arbitrary JSON payload — shape depends on message_type
    pub payload: Value,
    /// EIP-191 personal_sign signature of the canonical message hash
    pub signature: String,
}

impl X402Message {
    /// Construct a new unsigned message (signature = "").
    /// Call `sign_message` from the crypto module to fill the signature.
    pub fn new_unsigned(
        from: impl Into<String>,
        to: impl Into<String>,
        message_type: X402MessageType,
        payload: Value,
    ) -> Self {
        use uuid::Uuid;
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            version: X402_VERSION.to_string(),
            from: from.into(),
            to: to.into(),
            message_id: Uuid::new_v4().to_string(),
            timestamp,
            message_type,
            payload,
            signature: String::new(),
        }
    }

    /// Serialize to JSON string for transmission.
    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    /// Deserialize from JSON string.
    pub fn from_json(s: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(s)?)
    }

    /// The canonical byte string that gets signed / verified.
    ///
    /// We commit to all fields *except* the signature itself so replay
    /// attacks across message IDs / timestamps are impossible.
    pub fn signing_bytes(&self) -> Vec<u8> {
        format!(
            "x402:{}:{}:{}:{}:{}:{:?}:{}",
            self.version,
            self.from,
            self.to,
            self.message_id,
            self.timestamp,
            self.message_type,
            self.payload
        )
        .into_bytes()
    }
}

// ── TaskRequest payload ────────────────────────────────────────────────────

/// Payload embedded in X402Message.payload for TaskRequest messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequestPayload {
    /// On-chain task ID (matches AgentRegistry.createTask taskId)
    pub task_id: u64,
    /// Short description of the work to be done
    pub description: String,
    /// Target URL for scraping tasks
    pub url: String,
    /// Address of the AgentRegistry contract managing the escrow
    pub registry_address: String,
}

// ── TaskComplete payload ───────────────────────────────────────────────────

/// Payload embedded in X402Message.payload for TaskComplete messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCompletePayload {
    /// Mirrors the task_id from the originating TaskRequest
    pub task_id: u64,
    /// Arbitrary result data — for scraping tasks, the scraped content
    pub result: String,
    /// Human-readable status ("success" | "error")
    pub status: String,
}

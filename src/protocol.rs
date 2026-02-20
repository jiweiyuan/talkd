use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── IPC Commands (CLI → Daemon) ─────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum Command {
    /// Join a gossip channel by name
    #[serde(rename = "join")]
    Join {
        channel: String,
        #[serde(default = "default_id")]
        client_id: String,
    },

    /// Send a message to a channel
    #[serde(rename = "send")]
    Send {
        channel: String,
        message: String,
        #[serde(default = "default_id")]
        client_id: String,
        #[serde(default)]
        file: Option<FileAttachment>,
    },

    /// Send a direct message to a NodeId
    #[serde(rename = "dm")]
    Dm {
        target: String, // NodeId hex
        message: String,
        #[serde(default = "default_id")]
        client_id: String,
        #[serde(default)]
        file: Option<FileAttachment>,
    },

    /// Read messages from a channel or DM inbox
    #[serde(rename = "read")]
    Read {
        channel: String,
        #[serde(default = "default_id")]
        client_id: String,
        #[serde(default)]
        wait: bool,
        #[serde(default)]
        timeout: Option<u64>,
    },

    /// Leave a channel
    #[serde(rename = "leave")]
    Leave {
        channel: String,
        #[serde(default = "default_id")]
        client_id: String,
    },

    /// Generate an invite ticket for a channel
    #[serde(rename = "invite")]
    Invite { channel: String },

    /// Join via an invite ticket
    #[serde(rename = "accept")]
    Accept {
        ticket: String,
        channel: String,
        #[serde(default = "default_id")]
        client_id: String,
    },

    /// List peers in a channel (with full NodeIds)
    #[serde(rename = "peers")]
    Peers { channel: String },

    /// Get node status
    #[serde(rename = "status")]
    Status,

    /// Get our address
    #[serde(rename = "id")]
    Address,

    /// Health check
    #[serde(rename = "ping")]
    Ping,

    /// Shutdown daemon
    #[serde(rename = "stop")]
    Stop,
}

fn default_id() -> String {
    "default".into()
}

// ── File Attachment ─────────────────────────────────────────────────

/// Max file size for inline gossip transfer (3MB — base64 expands to ~4MB)
pub const MAX_INLINE_FILE_SIZE: usize = 3 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttachment {
    /// Original filename
    pub name: String,
    /// File size in bytes (before encoding)
    pub size: usize,
    /// Base64-encoded file content
    pub data: String,
}

// ── IPC Responses (Daemon → CLI) ────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<Message>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivered: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<HashMap<String, ChannelInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daemon_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peers: Option<Vec<PeerInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: String,
    pub short: String,
}

impl Response {
    pub fn ok() -> Self {
        Self {
            ok: true,
            error: None,
            messages: None,
            delivered: None,
            channels: None,
            daemon_id: None,
            id: None,
            ticket: None,
            channel: None,
            peers: None,
        }
    }
    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            ok: false,
            error: Some(msg.into()),
            ..Self::ok()
        }
    }
}

// ── Message ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub from: String,
    pub data: String,
    pub ts: String,
    /// If this message has a file, the local path where it was saved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<FileRef>,
}

/// Reference to a received file (saved on disk).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRef {
    pub name: String,
    pub size: usize,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelInfo {
    pub peers: usize,
    pub subscribers: usize,
    pub buffered: usize,
    #[serde(default)]
    pub total: usize,
}

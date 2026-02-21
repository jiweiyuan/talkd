# Architecture

How talkd works under the hood.

## Overview

```
CLI (talkd)          Daemon                     Remote Daemon
┌──────────┐    ┌──────────────┐           ┌──────────────┐
│ command   │───►│ Unix socket  │           │ Unix socket  │◄─── remote CLI
│   args    │    │   (IPC)      │           │   (IPC)      │
└──────────┘    │              │           │              │
                │ iroh Endpoint│◄── P2P ──►│ iroh Endpoint│
                │ (QUIC+TLS1.3)│ encrypted │ (QUIC+TLS1.3)│
                │              │           │              │
                │ iroh-gossip  │◄─ gossip ─►│ iroh-gossip  │
                │  (pub/sub)   │           │  (pub/sub)   │
                └──────────────┘           └──────────────┘
```

Four layers:

1. **CLI** (`src/main.rs`) — parses commands, talks to the daemon over IPC
2. **Client** (`src/client.rs`) — manages daemon lifecycle, IPC transport
3. **Daemon** (`src/daemon.rs`) — long-running background process managing channels and P2P connections
4. **P2P** (iroh + iroh-gossip) — peer discovery via BitTorrent DHT (Pkarr), encrypted transport via QUIC + TLS 1.3

## The Daemon

The daemon is a background Rust process (Tokio async runtime) that:

- Listens on a Unix socket at `~/.talkd/daemon.sock` for CLI commands
- Maintains an iroh Endpoint for QUIC transport
- Runs iroh-gossip for channel-based pub/sub messaging
- Buffers incoming messages until they're read
- Persists messages and file attachments to disk
- Auto-starts on the first CLI command, runs until `talkd stop`

### Daemon Files

| File | Purpose |
|------|---------|
| `~/.talkd/daemon.sock` | Unix socket for CLI ↔ daemon communication |
| `~/.talkd/daemon.pid` | PID file for the daemon process |
| `~/.talkd/daemon.log` | Daemon log file (append-only, timestamped) |
| `~/.talkd/identity` | Ed25519 secret key (mode 0600) |
| `~/.talkd/contacts.json` | Saved contacts with names and notes |
| `~/.talkd/channels/` | Per-channel message history |
| `~/.talkd/attachments/` | Received file attachments |

### Auto-Start Mechanism

When any CLI command runs (`src/client.rs`):

1. Try to connect to existing daemon socket
2. Send a `ping` command to verify it's alive
3. If connection fails, spawn a new daemon as a detached child process (`talkd __daemon`)
4. Poll up to 50 times (200ms intervals = 10s max) until the daemon responds

## Identity and Cryptography

### Ed25519 Identity

Each agent has a persistent Ed25519 keypair:

- Generated on `talkd init` (or auto-created on first use)
- Stored at `~/.talkd/identity` (32-byte secret key, mode 0600)
- The public key = NodeId (64 hex chars), used for peer identification

### Channel Topic Derivation

A channel is identified by a deterministic topic hash:

```
topic = SHA-256("talkd:topic:" + channel_name)
```

This produces a 32-byte topic used by iroh-gossip for pub/sub. All agents using the same channel name derive the same topic.

### DM Topic Derivation (ECDH)

Direct messages use X25519 Diffie-Hellman to derive a private topic:

1. Each party converts their Ed25519 key to X25519
2. Both derive the same shared secret via ECDH
3. The shared secret becomes the gossip topic

Only the two parties can derive this topic — no one else can subscribe.

## Peer Connection Flow

1. Agent A calls `talkd create research` → daemon derives topic, subscribes via iroh-gossip, generates invite ticket
2. Agent A shares the ticket with Agent B
3. Agent B calls `talkd join <ticket>` → daemon extracts bootstrap info, subscribes to the same topic
4. iroh-gossip handles peer discovery and connection via BitTorrent DHT
5. QUIC + TLS 1.3 encrypts all P2P traffic
6. Both agents are now connected — messages flow via gossip broadcast

### Invite Tickets

An invite ticket encodes:
- The topic ID (channel identifier)
- Bootstrap peer addresses (so the joiner can find the network)

Tickets are base32-encoded and can be shared via any channel (clipboard, file, another message).

## Message Flow

```
Agent A                              Agent B (remote)
talkd send research "hello"
    │
    ▼
daemon A: broadcast via iroh-gossip
    ├─ gossip sends to all topic peers ──────►
    │                                    daemon B: receives gossip message
    │                                        │ deserialize WireMessage
    │                                        │ save to channel store
    │                                        │ notify local subscribers
    │                                        ▼
    │                                    talkd read research
    │                                        │ drains subscriber cursor
    │                                        ▼
    │                                    "[14:30:05] a1b2c3d4: hello"
    │
    └─ also delivered to other local subscribers (excludes sender)
```

### Wire Message Format

Messages sent over iroh-gossip use postcard serialization:

```rust
WireMessage {
    from: String,      // sender NodeId (short hex)
    data: String,      // message content
    ts: String,        // ISO 8601 timestamp
    file: Option<FileAttachment>,  // optional base64-encoded file
}
```

### File Attachments

- Files up to 3MB can be attached inline (base64-encoded, ~4MB on wire)
- Received files are saved to `~/.talkd/attachments/<channel>/YYYYMMDDTHHMMSS-<filename>`
- The `read` response includes a `FileRef` with the local path

## Storage Layout

```
~/.talkd/
├── identity              (32-byte Ed25519 secret key, mode 0600)
├── contacts.json         (contact list: name → {id, note})
├── daemon.sock           (Unix socket for IPC)
├── daemon.pid            (daemon process ID)
├── daemon.log            (debug logs)
├── channels/
│   ├── ch-<name>.json    (channel message history + cursors)
│   └── dm-<peer>.json    (DM conversation history + cursors)
└── attachments/
    ├── ch-<name>/        (channel file attachments)
    └── dm-<peer>/        (DM file attachments)
```

### Message Persistence

Unlike fire-and-forget systems, talkd persists messages to disk:
- Each channel has a JSON store with message history
- Read cursors track per-subscriber position
- Messages are available until the store is cleared

## IPC Protocol

CLI ↔ Daemon communication uses newline-delimited JSON over a Unix socket.

**Request:**
```json
{"action": "send", "channel": "research", "message": "hello", "client_id": "default"}
```

**Response:**
```json
{"ok": true, "delivered": 2}
```

**Error response:**
```json
{"ok": false, "error": "Not in channel: research"}
```

### Actions

| Action | Fields | Response |
|--------|--------|----------|
| `ping` | — | `{ok: true}` |
| `join` | `channel`, `client_id` | `{ok: true}` |
| `accept` | `ticket`, `channel`, `client_id` | `{ok: true, channel}` |
| `send` | `channel`, `message`, `client_id`, `file?` | `{ok: true, delivered: N}` |
| `dm` | `target`, `message`, `client_id`, `file?` | `{ok: true, delivered: N}` |
| `read` | `channel`, `client_id`, `wait?`, `timeout?` | `{ok: true, messages: [...]}` |
| `invite` | `channel` | `{ok: true, ticket: "..."}` |
| `peers` | `channel` | `{ok: true, peers: [...]}` |
| `leave` | `channel`, `client_id` | `{ok: true}` |
| `status` | — | `{ok: true, channels: {...}, daemon_id, id}` |
| `id` | — | `{ok: true, id: "..."}` |
| `stop` | — | `{ok: true}` (then exits) |

**Note:** `ping` is an internal health-check used by the auto-start mechanism. It is not exposed as a CLI command.

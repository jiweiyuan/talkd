# talkd

P2P communication for AI agents.
No server. No setup.
One CLI. One Skill. Your agents talk to any agent, anywhere.

```bash
cargo install talkd
```

<p align="center">
  <video src="https://talkd.vercel.app/demo.mp4" autoplay loop muted playsinline width="720"></video>
</p>

## What is this?

AI agents are isolated. When two agents need to collaborate, there's no simple way for them to talk directly. talkd gives them a direct line — create a channel, share a ticket, and they find each other automatically over the internet.

- **No server** — peer-to-peer via [iroh](https://iroh.computer) + Pkarr discovery
- **No setup** — single binary, two commands, agents are talking
- **Works anywhere** — same machine or different continents
- **Encrypted** — QUIC transport with TLS 1.3, secure by default
- **Agent-native** — CLI-first, `--json` output, stdin support
- **Direct messages** — DM any agent by ID, no channel needed
- **File sharing** — attach files up to 3MB to any message
- **Single binary** — zero runtime dependencies

## Quick start

```bash
talkd init
# Generated identity: a1b2c3d4...
```

**Agent A** creates a channel:
```bash
talkd create research
# Created channel "research"
# Invite ticket:
# kx32abc...
```

**Agent B** joins with the ticket:
```bash
talkd join kx32abc...
# Joined channel "research"
```

**Now they talk:**
```bash
# Agent A
talkd send research "analyze the dataset"

# Agent B
talkd read research
# [14:30:05] a1b2c3d4: analyze the dataset
```

## Commands

All commands support `--json` for machine-readable output.

### Identity

#### `talkd init`
Initialize identity. Generates a persistent Ed25519 keypair.

#### `talkd id`
Show your NodeId (64 hex chars). Share this with other agents for DMs.

### Channels

#### `talkd create <channel>`
Create a new channel and print an invite ticket.

#### `talkd join <ticket>`
Join a channel via invite ticket.

#### `talkd invite <channel>`
Generate a new invite ticket for an existing channel.

#### `talkd leave <channel>`
Leave a channel.

### Messaging

#### `talkd send <channel> [message] [-f file]`
Send a message to a channel. Reads from stdin if no message given.
```bash
talkd send research "task complete"
cat results.json | talkd send research
talkd send research "see attached" --file report.csv
```

#### `talkd read <channel> [-w] [-t N]`
Read pending messages. Use `-w` to wait, `-t` for timeout.
```bash
talkd read research                      # non-blocking
talkd read research --wait               # block until a message arrives
talkd read research --wait --timeout 60  # give up after 60 seconds
```

#### `talkd listen <channel>`
Stream messages continuously as they arrive.
```bash
talkd listen research --json
# {"from":"a1b2c3d4","data":"step 1 done","ts":"2026-02-21T14:30:05Z"}
# {"from":"a1b2c3d4","data":"step 2 done","ts":"2026-02-21T14:30:12Z"}
```

### Direct Messages

#### `talkd add <name> <id> [--note "desc"]`
Save a contact for easy reference.
```bash
talkd add alice a1b2c3d4e5f6... --note "research specialist"
```

#### `talkd contacts`
List all saved contacts.

#### `talkd dm <target> [message] [-f file]`
Send a direct message by contact name or NodeId. No channel needed.
```bash
talkd dm alice "hello"
talkd dm alice --file data.json
```

### Status

#### `talkd peers <channel>`
List all peers in a channel with their full NodeIds.

#### `talkd status`
Show active channels, peers, and unread messages.

#### `talkd stop`
Stop the background daemon.

## How it works

```
Agent A                          Agent B
┌────────────┐                   ┌────────────┐
│ talkd CLI  │                   │ talkd CLI  │
└─────┬──────┘                   └─────┬──────┘
      │ Unix Socket                    │ Unix Socket
┌─────┴──────┐                   ┌─────┴──────┐
│ talkd      │<══ iroh-gossip ══>│ talkd      │
│ daemon     │    (over QUIC)    │ daemon     │
└─────┬──────┘                   └─────┴──────┘
      │                                │
      └──── Pkarr / BT DHT ────────────┘
             (peer discovery)
```

1. `talkd create` derives a topic hash and subscribes via iroh-gossip, then generates an invite ticket
2. `talkd join` extracts bootstrap info from the ticket and subscribes to the same topic
3. Peers discover each other via Pkarr (BitTorrent mainline DHT) + iroh relay
4. Direct QUIC connection established, encrypted with TLS 1.3
5. Background daemon keeps connections alive, CLI commands are instant
6. DMs use ECDH (X25519) to derive a private topic — only the two parties can see messages

## Architecture

```
┌───────────────────────────────────────┐
│             talkd CLI                 │
├───────────────────────────────────────┤
│  IPC: Unix socket, JSON-line protocol │
├───────────────────────────────────────┤
│             talkd daemon              │
│  ┌─────────────────┐ ┌─────────────┐  │
│  │  iroh-gossip    │ │    Pkarr    │  │
│  │  (messaging)    │ │ (discovery) │  │
│  └────────┬────────┘ └──────┬──────┘  │
│  ┌────────┴─────────────────┴──────┐  │
│  │  iroh (QUIC transport + relay)  │  │
│  └─────────────────────────────────┘  │
└───────────────────────────────────────┘
          Rust · single binary
```

## License

MIT

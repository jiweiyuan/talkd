# talkd

P2P communication for AI agents. No server. No setup. Just talk.

```bash
cargo install talkd
```

## What is this?

AI agents are isolated. When two agents need to collaborate, there's no simple way for them to talk directly. talkd gives them a direct line — pick a channel, share a secret, and they find each other automatically over the internet.

- **No server** — peer-to-peer via QUIC + BitTorrent DHT
- **No setup** — single binary, two commands, agents are talking
- **Works anywhere** — same machine or different continents
- **Encrypted** — TLS 1.3 via QUIC, secure by default
- **Agent-native** — CLI-first, `--json` output, stdin support
- **6MB binary** — zero runtime dependencies

## Quick start

**Agent A:**
```bash
talkd join ops -s mysecret
talkd send ops "task complete, results ready"
```

**Agent B:**
```bash
talkd join ops -s mysecret
talkd read ops
# [14:30:05] alice: task complete, results ready
```

## Commands

```
talkd join <channel> [-s secret]           Join/create a channel
talkd send <channel> [message]             Send (or pipe from stdin)
talkd read <channel> [-w] [-t N]           Read pending messages
talkd listen <channel> [-s secret]         Stream messages (long-running)
talkd leave <channel>                      Leave a channel
talkd status                               Show channels & peers
talkd stop                                 Stop the daemon
```

All commands support `--json` for machine-readable output.

## Features over walkie

| Feature | walkie | talkd |
|---------|--------|-------|
| Runtime | Node.js + npm | Single 6MB binary |
| Protocol | Custom UDX | QUIC (RFC 9000) |
| Encryption | Noise | TLS 1.3 |
| Discovery | Hyperswarm DHT | BitTorrent mainline DHT |
| `--json` output | ❌ | ✅ All commands |
| `listen` (streaming) | ❌ | ✅ |
| stdin pipe | ❌ | ✅ `echo msg \| talkd send ch` |
| Secret required | Yes | Optional |
| `create` vs `join` | Two commands | Just `join` |
| Binary size | ~50MB (with node_modules) | 6MB |
| Memory | ~50-80MB | ~10MB |

## How it works

```
Agent A                          Agent B
┌────────────┐                   ┌────────────┐
│ talkd CLI  │                   │ talkd CLI  │
└─────┬──────┘                   └─────┬──────┘
      │ Unix Socket                    │ Unix Socket
┌─────▼──────┐                   ┌─────▼──────┐
│ talkd      │◄═══ QUIC/TLS ═══►│ talkd      │
│ daemon     │    encrypted P2P   │ daemon     │
└─────┬──────┘                   └─────┬──────┘
      │                                │
      └──── BitTorrent DHT ────────────┘
             (peer discovery)
```

1. Channel name + secret → SHA-256 topic hash → SHA-1 DHT info_hash
2. Each daemon announces on the BitTorrent mainline DHT
3. Peers discover each other via DHT `get_peers`
4. Direct QUIC connection with TLS 1.3 encryption
5. Background daemon keeps connections alive, CLI commands are instant

## Environment variables

| Variable | Description | Default |
|----------|-------------|---------|
| `TALKD_ID` | Sender identity name | auto-derived from terminal session |
| `TALKD_DIR` | Data directory | `~/.talkd` |

## Examples

### JSON output (for agents)
```bash
talkd read ops --json
# {"ok":true,"messages":[{"from":"alice","data":"done","ts":1708000000}]}
```

### Pipe from stdin
```bash
cat results.json | talkd send ops
echo "batch complete" | talkd send ops
```

### Stream messages
```bash
talkd listen ops -s secret --json
# {"from":"worker-1","data":"step 1 done","ts":1708000001}
# {"from":"worker-1","data":"step 2 done","ts":1708000005}
# ... (continuous output)
```

### Wait for response
```bash
talkd send ops "process /data/input.csv"
talkd read ops --wait --timeout 120
```

## Architecture

- **Rust** — single binary, zero runtime dependencies
- **QUIC** (quinn) — multiplexed encrypted transport, RFC 9000
- **BitTorrent DHT** (mainline) — decentralized peer discovery
- **Daemon** — background process manages P2P connections
- **IPC** — Unix socket, JSON-line protocol between CLI and daemon

## License

MIT

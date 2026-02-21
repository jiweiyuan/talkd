---
name: talkd
description: P2P communication between AI agents using the talkd CLI. Use when the user asks to set up agent-to-agent communication, create a channel, send/receive messages between agents, direct-message another agent, share files between agents, or enable real-time coordination between multiple AI agents. Triggers on "talkd", "agent communication", "talk to another agent", "set up a channel", "inter-agent messaging", "collaborate with", "coordinate with", "direct message", "dm an agent", "send file to agent".
allowed-tools: Bash(talkd:*)
---

# talkd — P2P Communication for AI Agents

No server. No setup. Just talk. Each agent has a persistent identity (Ed25519 keypair). Two agents communicate by sharing an invite ticket — no shared secrets needed.

## Quick start

Step 0. Initialize identity (once, on first use):
```bash
talkd init
```

Step 1. Create a channel or join one:
```bash
talkd create <channel>           # prints an invite ticket
talkd join <ticket>              # join using a ticket from another agent
```

Step 2. Send and read messages:
```bash
talkd send <channel> "your message"
talkd read <channel>                      # non-blocking, returns buffered messages
talkd read <channel> --wait               # blocks until a message arrives
talkd read <channel> --wait --timeout 60  # give up after N seconds
talkd listen <channel>                    # stream messages continuously
```

Step 3. Send a direct message (no channel needed):
```bash
talkd add alice <node-id>         # save a contact
talkd dm alice "hello"            # DM by contact name or full ID
```

Step 4. Clean up when done:
```bash
talkd leave <channel>
```

## Example

```bash
# Agent A
talkd create research
# Created channel "research"
# Invite ticket:
# kx32abc...

# Agent B (same machine or different continent)
talkd join kx32abc...
# Joined channel "research"

# Agent A sends task
talkd send research "analyze the dataset"

# Agent B reads
talkd read research
# [14:30:05] a1b2c3d4: analyze the dataset
```

## Behavior to know

- **Invite tickets**: `talkd create` prints a ticket. Share it with peers to let them join. Generate more with `talkd invite <channel>`
- **Identity**: Each agent has a persistent Ed25519 keypair at `~/.talkd/identity`. Run `talkd id` to see your NodeId
- **Direct messages**: Use `talkd dm <target> "msg"` to message a peer by NodeId or contact name — no channel setup needed
- **File attachments**: Add `--file path/to/file` to `send` or `dm` (max 3MB)
- **JSON output**: Add `--json` to any command for machine-readable output
- **`read` drains the buffer** — each message is returned only once
- Sender never sees their own messages in `read`
- Daemon auto-starts on first command, runs at `~/.talkd/`
- Debug logs: `~/.talkd/daemon.log`

## More

- [references/commands.md](references/commands.md) — full command reference
- [references/polling-patterns.md](references/polling-patterns.md) — polling strategies and patterns
- [references/architecture.md](references/architecture.md) — how the daemon and P2P layer work

# Polling Patterns

Strategies for AI agents to send and receive messages effectively with talkd.

## Non-Blocking Poll

Check for messages without waiting. Best for periodic checks between task steps.

```bash
talkd read <channel>
```

Returns immediately. If no messages: `No new messages`. Use this when you have other work to do and just want to check for updates.

## Blocking Wait

Block until a message arrives or timeout elapses.

```bash
talkd read <channel> --wait --timeout 30
```

Use this when you're idle and waiting for a specific response from another agent. The timeout prevents hanging indefinitely.

## Continuous Stream

Stream messages as they arrive in real time.

```bash
talkd listen <channel>
```

Blocks forever, printing each message immediately. Use when you need to react to every message as it comes in. Automatically reconnects if the daemon restarts.

## Pattern: Task Delegation

One agent sends a task, waits for the result.

```bash
# Coordinator
talkd send work-channel "process /data/input.csv"
talkd read work-channel --wait --timeout 120   # Wait up to 2 min for result

# Worker
talkd read work-channel --wait                 # Get assignment
# ... process ...
talkd send work-channel "result: 42 records processed, output at /tmp/out.csv"
```

## Pattern: Task Delegation with Files

Send a file as part of the task, receive a file back.

```bash
# Coordinator
talkd send work-channel "analyze this data" --file /tmp/input.csv
talkd read work-channel --wait --timeout 120

# Worker
talkd read work-channel --wait
# Got: "analyze this data" + file saved to ~/.talkd/attachments/ch-work-channel/...-input.csv
# ... process the file ...
talkd send work-channel "analysis complete" --file /tmp/results.json
```

## Pattern: Heartbeat / Keep-Alive

Periodic status updates so a coordinator knows workers are alive.

```bash
# Worker (every N steps)
talkd send status-channel "worker-1: alive, step 5/10, 50% done"

# Coordinator (poll periodically)
talkd read status-channel
```

## Pattern: Stop Signal

A coordinator can send a stop signal mid-task.

```bash
# Coordinator
talkd send task-channel "STOP"

# Worker (checks between steps)
MESSAGES=$(talkd read task-channel)
if echo "$MESSAGES" | grep -q "STOP"; then
  talkd send task-channel "acknowledged STOP, cleaning up"
  # ... cleanup ...
  exit 0
fi
```

## Pattern: Request-Response

Simulate synchronous request-response over the async channel.

```bash
# Requester
talkd send qa-channel "REQUEST: what is the current price of BTC?"
RESPONSE=$(talkd read qa-channel --wait --timeout 60)

# Responder
talkd read qa-channel --wait
# Got: "REQUEST: what is the current price of BTC?"
# ... look up answer ...
talkd send qa-channel "RESPONSE: BTC = $45,230"
```

## Pattern: Direct Message Coordination

Use DMs for private 1:1 coordination without a shared channel.

```bash
# Agent A knows Agent B's ID (or contact name)
talkd dm agent-b "Can you help with data analysis?"
talkd read dm:<agent-b-id> --wait --timeout 60

# Agent B
talkd read dm:<agent-a-id> --wait
# Got: "Can you help with data analysis?"
talkd dm agent-a "Sure, send me the data"
```

## Pattern: Fan-Out / Fan-In

One coordinator, multiple workers.

```bash
# Coordinator: fan out
talkd send dispatch "task:worker-1:analyze batch A"
talkd send dispatch "task:worker-2:analyze batch B"
talkd send dispatch "task:worker-3:analyze batch C"

# Each worker: read and filter
MESSAGES=$(talkd read dispatch)
# Parse for your task based on worker ID prefix

# Coordinator: fan in (collect results)
talkd read dispatch --wait --timeout 120
# Repeat reads until all workers report back
```

## Pattern: JSON Automation

Use `--json` for structured message processing in scripts.

```bash
# Read messages as JSON for programmatic parsing
RESULT=$(talkd read work-channel --wait --timeout 30 --json)

# Parse with jq
echo "$RESULT" | jq -r '.messages[].data'

# Check delivery status
SENT=$(talkd send work-channel "hello" --json)
DELIVERED=$(echo "$SENT" | jq '.delivered')
if [ "$DELIVERED" = "0" ]; then
  echo "Warning: message not delivered to any peer"
fi
```

## Tips

- **Non-blocking reads are cheap** — call `talkd read` liberally between steps
- **Buffer awareness** — messages accumulate while you're not reading; a single `read` returns all pending messages
- **One read = drain** — `talkd read` returns all buffered messages and clears them; you won't see them again
- **Use `listen` for real-time** — when you need to react to every message, `listen` is more efficient than polling with `read --wait`
- **`--json` for automation** — all commands support `--json` for reliable machine parsing
- **File attachments** — use `--file` to send data files directly instead of encoding them in the message text
- **DMs for private communication** — use `talkd dm` for 1:1 messages that don't need a shared channel
- **Timeout padding** — the CLI adds 5 seconds to the `--wait` timeout internally for IPC overhead, so the actual wait duration matches what you specify

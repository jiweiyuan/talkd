#!/usr/bin/env bash
# Delegate a task to a remote agent and wait for the result.
#
# Usage: ./delegated-task.sh <channel> "task to delegate" [timeout_seconds]
#
# Example:
#   ./delegated-task.sh research "summarize the latest news on AI" 120

set -euo pipefail

CHANNEL="${1:?Channel name required}"
TASK="${2:?Task description required}"
TIMEOUT="${3:-60}"

# Create channel and get invite ticket
TICKET=$(talkd create "$CHANNEL" --json | jq -r '.ticket')
echo "Channel created. Share this ticket with the worker:"
echo "$TICKET"
echo ""
echo "Waiting for peer to connect..."

# Poll for peer connection
for i in $(seq 1 30); do
  STATUS=$(talkd status --json 2>/dev/null || true)
  PEERS=$(echo "$STATUS" | jq -r ".channels.\"$CHANNEL\".peers // 0")
  if [ "$PEERS" -gt 0 ] 2>/dev/null; then
    echo "Worker connected."
    break
  fi
  sleep 1
done

# Send the task
talkd send "$CHANNEL" "$TASK"
echo "Task delegated: $TASK"

# Wait for result
echo "Waiting up to ${TIMEOUT}s for result..."
RESULT=$(talkd read "$CHANNEL" --wait --timeout "$TIMEOUT")
echo "Result received:"
echo "$RESULT"

# Cleanup
talkd leave "$CHANNEL"

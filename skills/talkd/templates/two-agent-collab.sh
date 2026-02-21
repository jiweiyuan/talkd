#!/usr/bin/env bash
# Two-agent collaboration: coordinator sends a task, worker executes and reports back.
#
# For same-machine usage, set TALKD_ID to give each agent a unique identity.
# For cross-machine usage, TALKD_ID is optional (each machine has its own daemon).
#
# Usage:
#   Coordinator: ./two-agent-collab.sh coordinator <channel> "task description"
#   Worker:      ./two-agent-collab.sh worker <ticket>
#
# Same-machine example:
#   TALKD_ID=coordinator ./two-agent-collab.sh coordinator research "analyze data"
#   # (copy the printed ticket)
#   TALKD_ID=worker ./two-agent-collab.sh worker <ticket>

set -euo pipefail

ROLE="${1:?Usage: $0 <coordinator|worker> <channel|ticket> [task]}"

case "$ROLE" in
  coordinator)
    CHANNEL="${2:?Channel name required}"
    TASK="${3:?Task description required for coordinator}"
    talkd create "$CHANNEL"
    echo ""
    echo "Share the ticket above with the worker, then waiting for peer..."
    sleep 5  # Allow time for peer discovery
    talkd send "$CHANNEL" "$TASK"
    echo "Task sent. Waiting for result..."
    talkd read "$CHANNEL" --wait --timeout 120
    talkd leave "$CHANNEL"
    ;;
  worker)
    TICKET="${2:?Invite ticket required}"
    talkd join "$TICKET"
    echo "Joined channel. Waiting for task..."
    TASK=$(talkd read "$(talkd status --json | jq -r '.channels | keys[0]')" --wait --timeout 60)
    echo "Received task: $TASK"
    echo "--- Execute your work here ---"
    CHANNEL=$(talkd status --json | jq -r '.channels | keys[0]')
    talkd send "$CHANNEL" "done: task completed successfully"
    talkd leave "$CHANNEL"
    ;;
  *)
    echo "Unknown role: $ROLE (use 'coordinator' or 'worker')"
    exit 1
    ;;
esac

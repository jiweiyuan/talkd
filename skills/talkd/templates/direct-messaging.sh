#!/usr/bin/env bash
# Direct messaging between two agents without a shared channel.
#
# Agent A and Agent B exchange IDs and can DM each other directly.
# No channel creation or invite tickets needed.
#
# Usage:
#   Agent A: ./direct-messaging.sh send <contact-name-or-id> "message"
#   Agent A: ./direct-messaging.sh receive <contact-name-or-id> [timeout]
#   Setup:   ./direct-messaging.sh setup <name> <id> [note]
#
# Example:
#   # First, exchange IDs:
#   talkd id   # share this with the other agent
#
#   # Save the other agent as a contact:
#   ./direct-messaging.sh setup researcher abc123... "research specialist"
#
#   # Send a DM:
#   ./direct-messaging.sh send researcher "Can you analyze this dataset?"
#
#   # Wait for reply:
#   ./direct-messaging.sh receive researcher 60

set -euo pipefail

ACTION="${1:?Usage: $0 <send|receive|setup> ...}"

case "$ACTION" in
  setup)
    NAME="${2:?Contact name required}"
    ID="${3:?Contact ID required (64 hex chars from 'talkd id')}"
    NOTE="${4:-}"
    if [ -n "$NOTE" ]; then
      talkd add "$NAME" "$ID" --note "$NOTE"
    else
      talkd add "$NAME" "$ID"
    fi
    ;;
  send)
    TARGET="${2:?Target (contact name or ID) required}"
    MESSAGE="${3:?Message required}"
    talkd dm "$TARGET" "$MESSAGE"
    ;;
  receive)
    TARGET="${2:?Target (contact name or ID) required}"
    TIMEOUT="${3:-30}"
    talkd read "dm:$TARGET" --wait --timeout "$TIMEOUT"
    ;;
  *)
    echo "Unknown action: $ACTION (use 'setup', 'send', or 'receive')"
    exit 1
    ;;
esac

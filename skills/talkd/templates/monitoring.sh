#!/usr/bin/env bash
# Monitor agent activity on a channel from a separate terminal.
# Uses talkd listen for real-time streaming.
#
# Usage: ./monitoring.sh <ticket>
#
# Example:
#   ./monitoring.sh kx32abc...

set -euo pipefail

TICKET="${1:?Invite ticket required}"

export TALKD_ID=monitor
talkd join "$TICKET"
CHANNEL=$(talkd status --json | jq -r '.channels | keys[0]')
echo "Monitoring channel: $CHANNEL"
echo "Press Ctrl+C to stop"
echo "---"

trap 'echo ""; echo "Leaving channel..."; talkd leave "$CHANNEL"; exit 0' INT TERM

# Stream messages in real time
talkd listen "$CHANNEL"

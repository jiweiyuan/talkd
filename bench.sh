#!/usr/bin/env bash
# talkd latency benchmark — single-node IPC measurements
set -euo pipefail

CHANNEL="bench-$$"
ROUNDS=20
RESULTS=()

fmt_ms() {
  # input: nanoseconds, output: milliseconds with 1 decimal
  echo "scale=1; $1 / 1000000" | bc
}

time_cmd() {
  local start end
  start=$(date +%s%N 2>/dev/null || python3 -c 'import time; print(int(time.time()*1e9))')
  eval "$@" > /dev/null 2>&1
  end=$(date +%s%N 2>/dev/null || python3 -c 'import time; print(int(time.time()*1e9))')
  echo $(( end - start ))
}

stats() {
  local label="$1"; shift
  local vals=("$@")
  local n=${#vals[@]}
  # sort
  IFS=$'\n' sorted=($(sort -n <<<"${vals[*]}")); unset IFS
  local min=${sorted[0]}
  local max=${sorted[$((n-1))]}
  local p50=${sorted[$((n/2))]}
  local sum=0
  for v in "${vals[@]}"; do sum=$((sum + v)); done
  local avg=$((sum / n))
  printf "  %-28s  min=%6s  avg=%6s  p50=%6s  max=%6s ms  (n=%d)\n" \
    "$label" "$(fmt_ms $min)" "$(fmt_ms $avg)" "$(fmt_ms $p50)" "$(fmt_ms $max)" "$n"
}

echo "═══════════════════════════════════════════════════"
echo "  talkd latency benchmark (single-node IPC)"
echo "═══════════════════════════════════════════════════"
echo ""

# ── 1. Cold start (daemon not running) ──────────────────────────────
echo "▸ Cold start (daemon spawn + first command)..."
talkd stop > /dev/null 2>&1 || true
sleep 0.5
cold_ns=$(time_cmd "talkd status --json")
echo "  Cold start:                   $(fmt_ms $cold_ns) ms"
echo ""

# ── 2. Warm command latency ──────────────────────────────────────────
echo "▸ Warm commands (daemon already running, $ROUNDS rounds)..."

# status
vals=()
for i in $(seq 1 $ROUNDS); do
  ns=$(time_cmd "talkd status --json")
  vals+=($ns)
done
stats "status" "${vals[@]}"

# id
vals=()
for i in $(seq 1 $ROUNDS); do
  ns=$(time_cmd "talkd id --json")
  vals+=($ns)
done
stats "id" "${vals[@]}"

echo ""

# ── 3. Channel join ─────────────────────────────────────────────────
echo "▸ Channel create..."
create_ns=$(time_cmd "talkd create $CHANNEL --json")
echo "  create:                       $(fmt_ms $create_ns) ms"
echo ""

# ── 4. Send latency ─────────────────────────────────────────────────
echo "▸ Send ($ROUNDS rounds)..."
vals=()
for i in $(seq 1 $ROUNDS); do
  ns=$(time_cmd "talkd send $CHANNEL 'bench message $i' --json")
  vals+=($ns)
done
stats "send" "${vals[@]}"
echo ""

# ── 5. Read latency ─────────────────────────────────────────────────
echo "▸ Read ($ROUNDS rounds)..."
vals=()
for i in $(seq 1 $ROUNDS); do
  ns=$(time_cmd "talkd read $CHANNEL --json")
  vals+=($ns)
done
stats "read" "${vals[@]}"
echo ""

# ── 6. Send→Read round-trip ─────────────────────────────────────────
echo "▸ Send→Read round-trip ($ROUNDS rounds)..."
vals=()
for i in $(seq 1 $ROUNDS); do
  # drain first
  talkd read $CHANNEL --json > /dev/null 2>&1 || true
  start=$(date +%s%N 2>/dev/null || python3 -c 'import time; print(int(time.time()*1e9))')
  talkd send $CHANNEL "roundtrip-$i" --json > /dev/null 2>&1
  talkd read $CHANNEL --json > /dev/null 2>&1
  end=$(date +%s%N 2>/dev/null || python3 -c 'import time; print(int(time.time()*1e9))')
  vals+=($((end - start)))
done
stats "send+read" "${vals[@]}"
echo ""

# ── 7. Message sizes ────────────────────────────────────────────────
echo "▸ Send latency by message size..."
for size in 10 100 1000 10000; do
  msg=$(head -c $size /dev/urandom | base64 | head -c $size)
  vals=()
  for i in $(seq 1 10); do
    ns=$(time_cmd "talkd send $CHANNEL '$msg' --json")
    vals+=($ns)
  done
  stats "send ${size}B" "${vals[@]}"
done
echo ""

# ── 8. Throughput burst ─────────────────────────────────────────────
echo "▸ Burst throughput (100 sends)..."
start=$(date +%s%N 2>/dev/null || python3 -c 'import time; print(int(time.time()*1e9))')
for i in $(seq 1 100); do
  talkd send $CHANNEL "burst-$i" --json > /dev/null 2>&1
done
end=$(date +%s%N 2>/dev/null || python3 -c 'import time; print(int(time.time()*1e9))')
burst_ms=$(fmt_ms $((end - start)))
rate=$(echo "scale=1; 100000 / $burst_ms" | bc)
echo "  100 sends in ${burst_ms}ms  (${rate} msg/s)"
echo ""

# ── Cleanup ──────────────────────────────────────────────────────────
talkd leave $CHANNEL > /dev/null 2>&1 || true

echo "═══════════════════════════════════════════════════"
echo "  Done. All times = CLI process spawn + IPC + daemon processing"
echo "═══════════════════════════════════════════════════"

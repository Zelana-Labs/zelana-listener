#!/usr/bin/env bash
set -euo pipefail

# Duration in milliseconds each listener runs before being stopped
# 15 secs
DURATION_MS="${DURATION_MS:-10000}"

# Helper: millisecond sleep
sleep_ms() {
  local ms="$1"
  local s
  s="$(awk "BEGIN{printf \"%.3f\", ${ms}/1000}")"
  sleep "$s"
}

# Helper: clean kill
easy_kill() {
  local pid="$1"
  kill -TERM "$pid" 2>/dev/null || true
  wait "$pid" 2>/dev/null || true
}

run_listener() {
  local label="$1"
  local dir="$2"
  local cmd="$3"

  echo ""
  echo "=== $label ==="
  echo "→ entering: $dir"
  cd "$dir"

  echo "→ starting: $cmd"
  bash -lc "$cmd" &
  local pid=$!

  echo "→ running for ${DURATION_MS} ms..."
  sleep_ms "$DURATION_MS"

  echo "→ stopping: $label"
  easy_kill "$pid"

  echo "→ returning to root"
  cd - >/dev/null
}

echo "Starting listeners sequentially…"

# --- TypeScript ---
run_listener "ts_helius_http" "ts" "npm run helius:http"
run_listener "ts_helius_wss" "ts" "npm run helius:wss"
run_listener "ts_native"     "ts" "npm run native"

# --- Rust ---
#run_listener "rust_helius" "rust" "./target/debug/helius"
#run_listener "rust_native" "rust" "./target/debug/native"

echo ""
echo "All done ✅"

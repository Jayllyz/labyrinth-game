#!/bin/bash
# Plays two client teams against our own server and fails unless every team escapes.
#
# Usage: e2e/own-server.sh [SEED] [TIMEOUT_SECONDS]
set -euo pipefail

SEED=${1:-1}
TIMEOUT=${2:-120}
PORT=${PORT:-8779}
TEAMS=(Alpha Beta)

ROOT=$(cd "$(dirname "$0")/.." && pwd)
cargo build -q --release -p server -p client --manifest-path "$ROOT/Cargo.toml"
LOG=$(mktemp -t labyrinth-own-server.XXXXXX)

"$ROOT/target/release/server" --port "$PORT" --seed "$SEED" --maze 20,20 > "$LOG" 2>&1 &
SERVER_PID=$!
trap 'kill $SERVER_PID 2>/dev/null || true' EXIT
for _ in $(seq 50); do
  nc -z localhost "$PORT" 2>/dev/null && break
  sleep 0.1
done

CLIENT_PIDS=()
# One seed per team: with the same seed, two opponents blocking each other step aside in
# lockstep and never get past.
for i in "${!TEAMS[@]}"; do
  timeout "$TIMEOUT" "$ROOT/target/release/client" --port "$PORT" --team "${TEAMS[$i]}" \
    --seed "$((SEED + i))" > /dev/null 2>&1 &
  CLIENT_PIDS+=($!)
done
wait "${CLIENT_PIDS[@]}" || true

cat "$LOG"
for team in "${TEAMS[@]}"; do
  grep -q "Team $team escaped" "$LOG" || { echo "Team $team did not escape" >&2; exit 1; }
done

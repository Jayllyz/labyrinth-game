#!/bin/bash
# Runs one team per algorithm (Tremeaux, WallFollower, Alian) against a local server.
#
# Usage: e2e/multi.sh [WIDTH,HEIGHT] [TIMEOUT_SECONDS]
#   e2e/multi.sh            # server's default maze, 120 s
#   e2e/multi.sh 20,20 60
#   TUI=Alian e2e/multi.sh   # show that team's TUI, others run hidden
set -euo pipefail

MAZE=${1:-}
TIMEOUT=${2:-120}
PORT=${PORT:-8778}
TUI=${TUI:-}
ALGORITHMS=(Tremeaux WallFollower Alian)

if [[ -n "$TUI" && ! " ${ALGORITHMS[*]} " =~ " $TUI " ]]; then
  echo "TUI must be one of: ${ALGORITHMS[*]}" >&2
  exit 1
fi

ROOT=$(cd "$(dirname "$0")/.." && pwd)
if [[ "$(uname -s)-$(uname -m)" == "Darwin-arm64" ]]; then
  SERVER="$ROOT/e2e/server-arm"
else
  SERVER="$ROOT/e2e/server"
fi
LOG=$(mktemp -t labyrinth-server.XXXXXX)

cargo build -q -p client --manifest-path "$ROOT/Cargo.toml"
CLIENT="$ROOT/target/debug/client"

SERVER_ARGS=(run --port "$PORT" --gui websocket)
[[ -n "$MAZE" ]] && SERVER_ARGS+=(--maze "$MAZE")
"$SERVER" "${SERVER_ARGS[@]}" > "$LOG" 2>&1 &
SERVER_PID=$!
trap 'kill $SERVER_PID 2>/dev/null || true' EXIT
sleep 1

echo "Maze ${MAZE:-default}, timeout ${TIMEOUT}s, server log: $LOG"
CLIENT_PIDS=()
for algorithm in "${ALGORITHMS[@]}"; do
  [[ "$algorithm" == "$TUI" ]] && continue
  timeout "$TIMEOUT" "$CLIENT" --port "$PORT" --team "$algorithm" --algorithm "$algorithm" \
    > /dev/null 2>&1 &
  CLIENT_PIDS+=($!)
done
if [[ -n "$TUI" ]]; then
  # --foreground keeps the TUI in the terminal's process group so it can read keys.
  timeout --foreground "$TIMEOUT" "$CLIENT" --port "$PORT" --team "$TUI" --algorithm "$TUI" --tui \
    || true
fi
wait "${CLIENT_PIDS[@]}" || true

TEAM_SIZE=3
for algorithm in "${ALGORITHMS[@]}"; do
  exits=$(grep -c "$algorithm/Player[0-9]* found the exit" "$LOG" || true)
  echo "$algorithm: $exits/$TEAM_SIZE agents found the exit"
done

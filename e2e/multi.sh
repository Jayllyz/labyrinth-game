#!/bin/bash
# Runs one team per algorithm (Tremeaux, WallFollower, Alian) against a local server.
#
# Usage: e2e/multi.sh [WIDTH,HEIGHT] [TIMEOUT_SECONDS] [MOVE_DELAY_MS]
#   e2e/multi.sh              # server's default maze, 120 s, 100 ms per move
#   e2e/multi.sh 20,20 60 0   # full speed
#   TUI=Alian e2e/multi.sh    # show that team's TUI, others run hidden
set -euo pipefail

MAZE=${1:-}
TIMEOUT=${2:-120}
MOVE_DELAY=${3:-100}
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
# nc tries IPv6 too: the server listens on localhost, which can resolve to ::1 only.
for _ in $(seq 50); do
  nc -z localhost "$PORT" 2>/dev/null && break
  sleep 0.1
done

echo "Maze ${MAZE:-default}, timeout ${TIMEOUT}s, ${MOVE_DELAY} ms per move, server log: $LOG"
CLIENT_PIDS=()
for algorithm in "${ALGORITHMS[@]}"; do
  [[ "$algorithm" == "$TUI" ]] && continue
  timeout "$TIMEOUT" "$CLIENT" --port "$PORT" --team "$algorithm" --algorithm "$algorithm" \
    --move-delay "$MOVE_DELAY" > /dev/null 2>&1 &
  CLIENT_PIDS+=($!)
done
if [[ -n "$TUI" ]]; then
  # No timeout: killing the TUI skips its terminal cleanup. Quit it with q.
  "$CLIENT" --port "$PORT" --team "$TUI" --algorithm "$TUI" --tui --move-delay "$MOVE_DELAY" || true
fi
wait "${CLIENT_PIDS[@]}" || true

TEAM_SIZE=3
for algorithm in "${ALGORITHMS[@]}"; do
  exits=$(grep -c "$algorithm/$algorithm[0-9]* found the exit" "$LOG" || true)
  echo "$algorithm: $exits/$TEAM_SIZE agents found the exit"
done

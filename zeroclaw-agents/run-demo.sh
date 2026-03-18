#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# AgentMesh — Milestone 3 Demo: x402 P2P Agent Pipeline
#
# Starts all three agents and triggers the scraper→analyzer→publisher flow.
#
# Usage:
#   cp .env.example .env      # fill in private keys + REGISTRY_ADDRESS
#   ./run-demo.sh
# ─────────────────────────────────────────────────────────────────────────────
set -e

# Load .env if present
if [ -f .env ]; then
  set -a; source .env; set +a
fi

# Require private keys
: "${SCRAPER_PRIVATE_KEY:?Set SCRAPER_PRIVATE_KEY in .env}"
: "${ANALYZER_PRIVATE_KEY:?Set ANALYZER_PRIVATE_KEY in .env}"
: "${PUBLISHER_PRIVATE_KEY:?Set PUBLISHER_PRIVATE_KEY in .env}"

CARGO="cargo"
if ! command -v cargo &>/dev/null; then
  CARGO="$HOME/.cargo/bin/cargo"
fi

BUILD_DIR="$(pwd)/target/debug"

echo "🔨 Building all agents…"
$CARGO build 2>&1 | grep -E "Compiling|Finished|error"
echo ""

# ── Start Publisher ───────────────────────────────────────────────────────────
echo "📢 Starting publisher on :${PUBLISHER_PORT:-8082}…"
PUBLISHER_PRIVATE_KEY="$PUBLISHER_PRIVATE_KEY" \
PUBLISHER_PORT="${PUBLISHER_PORT:-8082}" \
"$BUILD_DIR/publisher" &
PUBLISHER_PID=$!
sleep 0.5

# ── Start Scraper ─────────────────────────────────────────────────────────────
echo "🕷️  Starting scraper on :${SCRAPER_PORT:-8080}…"
SCRAPER_PRIVATE_KEY="$SCRAPER_PRIVATE_KEY" \
RPC_URL="${RPC_URL:-https://ethereum-sepolia-rpc.publicnode.com}" \
REGISTRY_ADDRESS="${REGISTRY_ADDRESS:-0x0000000000000000000000000000000000000000}" \
SCRAPER_PORT="${SCRAPER_PORT:-8080}" \
"$BUILD_DIR/scraper" &
SCRAPER_PID=$!
sleep 0.5

# ── Start Analyzer ────────────────────────────────────────────────────────────
echo "🧠 Starting analyzer on :${ANALYZER_PORT:-8081}…"
echo "   → will query registry, send TaskRequest to scraper, forward title to publisher"
ANALYZER_PRIVATE_KEY="$ANALYZER_PRIVATE_KEY" \
RPC_URL="${RPC_URL:-https://ethereum-sepolia-rpc.publicnode.com}" \
REGISTRY_ADDRESS="${REGISTRY_ADDRESS:-0x0000000000000000000000000000000000000000}" \
PUBLISHER_ADDRESS="${PUBLISHER_ADDRESS:-0x0000000000000000000000000000000000000000}" \
TARGET_URL="${TARGET_URL:-https://example.com}" \
ANALYZER_PORT="${ANALYZER_PORT:-8081}" \
"$BUILD_DIR/analyzer" &
ANALYZER_PID=$!

echo ""
echo "✅ All agents running. Press Ctrl+C to stop."
echo "   scraper  PID: $SCRAPER_PID  (ws://localhost:${SCRAPER_PORT:-8080})"
echo "   analyzer PID: $ANALYZER_PID (ws://localhost:${ANALYZER_PORT:-8081})"
echo "   publisher PID: $PUBLISHER_PID (ws://localhost:${PUBLISHER_PORT:-8082})"
echo ""

# Cleanup on exit
cleanup() {
  echo ""
  echo "🛑 Stopping agents…"
  kill $SCRAPER_PID $ANALYZER_PID $PUBLISHER_PID 2>/dev/null || true
  wait 2>/dev/null
}
trap cleanup EXIT INT TERM

wait

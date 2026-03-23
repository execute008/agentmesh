#!/bin/bash
# Start 3 ZeroClaw gateway instances for AgentMesh demo

set -e

ZEROCLAW_BIN=~/synthhack/zeroclaw/target/release/zeroclaw
WORKSPACES=~/synthhack/agentmesh/zeroclaw-workspaces

# Check if ZeroClaw is built
if [ ! -f "$ZEROCLAW_BIN" ]; then
    echo "❌ ZeroClaw not built. Run: cd ~/synthhack/zeroclaw && cargo build --release"
    exit 1
fi

# Check for ANTHROPIC_API_KEY
if [ -z "$ANTHROPIC_API_KEY" ]; then
    echo "❌ ANTHROPIC_API_KEY not set"
    exit 1
fi

echo "🚀 Starting AgentMesh ZeroClaw agents..."

# Start scraper agent
echo "  🕷️  Scraper agent (port 8080)..."
cd "$WORKSPACES/scraper"
$ZEROCLAW_BIN gateway --port 8080 &
SCRAPER_PID=$!

# Start analyzer agent
echo "  🧠 Analyzer agent (port 8081)..."
cd "$WORKSPACES/analyzer"
$ZEROCLAW_BIN gateway --port 8081 &
ANALYZER_PID=$!

# Start publisher agent
echo "  📢 Publisher agent (port 8082)..."
cd "$WORKSPACES/publisher"
$ZEROCLAW_BIN gateway --port 8082 &
PUBLISHER_PID=$!

echo ""
echo "✅ All agents running!"
echo "   Scraper:   http://localhost:8080 (PID $SCRAPER_PID)"
echo "   Analyzer:  http://localhost:8081 (PID $ANALYZER_PID)"
echo "   Publisher: http://localhost:8082 (PID $PUBLISHER_PID)"
echo ""
echo "📡 Talk to agents via CLI:"
echo "   zeroclaw agent --port 8080 -m \"hello scraper\""
echo "   zeroclaw agent --port 8081 -m \"hello analyzer\""
echo "   zeroclaw agent --port 8082 -m \"hello publisher\""
echo ""
echo "Press Ctrl+C to stop all agents"

# Wait for interrupt
trap "kill $SCRAPER_PID $ANALYZER_PID $PUBLISHER_PID 2>/dev/null; exit" INT TERM
wait

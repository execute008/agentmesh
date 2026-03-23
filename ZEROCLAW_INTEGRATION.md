# AgentMesh × ZeroClaw Integration

## Architecture

AgentMesh provides the **coordination layer** (on-chain registry, P2P messaging, payment escrow, reputation).

ZeroClaw provides the **intelligence layer** (LLM-powered reasoning, tool execution, autonomy).

Each agent runs a **ZeroClaw gateway** with its own workspace and system prompt.

## Agent Setup

### 1. Scraper Agent
- **Workspace:** `~/synthhack/agentmesh/zeroclaw-workspaces/scraper`
- **Port:** 8080
- **Role:** Web scraping specialist
- **System prompt:** `IDENTITY.md`

### 2. Analyzer Agent
- **Workspace:** `~/synthhack/agentmesh/zeroclaw-workspaces/analyzer`
- **Port:** 8081
- **Role:** Task orchestrator and coordinator
- **System prompt:** `IDENTITY.md`

### 3. Publisher Agent
- **Workspace:** `~/synthhack/agentmesh/zeroclaw-workspaces/publisher`
- **Port:** 8082
- **Role:** Notification and delivery specialist
- **System prompt:** `IDENTITY.md`

## Running the Demo

### Build ZeroClaw (one-time)
```bash
cd ~/synthhack/zeroclaw
cargo build --release
```

### Start all 3 agents
```bash
export ANTHROPIC_API_KEY="your-key-here"
cd ~/synthhack/agentmesh
./start-zeroclaw-agents.sh
```

### Talk to individual agents
```bash
# Scraper
zeroclaw agent --port 8080 -m "Fetch https://draht.dev and extract the title"

# Analyzer  
zeroclaw agent --port 8081 -m "Coordinate a scraping task for https://example.com"

# Publisher
zeroclaw agent --port 8082 -m "Format and deliver the result"
```

### Full workflow demo
```bash
# 1. Analyzer receives task
zeroclaw agent --port 8081 -m "I need to scrape https://news.ycombinator.com"

# 2. Analyzer discovers scraper from on-chain registry
# 3. Analyzer creates TaskRequest, escrows 0.001 ETH
# 4. Analyzer sends x402 message to scraper (P2P)
# 5. Scraper executes task, returns result via x402
# 6. Analyzer settles payment on-chain
# 7. Analyzer sends result to publisher
# 8. Publisher delivers formatted output
```

## Integration Points

### On-chain registration
Each agent auto-registers itself to the AgentRegistry contract on startup:
- Scraper: `scraper-001`, capability `web-scraping`, price 0.001 ETH
- Analyzer: `analyzer-001`, capability `orchestration`, price 0
- Publisher: `publisher-001`, capability `notification`, price 0

### x402 protocol
Agents communicate P2P using the x402 message format:
- `TaskRequest` — request work from another agent
- `TaskComplete` — return result to requester
- Each message is cryptographically signed

### Smart contract settlement
- Payment escrow via `createTask()`
- Automated settlement via `settleTask()`
- Reputation updates tracked on-chain

## Value Proposition

**AgentMesh** = trustless coordination infrastructure  
**ZeroClaw** = intelligent autonomous agents

Together: **Self-coordinating AI agent economy** with on-chain guarantees.

No central orchestrator. No trusted intermediary. Pure P2P with smart contract settlement.

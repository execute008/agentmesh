# AgentMesh

**Decentralized agent coordination protocol** — deployable smart contract standard enabling any agent to create an on-chain mesh, discover peers via chain scanning, coordinate via x402 P2P WebSocket messaging, and settle payments with on-chain escrow and reputation.

Built at [Synthesis Hackathon 2026](https://synthesis.md).

---

## Overview

AgentMesh lets autonomous agents find each other, negotiate work, and settle payments — all without a central server. Agents register on-chain, expose direct WebSocket endpoints, and speak the x402 protocol peer-to-peer.

```
┌─────────────────────────────────────────────────────┐
│                  ON-CHAIN (Ethereum)                │
│                                                     │
│  AgentRegistry.sol  ──►  Task Escrow (internal)     │
│  (identity, caps,         (lock → complete →        │
│   reputation, endpoint)    release + rep update)    │
└──────────────┬──────────────────────────────────────┘
               │ query endpoint / register
               ▼
┌─────────────────────────────────────────────────────┐
│              OFF-CHAIN (P2P WebSocket)              │
│                                                     │
│  Agent A  ──x402──►  Agent B  ──x402──►  Agent C    │
│  (WS server)          (WS server)        (WS server)│
│  ws://a.ngrok.io      ws://b.ngrok.io    ws://c...  │
└─────────────────────────────────────────────────────┘
               ▲
               │ scan / search / deploy / register
┌──────────────┴──────────────────────────────────────┐
│               CLI TOOL (agentmesh)                  │
│  scan | list | search | register | deploy           │
└─────────────────────────────────────────────────────┘
```

---

## Repo Structure

```
agentmesh/
├── agentmesh-contracts/   # Solidity — AgentRegistry.sol (Foundry)
├── agentmesh-cli/         # Rust CLI — deploy/scan/list/search/register
└── zeroclaw-agents/       # Rust workspace — 3 x402 P2P demo agents
    ├── shared/            # X402Message types, EIP-191 signing, send_to_agent()
    ├── scraper/           # Web scraping agent (WS :8080)
    ├── analyzer/          # Orchestrating agent (WS :8081)
    └── publisher/         # Result publishing agent (WS :8082)
```

---

## Prerequisites

- [Foundry](https://book.getfoundry.sh/getting-started/installation) (`forge`, `cast`)
- [Rust](https://rustup.rs/) (stable, 1.75+)
- A funded Sepolia wallet (for deploy + task escrow)
- [ngrok](https://ngrok.com/) (for live agent endpoints behind NAT)

---

## Quickstart

### 1. Clone & build contracts

```bash
git clone https://github.com/execute008/agentmesh
cd agentmesh/agentmesh-contracts
forge build
forge test  # 23 tests, all passing
```

### 2. Deploy to Sepolia

```bash
cd agentmesh-contracts
forge script script/Deploy.s.sol:Deploy \
  --rpc-url $SEPOLIA_RPC_URL \
  --private-key $PRIVATE_KEY \
  --broadcast
```

### 3. Build & configure the CLI

```bash
cd agentmesh-cli
cargo build --release
cp .env.example .env
# Edit .env: PRIVATE_KEY, SEPOLIA_RPC_URL
```

### 4. Scan for meshes and register an agent

```bash
# Scan for deployed meshes
./target/release/agentmesh scan --chain sepolia

# List agents in a mesh
./target/release/agentmesh list --contract 0xYOUR_CONTRACT

# Register your agent
./target/release/agentmesh register \
  --contract 0xYOUR_CONTRACT \
  --agent-id my-agent-001 \
  --capabilities web-scraping,html-parsing \
  --price 10000000000000000 \
  --endpoint wss://your-ngrok-url.ngrok.io
```

### 5. Run the x402 P2P demo

```bash
cd zeroclaw-agents
cp .env.example .env
# Edit .env: PRIVATE_KEY_SCRAPER, PRIVATE_KEY_ANALYZER, PRIVATE_KEY_PUBLISHER, REGISTRY_ADDRESS

# Start all 3 agents (3 terminals, or use the script)
bash run-demo.sh
```

The demo flow:
1. **Analyzer** queries the registry → finds the scraper's WS endpoint
2. Analyzer sends a signed `TaskRequest` (x402) → **Scraper** fetches a URL
3. Scraper replies with `TaskComplete` (x402) → Analyzer extracts `<title>`
4. Analyzer calls `createTask` + `completeTask` + `releasePayment` on-chain → ETH transferred + reputation +5
5. **Publisher** prints the final result

---

## Smart Contract

`AgentRegistry.sol` implements a deployable per-mesh registry:

| Feature | Details |
|---|---|
| Identity | ERC-8004 compliant — each agent has a wallet |
| Registration | `registerAgent(id, capabilities, price, wsEndpoint)` |
| Discovery | `searchByCapability(cap)` → wallet list |
| Endpoint lookup | `getAgent(wallet)` → full agent struct incl. WS URL |
| Task escrow | `createTask{value}` → `completeTask` → `releasePayment` |
| Reputation | On-chain score, +5 per completed task |

Tests: `forge test` — 23 tests, 100% coverage.

---

## CLI Commands

| Command | Description |
|---|---|
| `agentmesh deploy` | Deploy a new `AgentRegistry` contract |
| `agentmesh scan` | Scan chain for `MeshCreated` events → `meshes.json` |
| `agentmesh list` | List all agents in a mesh |
| `agentmesh search` | Search agents by capability across all meshes |
| `agentmesh register` | Register an agent on-chain |

See [agentmesh-cli/README.md](agentmesh-cli/README.md) for full usage.

---

## x402 Agent Protocol

Agents communicate via signed JSON messages over direct WebSocket connections:

```
TaskRequest  { task_id, requester_wallet, payload, signature }
TaskComplete { task_id, executor_wallet, result, signature }
```

Signatures use EIP-191 personal sign. All agents verify signatures before processing. No relay server — purely P2P.

---

## Tech Stack

- **Solidity / Foundry** — smart contracts, tests, deploy scripts
- **Rust / alloy** — CLI (direct EVM RPC, no ethers wrapper)
- **Rust / tokio + tungstenite** — async WebSocket agents
- **x402** — P2P agent messaging protocol
- **ERC-8004** — on-chain agent identity standard

---

## ZeroClaw Integration

AgentMesh provides the **coordination layer** (on-chain registry, P2P messaging, payment escrow, reputation).

ZeroClaw provides the **intelligence layer** (LLM-powered reasoning, tool execution, autonomy).

Each agent runs a **ZeroClaw gateway** with its own workspace and system prompt.

### Agent Setup

**Scraper Agent** - Web scraping specialist
- Workspace: `zeroclaw-workspaces/scraper`
- Port: 8080
- System prompt: `IDENTITY.md`

**Analyzer Agent** - Task orchestrator
- Workspace: `zeroclaw-workspaces/analyzer`
- Port: 8081
- System prompt: `IDENTITY.md`

**Publisher Agent** - Notification specialist
- Workspace: `zeroclaw-workspaces/publisher`
- Port: 8082
- System prompt: `IDENTITY.md`

### Running with ZeroClaw

```bash
# 1. Build ZeroClaw (one-time)
cd ~/synthhack/zeroclaw
cargo build --release

# 2. Set API key
export ANTHROPIC_API_KEY="your-key-here"

# 3. Start all 3 agents
cd ~/synthhack/agentmesh
./start-zeroclaw-agents.sh

# 4. Talk to individual agents
zeroclaw agent --port 8080 -m "Fetch https://draht.dev and extract the title"
zeroclaw agent --port 8081 -m "Coordinate a scraping task"
zeroclaw agent --port 8082 -m "Format and deliver the result"
```

See `ZEROCLAW_INTEGRATION.md` for full details.

---

## Next Steps (Demo Preparation)

### Pre-Recording Checklist

- [ ] 3 wallets funded with Sepolia ETH
- [ ] Agents registered on-chain
- [ ] `run-demo.sh` tested end-to-end successfully
- [ ] Etherscan tab ready with contract address (`0x1f24bfaf2c80299c512a5b59b0408726ca57b96f`)
- [ ] Terminal layout: 3 agents visible (tmux or split terminals)
- [ ] Code editor open to AgentRegistry.sol (for architecture segment)
- [ ] ZeroClaw gateways running (ports 8080, 8081, 8082)

### Video Recording Plan (3 minutes)

**Segment 1: Problem** (15s)
- You on camera explaining centralization issues
- Key points: Single points of failure, no trustless discovery, no on-chain payments

**Segment 2: Solution Overview** (30s)
- Explain 3 components: AgentRegistry.sol, agentmesh CLI, x402 P2P protocol
- Mention: On-chain escrow, direct WebSocket, trustless settlement

**Segment 3: Architecture** (40s)
- Screen recording: AgentRegistry.sol code walkthrough
- Show: `searchByCapability`, x402 message structure, escrow flow
- Highlight: createTask → completeTask → releasePayment

**Segment 4: Live Demo** (60s)
1. Show deployed contract on Etherscan (5s)
2. Register agents via CLI (10s)
3. Start agents with `run-demo.sh` (5s)
4. P2P communication logs (20s) - TaskRequest → TaskComplete
5. On-chain settlement (15s) - Show tx on Etherscan, 0.001 ETH transfer, reputation 50→55
6. Final state (5s) - Publisher prints result

**Segment 5: Results/Impact** (20s)
- Metrics: 26/26 tests passing, P2P WebSocket, EIP-191 signatures, on-chain escrow
- Key stat: Reputation starts at 50, +5 per task, max 100

**Segment 6: Call to Action** (auto-generated, 12s)
- Title card with GitHub link and contract address

### Video Files to Record

1. `problem.mp4` (15s) - You explaining the problem
2. `solution.mp4` (30s) - You explaining AgentMesh
3. `architecture.mp4` (40s) - Screen recording: code walkthrough
4. `demo.mp4` (60s) - Full live demo
5. `results.mp4` (20s) - You summarizing impact

**Recording tips:**
- 1920x1080 minimum, external mic if possible
- OBS or QuickTime for screen recording
- High energy, keep pace up
- 2 seconds padding at start/end

See `DEMO_SCRIPT.md` for full script details.

---

## Submission (Synthesis Hackathon)

**Status:** In Development | **Deadline:** March 22, 2026

### 🚀 Submission Quickstart

1. **Self-Custody Transfer** (required) - Get a wallet address, then run:
   ```bash
   ./submit-project.sh  # Options 4 & 5
   ```

2. **Compile Conversation Log**:
   ```bash
   ./compile-conversation-log.sh
   ```

3. **Record Demo Video** - Follow `DEMO_SCRIPT.md` (3 min)

4. **Post on Moltbook** - Announce at https://www.moltbook.com

5. **Submit via API**:
   ```bash
   ./submit-project.sh  # Option 6: Create draft, Option 8: Publish
   ```

6. **Tweet** - Tag @synthesis_md

**Full guide:** See `SUBMISSION-GUIDE.md` for step-by-step instructions

---

## License

MIT

# AgentMesh

**Decentralized agent coordination protocol** — deployable smart contract standard enabling any agent to create an on-chain mesh, discover peers via chain scanning, coordinate via x402 P2P WebSocket messaging, and settle payments with on-chain escrow and reputation.

Built at [Synthesis Hackathon 2026](https://synthesis.gg).

---

## Overview

AgentMesh lets autonomous agents find each other, negotiate work, and settle payments — all without a central server. Agents register on-chain, expose direct WebSocket endpoints, and speak the x402 protocol peer-to-peer.

```
┌─────────────────────────────────────────────────────┐
│                  ON-CHAIN (Ethereum)                │
│                                                     │
│  AgentRegistry.sol  ──►  Task Escrow (internal)    │
│  (identity, caps,         (lock → complete →        │
│   reputation, endpoint)    release + rep update)    │
└──────────────┬──────────────────────────────────────┘
               │ query endpoint / register
               ▼
┌─────────────────────────────────────────────────────┐
│              OFF-CHAIN (P2P WebSocket)              │
│                                                     │
│  Agent A  ──x402──►  Agent B  ──x402──►  Agent C   │
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

## License

MIT

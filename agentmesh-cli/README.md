# agentmesh-cli

Rust CLI for the AgentMesh decentralized agent coordination protocol.

## Install

```bash
# From repo root
cd agentmesh-cli
cargo build --release
# Binary at target/release/agentmesh

# Or install globally
cargo install --path .
```

## Setup

```bash
cp .env.example .env
# Edit .env with your PRIVATE_KEY and RPC URL
```

## Commands

### Deploy a new mesh

```bash
agentmesh deploy --name "AgentMesh-Demo" --chain sepolia
```

Deploys a new `AgentRegistry` contract and writes the address to `meshes.json`.

> **Requires:** `forge build` run in `agentmesh-contracts/` first (reads bytecode from the artifact JSON).

### Scan for existing meshes

```bash
agentmesh scan --chain sepolia
agentmesh scan --chain sepolia --from-block 7500000
```

Scans for `MeshCreated` events on-chain and populates `meshes.json`.

### List agents in a mesh

```bash
agentmesh list --contract 0xAbCd...
agentmesh list --contract 0xAbCd... --chain mainnet
```

Queries `getAllAgents()` + `getAgent()` for each wallet and prints a formatted summary.

### Search for a capability

```bash
agentmesh search "web-scraping"
agentmesh search "analysis" --chain sepolia
```

Searches across all meshes listed in `meshes.json` using `searchByCapability()`.

### Register an agent

```bash
agentmesh register \
  --contract 0xAbCd... \
  --agent-id scraper-001 \
  --capabilities web-scraping,html-parsing \
  --price 10000000000000000 \
  --endpoint wss://scraper.example.com

# With explicit private key
agentmesh register --contract 0x... --agent-id bot-1 \
  --capabilities web-scraping \
  --price 1000000 \
  --endpoint wss://bot1.ngrok.io \
  --private-key 0x...

# With keystore file
agentmesh register --contract 0x... ... --account ~/.keystore/deployer.json
```

## meshes.json format

```json
[
  {
    "address": "0xabc123...",
    "name": "AgentMesh-Demo",
    "deployed_at": 7512345
  }
]
```

## Environment Variables

| Variable | Description |
|---|---|
| `PRIVATE_KEY` | Signer private key (0x-prefixed hex) |
| `SEPOLIA_RPC_URL` | Sepolia RPC endpoint |
| `MAINNET_RPC_URL` | Mainnet RPC endpoint |
| `RPC_URL` | Generic RPC override (any chain) |

## Architecture

The CLI communicates directly with deployed `AgentRegistry` contracts using **alloy** (the modern Rust Ethereum library). No central server — all data comes from on-chain events and contract calls.

```
agentmesh scan       → eth_getLogs (MeshCreated events) → meshes.json
agentmesh list       → getAllAgents() + getAgent() calls
agentmesh search     → searchByCapability() across all meshes
agentmesh register   → registerAgent() transaction (signed)
agentmesh deploy     → raw deploy transaction (signed) → meshes.json
```

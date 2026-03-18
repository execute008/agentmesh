# Architecture

Generated: 2026-03-18 (filled from PROMPT.md spec)

## Overview
Hybrid on-chain / off-chain decentralized protocol. No central server.

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

## Module Boundaries

### `agentmesh-contracts/` (Foundry project)
- `src/AgentRegistry.sol` — single deployable contract per mesh
- `src/interfaces/IAgentRegistry.sol` — interface for external consumers
- `test/` — Foundry tests
- `script/Deploy.s.sol` — deploy script

### `agentmesh-cli/` (Rust binary)
- `src/main.rs` — clap CLI entry point
- `src/scanner.rs` — chain event scanning, mesh discovery
- `src/commands/` — one module per CLI subcommand

### `zeroclaw-agents/` (Rust workspace members)
- `scraper/` — web scraping agent, WS server on :8080
- `analyzer/` — orchestrating agent, WS server on :8081
- `publisher/` — result publishing agent, WS server on :8082
- `shared/` — x402 message types, signing, send_to_agent helper

## Data Flow
1. Agent starts → runs local WS server → exposes via ngrok
2. Agent calls `AgentRegistry.registerAgent(id, caps, price, wsEndpoint)`
3. Requester calls `AgentRegistry.searchByCapability("web-scraping")` → gets wallet list
4. Requester calls `AgentRegistry.getAgent(wallet)` → gets WS endpoint
5. Requester opens direct WS connection to executor's ngrok URL
6. x402 `TaskRequest` message sent (wallet-signed)
7. Executor verifies signature, does work, sends x402 `TaskComplete` back
8. Requester calls `AgentRegistry.createTask{value}(taskId, executorAddr)` — escrows ETH
9. Requester calls `AgentRegistry.completeTask(taskId)`
10. Requester calls `AgentRegistry.releasePayment(taskId, self)` → ETH transferred + rep +5

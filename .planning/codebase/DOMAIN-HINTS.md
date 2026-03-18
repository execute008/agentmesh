# Domain Model Hints

Generated: 2026-03-18 (filled from PROMPT.md spec)

## Core Entities
- **Mesh** — a deployed `AgentRegistry` contract instance (address + name + owner)
- **Agent** — wallet-identified participant with capabilities, price, endpoint, reputation
- **Task** — a unit of work with escrow, executor, completion state
- **Capability** — string tag describing what an agent can do (e.g. "web-scraping")
- **X402Message** — signed off-chain message between agents

## Bounded Contexts (preliminary)
1. **Registry** — agent identity, capability advertisement, mesh ownership
2. **Escrow & Settlement** — task creation, locking ETH, completion, payment release
3. **Reputation** — on-chain scores that follow wallets across meshes
4. **Messaging** — x402 off-chain P2P WS protocol (signing, routing, delivery)
5. **Discovery** — CLI scanning chains for mesh contracts, cross-mesh search

## Value Objects
- Capability (string, immutable)
- Reputation score (uint8, 0-100)
- Price per task (uint256 wei)
- WS Endpoint URL (string)

## Domain Events (cross-boundary)
- `MeshCreated(name, owner)` — on-chain
- `AgentRegistered(wallet, agentId, capabilities)` — on-chain
- `TaskCreated(taskId, requester, executor, amount)` — on-chain
- `TaskCompleted(taskId)` — on-chain
- `PaymentReleased(taskId, amount)` — on-chain
- `ReputationUpdated(wallet, newReputation)` — on-chain
- `X402TaskRequest` — off-chain
- `X402TaskComplete` — off-chain

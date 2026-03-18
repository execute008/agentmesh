# Domain Model — AgentMesh

## Bounded Contexts

### 1. Registry
**Responsibility:** Agent identity, capability advertisement, mesh ownership, on-chain endpoint discovery.  
Lives entirely in `AgentRegistry.sol` and the CLI `register`/`list` commands.

### 2. Escrow & Settlement
**Responsibility:** Locking ETH before work starts, confirming task completion, releasing payment to executor.  
Lives in `AgentRegistry.sol` (Task struct + createTask/completeTask/releasePayment). Escrow is created **before** x402 work begins.

### 3. Reputation
**Responsibility:** Per-wallet on-chain score (0–100) that increments +5 on successful payment release. Follows the wallet address across any mesh.  
Computed and stored inside `AgentRegistry.sol`, queryable via `getAgent`.

### 4. Messaging (x402)
**Responsibility:** Off-chain P2P signed message exchange between agents over direct WebSocket connections. Endpoints discovered via Registry.  
Lives in `zeroclaw-agents/shared/` — message types, signing, `send_to_agent`.

### 5. Discovery
**Responsibility:** CLI scanning Sepolia for deployed `AgentRegistry` contracts (via `MeshCreated` events), persisting results to `meshes.json`, enabling global capability search.  
Lives in `agentmesh-cli/src/scanner.rs` + `meshes.json` local file.

---

## Ubiquitous Language

| Term | Definition |
|------|-----------|
| **Mesh** | A single deployed `AgentRegistry` contract instance. Identified by address + name. Any agent can deploy one. |
| **Agent** | A wallet-identified autonomous process with declared capabilities, a price, a WebSocket endpoint, and a reputation score. |
| **Capability** | A plain-string tag an agent advertises (e.g. `"web-scraping"`, `"analysis"`, `"publishing"`). Used for discovery. |
| **Endpoint** | The public WebSocket URL (`wss://...`) where an agent receives incoming x402 messages. Stored on-chain. |
| **Task** | A unit of work with an escrowed payment, a requester, an executor, and a completion state. Lives on-chain. |
| **Escrow** | ETH locked in the contract at task creation, released to executor after requester confirms completion. |
| **Reputation** | An on-chain uint8 score per wallet. Starts at 50. Increases +5 per successful payment release. Max 100. |
| **x402 Message** | A signed off-chain JSON message sent directly over WebSocket between two agents. Includes `from`, `to`, `message_type`, `payload`, and `signature`. |
| **TaskRequest** | x402 message type: requester asks executor to perform work. Sent **after** escrow is created. |
| **TaskComplete** | x402 message type: executor notifies requester that work is done and sends result payload. |
| **meshes.json** | Local file maintained by the CLI listing known deployed mesh addresses. Written by `deploy`, read by all other commands. |

---

## Context Map

```
Discovery ──reads──► meshes.json ◄──writes── (deploy command)
     │
     └──queries──► Registry (on-chain)
                        │
                        ├── Reputation (computed within Registry)
                        │
                        └── Escrow & Settlement (computed within Registry)
                                    │
                                    ▼
                        Messaging (x402) ◄─── endpoint discovered via Registry
```

- **Registry → Messaging**: upstream. Registry provides the endpoint URL that Messaging uses to route x402 messages.
- **Escrow → Registry**: same contract, internal dependency. Escrow state stored alongside agent state.
- **Discovery → Registry**: ACL. CLI translates raw chain events into local `meshes.json` format.

---

## Aggregates

### Registry Context
- **AgentRegistry** (root) — owns `Agent` records, `Task` records, `meshName`, `meshOwner`
  - `Agent` — wallet, agentId, capabilities[], pricePerTask, reputation, endpoint, active
  - `Task` — requester, executor, taskId, escrowAmount, completed, released

### Messaging Context
- **X402Message** (root) — version, from, to, messageId, timestamp, messageType, payload, signature

---

## Domain Events

| Event | Context | Direction | Description |
|-------|---------|-----------|-------------|
| `MeshCreated(name, owner)` | Registry | on-chain | New AgentRegistry deployed |
| `AgentRegistered(wallet, agentId, capabilities)` | Registry | on-chain | Agent joins a mesh |
| `AgentUpdated(wallet, capabilities)` | Registry | on-chain | Agent updates profile |
| `TaskCreated(taskId, requester, executor, amount)` | Escrow | on-chain | ETH locked, work can begin |
| `TaskCompleted(taskId)` | Escrow | on-chain | Executor marked work done |
| `PaymentReleased(taskId, amount)` | Escrow | on-chain | ETH sent to executor |
| `ReputationUpdated(wallet, newReputation)` | Reputation | on-chain | Score changed after release |
| `X402TaskRequest` | Messaging | off-chain | Requester → Executor: do this work |
| `X402TaskComplete` | Messaging | off-chain | Executor → Requester: here's the result |

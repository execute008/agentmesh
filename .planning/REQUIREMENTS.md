# Requirements — AgentMesh

**Hackathon:** The Synthesis · Deadline: Mar 22, 2026  
**Prize targets:** Open Track $20k · x402 $5k · Protocol Labs $8k

---

## V1 — MVP (must ship by Mar 22)

### Smart Contracts [Registry · Escrow · Reputation]
- [ ] `AgentRegistry.sol` compiles with Foundry, all Foundry tests pass (>80% coverage)
- [ ] `registerAgent(agentId, capabilities[], pricePerTask, wsEndpoint)` — stores ERC-8004 identity on-chain
- [ ] `createTask{value}(taskId, executorAddr)` — locks ETH escrow **before** x402 work starts
- [ ] `completeTask(taskId)` — executor marks work done
- [ ] `releasePayment(taskId, requester)` — transfers ETH to executor + reputation +5
- [ ] `searchByCapability(cap)` — returns array of matching agent wallets
- [ ] `getAgent(wallet)` — returns full Agent struct including WS endpoint
- [ ] Deployed to Sepolia, 1 mesh instance (`AgentMesh-Demo`), 3 agents registered
- [ ] Verified on Sepolia Etherscan (source code + ABI)

### CLI Tool [Discovery]
- [ ] `agentmesh deploy --name <name>` — deploys AgentRegistry, writes address to `meshes.json`
- [ ] `agentmesh scan --chain sepolia` — scans for `MeshCreated` events, populates `meshes.json`
- [ ] `agentmesh list --contract <addr>` — lists all active agents in a mesh
- [ ] `agentmesh search <capability>` — searches across all meshes in `meshes.json`
- [ ] `agentmesh register --contract <addr> --agent-id <id> --capabilities <cap> --price <wei> --endpoint <wss>` — registers agent on-chain
- [ ] `meshes.json` format: `[{"address": "0x...", "name": "...", "deployed_at": <blocknum>}]`

### x402 P2P Messaging [Messaging]
- [ ] `X402Message` struct: version, from, to, messageId, timestamp, messageType, payload, signature
- [ ] Message types: `TaskRequest`, `TaskComplete` (minimum for MVP)
- [ ] Each agent runs its own WS server (axum + WebSocket upgrade)
- [ ] Outbound: `send_to_agent(targetWallet, meshContract, message)` — queries contract for endpoint, opens direct WS, signs + sends
- [ ] Inbound: verify wallet signature before processing
- [ ] No central relay — fully peer-to-peer

### Demo Flow (end-to-end) [all contexts]
- [ ] **Analyzer** boots → queries `searchByCapability("web-scraping")` → gets scraper wallet
- [ ] Analyzer calls `createTask{0.01 ether}(taskId, scraperWallet)` — escrow locked on-chain
- [ ] Analyzer opens direct WS to scraper's ngrok endpoint → sends x402 `TaskRequest` `{url: "https://example.com"}`
- [ ] **Scraper** receives request → fetches https://example.com → sends x402 `TaskComplete` `{html: "..."}`
- [ ] Analyzer receives HTML → extracts `<title>` text
- [ ] Analyzer calls `completeTask` + `releasePayment` → ETH sent to scraper, reputation updated
- [ ] Analyzer sends x402 to **Publisher** `{title: "<extracted title>"}` (no escrow)
- [ ] Publisher prints title to stdout
- [ ] All 3 agents have distinct wallets; reputation visible on-chain

### Repository [submission]
- [ ] Public GitHub repo
- [ ] `README.md` with architecture diagram, quick start, prize track mapping
- [ ] Demo video (3–5 min) showing full flow

---

## V2 — Post-hackathon / stretch

- [ ] 3 separate mesh deployments (true cross-mesh discovery demo)
- [ ] libp2p replacing ngrok for production-grade P2P
- [ ] `TaskFailed` + dispute/refund flow in escrow
- [ ] Real-time dashboard UI showing agent activity
- [ ] Solana / Anchor version of AgentRegistry
- [ ] 10+ agents on Raspberry Pi cluster

---

## Out of Scope

- Frontend / web UI (stdout is sufficient for demo)
- Multi-chain (Base, Mainnet) deployment during hackathon
- Agent authentication beyond wallet signature
- Persistent task history / indexer
- Token-based payments (ETH only)

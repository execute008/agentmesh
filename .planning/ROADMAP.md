# Roadmap — AgentMesh

**Total runway:** 4 days (Mar 18–22, 2026)  
**Strategy:** Contracts first → CLI → Agents → Wire together → Demo + submit

---

## Milestone 1 — Smart Contracts (Day 1 · Mar 18)

### Phase 1.1 — Foundry project scaffold
- Initialize `agentmesh-contracts/` with `forge init`
- Install OpenZeppelin: `forge install OpenZeppelin/openzeppelin-contracts`
- Confirm `forge build` passes on empty project

### Phase 1.2 — AgentRegistry.sol
- Implement full contract per spec (Agent struct, Task struct, all functions)
- NatSpec on all public functions
- Events emitted on every state change (needed for CLI scanning)

### Phase 1.3 — Foundry tests
- `AgentRegistry.t.sol`: registerAgent, createTask+escrow, releasePayment+reputation, searchByCapability
- All tests pass (`forge test`)
- Coverage check (`forge coverage`) → >80% on critical paths

### Phase 1.4 — Deploy to Sepolia
- `script/Deploy.s.sol` with env-based config
- Deploy `AgentMesh-Demo` mesh
- Verify on Etherscan
- Save contract address (will go into `meshes.json`)

**Exit criteria:** Contract live on Sepolia, tests green, address known.

---

## Milestone 2 — CLI Tool (Day 2 · Mar 19)

### Phase 2.1 — Rust project scaffold
- `cargo new agentmesh-cli`
- Add all dependencies to `Cargo.toml`
- Clap skeleton compiles with all subcommand stubs

### Phase 2.2 — meshes.json + deploy command
- `meshes.json` read/write helpers
- `agentmesh deploy --name <name>` — deploys contract, writes to `meshes.json`

### Phase 2.3 — scanner + scan command
- `scanner.rs`: scan `MeshCreated` events from Sepolia, populate `meshes.json`
- `agentmesh scan --chain sepolia` — works against public RPC

### Phase 2.4 — list + search + register commands
- `agentmesh list --contract <addr>` — reads `getAllAgents` + `getAgent`
- `agentmesh search <cap>` — cross-mesh `searchByCapability` against all meshes in `meshes.json`
- `agentmesh register` — calls `registerAgent` with wallet signature

**Exit criteria:** All 5 CLI commands functional, 3 agents registered on Sepolia via CLI.

---

## Milestone 3 — x402 Agents (Day 3 · Mar 20)

### Phase 3.1 — Shared types crate
- `zeroclaw-agents/shared/` crate
- `X402Message` struct + `X402MessageType` enum (TaskRequest, TaskComplete)
- Message signing (wallet → keccak256 of payload → ethers sign)
- Signature verification
- `send_to_agent(targetWallet, meshContract, message)` helper
- Unit tests: serialize/deserialize roundtrip, sign/verify

### Phase 3.2 — Scraper agent
- WS server on :8080 (axum + WS upgrade)
- Handles `TaskRequest` → `reqwest::get` → sends `TaskComplete {html}`
- Verifies incoming message signatures
- `ngrok http 8080` for public endpoint

### Phase 3.3 — Analyzer agent
- WS server on :8081
- Boot sequence: query registry → `createTask{0.01 ether}` → send `TaskRequest` to scraper
- Receives `TaskComplete` → extract `<title>` from HTML
- Call `completeTask` + `releasePayment` on-chain
- Forward title to publisher via x402

### Phase 3.4 — Publisher agent
- WS server on :8082
- Receives x402 message → prints `[PUBLISHER] Title: <title>` to stdout
- No payment leg

**Exit criteria:** All 3 agents run locally, full flow completes end-to-end on localhost with anvil.

---

## Milestone 4 — Integration & Submission (Day 4 · Mar 21–22)

### Phase 4.1 — Sepolia end-to-end
- Fund 3 wallets with Sepolia ETH
- Register all 3 agents on `AgentMesh-Demo` with ngrok endpoints
- Run full demo flow against live Sepolia
- Verify on Etherscan: TaskCreated → TaskCompleted → PaymentReleased → ReputationUpdated events

### Phase 4.2 — README + docs
- Architecture diagram (ASCII or Mermaid)
- Quick start guide (deploy → register → run agents)
- Prize track mapping section
- `conversationLog` entries for Synthesis submission

### Phase 4.3 — Demo video
- 3–5 min screen recording
- Show: CLI scan → agent startup → full P2P flow → Etherscan proof

### Phase 4.4 — Synthesis submission
- Create project via Synthesis API (`POST /projects`)
- Fill all fields: title, description, repoUrl, demoUrl, submissionMetadata
- Publish submission before deadline

**Exit criteria:** Submission published, repo public, demo video linked.

---

## Risk Mitigations

| Risk | Mitigation |
|------|-----------|
| ngrok free tier limits | Have Cloudflare Tunnel as backup |
| Sepolia faucet dry | Fund wallets on Day 1, keep buffer |
| ethers-rs v2 API surprises | Use `abigen!` macro, check docs early |
| Foundry ABI path wrong | Build contracts before cargo build |
| Public RPC rate limits | Switch to Alchemy key if needed |

# Test Strategy — AgentMesh

## Test Framework

| Layer | Framework | Rationale |
|-------|-----------|-----------|
| Smart contracts | **Foundry** (`forge test`) | Native Solidity testing, fast, built-in cheatcodes (`vm.prank`, `vm.deal`), coverage via `forge coverage` |
| Rust (CLI + agents) | **`cargo test`** (built-in) | Standard, zero config. Integration tests in `tests/` dir per crate. |

---

## Directory Conventions

```
agentmesh-contracts/
└── test/
    ├── AgentRegistry.t.sol       # unit tests (registration, escrow, reputation)
    └── AgentRegistryIntegration.t.sol  # multi-agent flow tests

agentmesh-cli/
├── src/
│   └── scanner.rs                # unit tests inline (#[cfg(test)])
└── tests/
    └── cli_integration.rs        # CLI command integration tests (against anvil)

zeroclaw-agents/
└── shared/
    └── tests/
        └── x402_messages.rs      # message serialization + signature verification
```

---

## Coverage Goals

| Component | Target | Critical Paths |
|-----------|--------|---------------|
| `AgentRegistry.sol` | **>80%** | registerAgent, createTask, releasePayment, searchByCapability |
| `agentmesh-cli` scanner | **>60%** | mesh discovery, meshes.json read/write |
| `shared` x402 types | **>70%** | message signing, signature verification |

---

## Testing Levels

### Unit (fast, no network)

**Contracts (Foundry):**
- `testRegisterAgent()` — happy path + duplicate guard
- `testCreateTaskWithEscrow()` — ETH locked, balance check
- `testPaymentRelease()` — ETH transferred + reputation +5
- `testSearchByCapability()` — returns correct subset
- `testCannotRegisterTwice()` — revert on duplicate
- `testCannotReleaseUncompletedTask()` — revert guard

**Rust (inline `#[cfg(test)]`):**
- x402 message serialize/deserialize roundtrip
- Signature verification (known key + payload)
- `meshes.json` read/write helpers

### Integration (requires local anvil or Sepolia fork)

**CLI against `anvil`:**
- Deploy mesh → written to `meshes.json`
- Register agent → queryable via `list`
- `search "web-scraping"` → returns registered agent

**Agent flow (local anvil + localhost WS):**
- Scraper WS server starts → accepts connection → returns HTML
- Analyzer: discover scraper on-chain → create escrow → send x402 → receive result → release payment
- Publisher: receive x402 → print title to stdout

### E2E (Sepolia — run once before submission)
- Deploy `AgentMesh-Demo` mesh → address in `meshes.json`
- Register all 3 agents with their wallets and ngrok endpoints
- Run full analyzer → scraper → publisher flow
- Verify on Sepolia Etherscan: TaskCreated + PaymentReleased + ReputationUpdated events

---

## Excluded

| What | Why |
|------|-----|
| `script/Deploy.s.sol` | Foundry deploy scripts — not unit testable, covered by e2e |
| ngrok tunnel setup | External service, not in our control |
| `.synthesis` credentials file | Config/secrets, not code |
| `meshes.json` format migration | No versioning needed for hackathon scope |

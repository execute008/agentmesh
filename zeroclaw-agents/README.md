# zeroclaw-agents — AgentMesh Milestone 3: x402 P2P Agents

Three Rust agent binaries that communicate **fully peer-to-peer** over WebSocket using the **x402 protocol** — no central relay, endpoints discovered from the on-chain AgentRegistry.

```
scraper (ws/:8080) ←── TaskRequest ─── analyzer (ws/:8081) ←── discovered via registry
     │                                       │
     └──── TaskComplete ────────────────────►│
                                             │ extract <title>
                                             │ completeTask + releasePayment on-chain
                                             │ Notification
                                             └──────────────────► publisher (ws/:8082)
                                                                       │
                                                                       └── println! title
```

## Quick Start

```bash
cp .env.example .env
# fill in: SCRAPER_PRIVATE_KEY, ANALYZER_PRIVATE_KEY, PUBLISHER_PRIVATE_KEY
# optionally: RPC_URL, REGISTRY_ADDRESS, TARGET_URL

./run-demo.sh
```

## Workspace Layout

```
zeroclaw-agents/
├── shared/          # x402 types, signing, P2P transport, registry queries
│   └── src/
│       ├── lib.rs
│       ├── types.rs      # X402Message, X402MessageType, payload structs
│       ├── crypto.rs     # sign_message, verify_signature (EIP-191, alloy)
│       ├── transport.rs  # send_to_agent, send_and_receive (tokio-tungstenite)
│       └── registry.rs   # find_agents_by_capability, get_agent (alloy sol!)
├── scraper/         # axum WS server :8080 — fetches URLs on demand
├── analyzer/        # axum WS server :8081 — orchestrates the pipeline
└── publisher/       # axum WS server :8082 — prints extracted titles
```

## x402 Protocol

Every message is a JSON-encoded `X402Message`:

```json
{
  "version":      "x402/1.0",
  "from":         "0xABCD…",
  "to":           "0x1234…",
  "message_id":   "uuid-v4",
  "timestamp":    1710000000,
  "message_type": "TASK_REQUEST",
  "payload":      { "task_id": 1, "url": "https://example.com", … },
  "signature":    "0xhex…"
}
```

`signature` is an **EIP-191 personal_sign** of the canonical signing bytes, computed with alloy's `PrivateKeySigner`. Recipients call `verify_signature()` to confirm origin.

## Message Types (MVP)

| Type | Direction | Payload |
|------|-----------|---------|
| `TASK_REQUEST` | analyzer → scraper | `TaskRequestPayload` — `task_id`, `url`, `description`, `registry_address` |
| `TASK_COMPLETE` | scraper → analyzer | `TaskCompletePayload` — `task_id`, `result` (HTML body), `status` |
| `NOTIFICATION` | analyzer → publisher | `{ task_id, title, url }` |

## P2P Discovery

1. Analyzer calls `AgentRegistry.searchByCapability("web-scraping")` on-chain
2. Gets back a list of wallet addresses → calls `getAgent(wallet)` for each
3. Reads `wsEndpoint` field (registered by the scraper with `agentmesh register`)
4. Opens a direct WebSocket connection — no relay involved

Same pattern for publisher: analyzer looks up publisher wallet → gets endpoint → sends `Notification`.

**Dev fallback:** if `REGISTRY_ADDRESS` is zero or the registry has no agents, all three agents fall back to `localhost:808{0,1,2}` so you can demo without a live chain.

## On-Chain Settlement

After receiving a `TaskComplete` x402 message, the analyzer:

1. Calls `AgentRegistry.completeTask(taskId)` — marks task done on-chain
2. Calls `AgentRegistry.releasePayment(taskId, requester)` — releases ETH escrow to scraper

These are the same functions defined in `agentmesh-contracts/src/AgentRegistry.sol`.

## Stack

- **Rust** + **tokio** (async runtime)
- **axum 0.7** (WebSocket server, `ws` feature)
- **tokio-tungstenite 0.24** (WebSocket client for P2P sends)
- **alloy 1.7** (Ethereum signing, contract calls — consistent with agentmesh-cli)
- **reqwest 0.12** (HTTP fetching in scraper)
- **serde_json** (x402 message serialization)

## Building Individually

```bash
# from zeroclaw-agents/
cargo build -p scraper
cargo build -p analyzer
cargo build -p publisher

# or all at once
cargo build
```

## Registering Agents (for real on-chain discovery)

After deploying a registry with `agentmesh deploy`:

```bash
# Register the scraper with "web-scraping" capability
agentmesh register \
  --registry 0xYOUR_REGISTRY \
  --agent-id scraper-1 \
  --capabilities web-scraping \
  --price 1000000000000000 \
  --endpoint ws://YOUR_PUBLIC_IP:8080

# Register the publisher
agentmesh register \
  --registry 0xYOUR_REGISTRY \
  --agent-id publisher-1 \
  --capabilities publishing \
  --price 0 \
  --endpoint ws://YOUR_PUBLIC_IP:8082
```

Then set `PUBLISHER_ADDRESS` to the publisher's registered wallet address before starting the analyzer.

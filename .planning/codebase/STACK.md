# Technology Stack

Generated: 2026-03-18 (filled from PROMPT.md spec)

## Languages & Runtimes
- **Solidity ^0.8.20** — smart contracts (AgentRegistry, escrow, reputation)
- **Rust (edition 2021)** — CLI tool + agent binaries (ethers-rs, tokio, clap)
- No frontend planned (CLI + on-chain)

## Build Tools
- **Foundry** — Solidity compile, test, deploy (forge, cast, anvil)
- **Cargo** — Rust workspace build

## Key Dependencies
### Contracts
- OpenZeppelin Contracts (access control, security patterns)

### Rust CLI / Agents
- `ethers = "2.0"` — blockchain interaction, ABI bindings, wallet signing
- `tokio` (full) — async runtime
- `clap v4` (derive) — CLI argument parsing
- `axum` — agent WebSocket server (HTTP upgrade)
- `tokio-tungstenite` — WebSocket client (agent-to-agent P2P)
- `serde / serde_json` — message serialization
- `anyhow` — error handling
- `dotenv` — env var loading

## Networks
- **Ethereum Sepolia** — primary testnet deployment
- **Base Mainnet** — ERC-8004 identity (via Synthesis hackathon registration)

## Protocols
- **ERC-8004** — on-chain agent identity standard
- **x402** — off-chain P2P WebSocket messaging format (signed messages)

## Infrastructure (Demo)
- **ngrok / Cloudflare Tunnel** — NAT traversal for WebSocket endpoints
- libp2p — stretch goal for production P2P

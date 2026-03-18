# Concerns

Generated: 2026-03-18

## Technical Risks
- **NAT traversal** — ngrok free tier has connection limits; need backup (cloudflare tunnel)
- **Foundry ABI output path** — `out/AgentRegistry.sol/AgentRegistry.json` must exist before Rust `abigen!` compiles
- **ethers-rs v2 API** — breaking changes from v1; some examples online are outdated
- **Sepolia faucet** — need test ETH for 3 mesh deployments + escrow transactions

## Hackathon Constraints
- **4-day deadline** — Mar 18-22, 2026; no scope creep
- **Demo must be live** — judges expect working end-to-end, not mocked
- **Open source required** — repo must be public before submission
- **Conversation log** — must document human-agent collaboration for judges

## Out of Scope (for MVP)
- Frontend / dashboard UI (stretch)
- Solana / Anchor version (stretch)
- libp2p production P2P (stretch)
- Multi-chain mesh federation

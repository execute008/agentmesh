# Phase 4 Context

## Domain Boundary
Roadmap Phase 1.4 — Deploy AgentRegistry to Sepolia. Covers: keystore creation, deploy script, meshes.json generation, Etherscan verification, faucet documentation.

## Decisions
1. **Mesh name config:** Read from `MESH_NAME` env var, not hardcoded — reusable for future deployments
2. **Private key management:** Foundry keystore (`cast wallet new`) — encrypted on disk, password-prompted at deploy time
3. **Etherscan verification:** Separate step (`forge verify-contract` after deploy), not inline `--verify` — more resilient to Etherscan API flakiness
4. **RPC endpoint:** Public Sepolia RPC (e.g. `https://ethereum-sepolia-rpc.publicnode.com`), with Alchemy as documented fallback
5. **Contract address storage:** Deploy script writes `meshes.json` directly via Foundry `vm.writeJson` cheatcode
6. **Keystore setup:** Include `cast wallet new` as an explicit task in the plan
7. **Sepolia funding:** Include a task documenting faucet URL and recommended ETH amount
8. **Deploy script output:** Emit summary to stdout (address, mesh name, block number, chain ID) AND write `meshes.json`

## Claude's Discretion
- Exact Foundry `vm.writeJson` implementation details
- Choice of public RPC URL (any reliable Sepolia endpoint)
- Deploy script file structure (`script/Deploy.s.sol` as per roadmap)
- `meshes.json` output path (project root)

## Deferred Ideas
- Inline `--verify` can be revisited if separate verification proves annoying
- Multiple mesh deployments (V2 scope)

---
Created: 2026-03-18 06:13

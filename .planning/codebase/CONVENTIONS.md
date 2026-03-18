# Conventions

Generated: 2026-03-18 (filled from PROMPT.md spec)

## Solidity
- SPDX-License-Identifier: MIT on every file
- NatSpec (`@notice`, `@dev`, `@param`) on all public functions
- Section separators: `/*// SECTION NAME //*/`
- Error messages: descriptive string literals (no custom errors yet)
- Events: emit for every state-changing action (enables CLI scanning)

## Rust
- Edition 2021
- `anyhow::Result` for error propagation throughout
- `tokio` async everywhere (no blocking code on async tasks)
- Struct naming: PascalCase, file naming: snake_case
- Message types in shared `agentmesh-types` crate

## Git
- Commit format: `<type>: <description>` (feat, fix, chore, docs, test)
- Branch: work directly on `main` (solo hackathon sprint)

## Environment
- All secrets via `.env` file (never committed — in `.gitignore`)
- `.synthesis` file stores hackathon credentials (already gitignored)

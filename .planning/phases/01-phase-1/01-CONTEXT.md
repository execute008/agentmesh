# Phase 1.1 Context — Foundry Project Scaffold

## Domain Boundary
Bootstraps the `agentmesh-contracts/` Foundry project. No contract logic yet — just a clean, correctly-configured build environment that Phase 1.2 will write into.

## Decisions

| # | Decision | Value |
|---|----------|-------|
| 1 | Foundry project structure | Default: `src/`, `test/`, `script/` |
| 2 | OpenZeppelin version | v5.x latest stable (git submodule, `forge install` HEAD) |
| 3 | OZ remapping | `@openzeppelin/contracts/` (standard forge remapping) |
| 4 | Solidity version | `^0.8.20` |
| 5 | Dependency strategy | Git submodules (forge standard, no npm) |
| 6 | Optimizer | Enabled, 200 runs (standard balance of deploy cost vs call cost) |

## foundry.toml Target State

```toml
[profile.default]
src = "src"
out = "out"
libs = ["lib"]
solc = "0.8.20"
optimizer = true
optimizer_runs = 200

[profile.default.fuzz]
runs = 256
```

## remappings.txt Target State

```
@openzeppelin/contracts/=lib/openzeppelin-contracts/contracts/
```

## Claude's Discretion
- Exact `forge init` flags (e.g. `--no-git` vs default — use default since repo already has git)
- Whether to remove the boilerplate `Counter.sol` / `Counter.t.sol` that `forge init` generates (remove them — keep it clean for Phase 1.2)
- Commit message wording

## Deferred Ideas
- `via_ir = true` optimizer — not needed for hackathon scope
- Pinning to a specific OZ v5 tag — use HEAD of v5 branch for now, can pin if build breaks

---
Captured: 2026-03-18

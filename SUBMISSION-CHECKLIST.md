# AgentMesh - Synthesis Hackathon Submission Checklist

**Deadline:** March 22, 2026  
**Project:** AgentMesh - Decentralized agent coordination protocol  
**Team ID:** `e61883513690412ebb0131aa0c2b4a1f`  
**Participant ID:** `420f191ade9d4534928bffc6d3a8bc9a`

---

## Submission Requirements (from synthesis.md/submission/skill.md)

### ✅ Phase 1: Pre-Submission Setup

- [x] **Registered** - Credentials in `.synthesis` file
- [ ] **Self-Custody Transfer** - REQUIRED before publishing
  - Run: `POST /participants/me/transfer/init` with your wallet address
  - Then: `POST /participants/me/transfer/confirm`
  - Need wallet address first (see Wallet Setup guide: https://synthesis.md/wallet-setup/skill.md)
- [x] **Team** - Auto-created during registration
- [ ] **Browse tracks** - Need to select at least 1 track UUID

### 📋 Phase 2: Project Creation Data

All fields below are REQUIRED for `POST /projects`:

#### Core Fields
- [ ] **teamUUID** - Get from `GET /teams/:teamUUID`
- [x] **name** - "AgentMesh"
- [x] **description** - Already in README (see line 3-5)
- [ ] **problemStatement** - Need to write (see template below)
- [x] **repoURL** - `https://github.com/execute008/agentmesh`
- [ ] **trackUUIDs** - Browse via `GET /catalog`, need at least 1
  - Recommended tracks to check:
    - Best Infrastructure/Protocol
    - x402 Protocol Implementation
    - ERC-8004 Identity
    - Open Track (Synthesis)

#### Conversation Log
- [ ] **conversationLog** - REQUIRED, captures human-agent collaboration
  - Source material available:
    - `.planning/` directory (DOMAIN.md, ROADMAP.md, REQUIREMENTS.md, etc.)
    - `HANDOFF.md`
    - `PROMPT.md`
    - Git commit messages
  - Need to compile into narrative form

#### Submission Metadata (ALL REQUIRED)
- [ ] **submissionMetadata.agentFramework** - What framework was the PROJECT built with?
  - Options: `langchain`, `elizaos`, `mastra`, `vercel-ai-sdk`, `anthropic-agents-sdk`, `other`
  - AgentMesh is a protocol/smart contract → probably `other`
- [ ] **submissionMetadata.agentFrameworkOther** - If "other", describe it
- [ ] **submissionMetadata.agentHarness** - What harness was YOUR AGENT running on while building?
  - Options: `openclaw`, `claude-code`, `codex-cli`, `opencode`, `cursor`, `cline`, `aider`, `windsurf`, `copilot`, `other`
  - Likely: `openclaw` or `claude-code`
- [ ] **submissionMetadata.model** - AI model used (e.g., `claude-sonnet-4-6`, `claude-opus-4-6`)
  - From HANDOFF.md: Opus-4-6 for discuss/plan, Sonnet-4-6 for execute/verify
- [ ] **submissionMetadata.skills** - Agent skills ACTUALLY loaded (min 1)
  - Only list skills your agent had active, NOT skills you heard about
  - Examples: `web-search`, `react-best-practices`, etc.
- [ ] **submissionMetadata.tools** - External tools used (min 1)
  - Examples: `Foundry`, `Rust`, `Cargo`, `alloy`, `tokio`, `tungstenite`, `ngrok`, `Sepolia`
- [ ] **submissionMetadata.helpfulResources** - URLs actually consulted
  - Examples: Foundry docs, ERC-8004 spec, x402 protocol docs, Rust docs
- [ ] **submissionMetadata.helpfulSkills** - (OPTIONAL) Which skills were especially helpful and why
  - Format: `[{ "name": "skill-name", "reason": "what it helped with" }]`
- [ ] **submissionMetadata.intention** - Post-hackathon plans
  - Options: `continuing`, `exploring`, `one-time`
- [ ] **submissionMetadata.intentionNotes** - (OPTIONAL) Additional context
- [ ] **submissionMetadata.moltbookPostURL** - (OPTIONAL but encouraged) Moltbook announcement

#### Optional Fields
- [ ] **deployedURL** - Live demo URL (if deployed)
  - Currently: agents run locally on localhost
  - Could deploy to a server or use ngrok URLs
- [ ] **videoURL** - Demo video (STRONGLY RECOMMENDED)
  - Upload to YouTube/Loom
  - Show: problem → solution → live demo → on-chain interactions
- [ ] **pictures** - Screenshots/images
- [ ] **coverImageURL** - Project cover image

---

## 🎬 Phase 3: Demo Video Script

**Target length:** 3-5 minutes

### Suggested Structure:

1. **Hook (15 seconds)**
   - "What if AI agents could find each other, negotiate work, and settle payments—without any central server?"

2. **Problem Statement (30 seconds)**
   - Current state: Agents rely on centralized directories, marketplaces, or manual connections
   - Issues: Single points of failure, trusted intermediaries, no on-chain reputation

3. **Solution Overview (45 seconds)**
   - AgentMesh: Deployable smart contract + P2P WebSocket protocol
   - On-chain identity (ERC-8004), capability discovery, task escrow, reputation
   - x402 protocol for direct agent-to-agent communication

4. **Live Demo (2 minutes)**
   - Terminal 1: Deploy AgentRegistry.sol to Sepolia
   - Terminal 2: Register 3 agents (scraper, analyzer, publisher)
   - Terminal 3-5: Start WS servers for each agent
   - Show:
     - Analyzer queries registry → finds scraper's endpoint
     - Analyzer sends TaskRequest (x402) → scraper fetches URL
     - Scraper sends TaskComplete → analyzer processes
     - createTask + completeTask + releasePayment on-chain
     - ETH transferred, reputation updated (+5)
     - Publisher prints final result

5. **Tech Highlights (30 seconds)**
   - Solidity/Foundry (23 tests, all passing)
   - Rust CLI (deploy/scan/list/search/register)
   - P2P WebSocket agents (no relay server)
   - ERC-8004 identity, on-chain escrow

6. **Call to Action (15 seconds)**
   - "Try it yourself: github.com/execute008/agentmesh"
   - "Built at Synthesis Hackathon 2026"

### Recording Tips:
- Use OBS Studio or QuickTime for screen recording
- Show terminal output clearly (increase font size)
- Narrate what's happening in each step
- Keep it focused: problem → solution → demo → results

---

## 📝 Phase 4: Problem Statement Template

*(Required field, separate from description)*

```markdown
## Problem Statement

AI agents today lack a standardized way to discover and coordinate with each other without trusted intermediaries. When agents need external capabilities—web scraping, data analysis, content publishing—they rely on:

1. **Centralized directories** - Single points of failure, requires trust in directory operator
2. **Manual configuration** - Developers hard-code agent endpoints, no dynamic discovery
3. **Off-chain payments** - No on-chain settlement, no verifiable reputation

This creates friction for autonomous agent coordination and limits composability in multi-agent systems.

**Who is affected:**
- Developers building multi-agent systems
- Autonomous agents needing to discover and pay peers
- DAOs and protocols wanting to coordinate agent workforces

**What changes with AgentMesh:**
- Any agent can deploy a mesh (smart contract) and become discoverable
- Agents query the registry on-chain to find peers by capability
- Direct P2P WebSocket connections (x402 protocol) - no relay server
- On-chain task escrow and reputation - trustless payments and verifiable track records
```

---

## 🚀 Phase 5: Moltbook Post

**Before or after creating draft**, post on Moltbook:

Visit: https://www.moltbook.com  
Read skill: https://www.moltbook.com/skill.md

**Post should include:**
- What you're building (AgentMesh - decentralized agent coordination)
- Why it matters (trustless agent discovery + P2P communication + on-chain settlement)
- Tracks you're competing in (x402, ERC-8004, Infrastructure, Open)
- Link to repo: https://github.com/execute008/agentmesh
- Mention it's for Synthesis Hackathon 2026

**After posting:**
- Copy the post URL (e.g., `https://www.moltbook.com/posts/abc123`)
- Add to `submissionMetadata.moltbookPostURL`

---

## 🎯 Phase 6: Track Selection

**Browse available tracks:**
```bash
curl https://synthesis.devfolio.co/catalog?page=1&limit=20
```

**Recommended tracks to apply for:**
1. **Synthesis Open Track** - All projects auto-qualify
2. **Best Infrastructure/Protocol** - AgentMesh is core infrastructure
3. **x402 Protocol** - Direct P2P implementation ($5k prize mentioned in HANDOFF.md)
4. **ERC-8004 Identity** - Full ERC-8004 compliance ($8k prize mentioned)

Save the track UUIDs for submission.

---

## 📦 Phase 7: API Workflow

### 1. Get Team Info
```bash
curl https://synthesis.devfolio.co/teams/:teamUUID \
  -H "Authorization: Bearer sk-synth-161f197d711c875d49fcb3bfccab6547c18ea5f81b269ba8"
```

### 2. Transfer to Self-Custody (REQUIRED)
```bash
# Step 1: Initiate
curl -X POST https://synthesis.devfolio.co/participants/me/transfer/init \
  -H "Authorization: Bearer sk-synth-..." \
  -H "Content-Type: application/json" \
  -d '{"targetOwnerAddress": "0xYOUR_WALLET_ADDRESS"}'

# Step 2: Verify the address in response, then confirm
curl -X POST https://synthesis.devfolio.co/participants/me/transfer/confirm \
  -H "Authorization: Bearer sk-synth-..." \
  -H "Content-Type: application/json" \
  -d '{"transferToken": "tok_...", "targetOwnerAddress": "0xYOUR_WALLET_ADDRESS"}'
```

### 3. Create Project (Draft)
```bash
curl -X POST https://synthesis.devfolio.co/projects \
  -H "Authorization: Bearer sk-synth-..." \
  -H "Content-Type: application/json" \
  -d '{
    "teamUUID": "e61883513690412ebb0131aa0c2b4a1f",
    "name": "AgentMesh",
    "description": "Decentralized agent coordination protocol — deployable smart contract standard enabling any agent to create an on-chain mesh, discover peers via chain scanning, coordinate via x402 P2P WebSocket messaging, and settle payments with on-chain escrow and reputation.",
    "problemStatement": "...",
    "repoURL": "https://github.com/execute008/agentmesh",
    "trackUUIDs": ["track-uuid-1", "track-uuid-2"],
    "conversationLog": "...",
    "submissionMetadata": { ... },
    "videoURL": "https://youtube.com/watch?v=...",
    "deployedURL": "https://agentmesh-demo.vercel.app"
  }'
```

### 4. Publish (Team Admin Only)
```bash
curl -X POST https://synthesis.devfolio.co/projects/:projectUUID/publish \
  -H "Authorization: Bearer sk-synth-..."
```

### 5. Tweet
Post on Twitter/X tagging **@synthesis_md**:
```
Just shipped AgentMesh at @synthesis_md! 🚀

Decentralized agent coordination: on-chain identity (ERC-8004) + P2P messaging (x402) + task escrow

Agents can now discover each other, negotiate work, and settle payments—all without a central server.

Try it: github.com/execute008/agentmesh
```

---

## ⚠️ Critical Pre-Publishing Requirements

From the Synthesis submission skill, you **CANNOT publish** unless:

1. ✅ **Self-custody transfer completed** - MUST do this first
2. ✅ **Project has a name** - We have "AgentMesh"
3. ✅ **At least one track assigned** - Need to browse catalog and select
4. ✅ **Only team admin can publish** - Verify you're the admin

**After publishing:**
- Minor edits allowed until hackathon ends
- Project appears in public `GET /projects` listing
- Status changes from `draft` to `publish`

---

## 🎓 Resources Mentioned in Docs

- ERC-8004 spec: https://eips.ethereum.org/EIPS/eip-8004
- Wallet setup guide: https://synthesis.md/wallet-setup/skill.md
- Moltbook skill: https://www.moltbook.com/skill.md
- Prize catalog: https://synthesis.devfolio.co/catalog/prizes.md
- Themes & ideas: https://synthesis.md/themes.md
- Telegram group: https://nsb.dev/synthesis-updates (join this!)

---

## 📋 Next Immediate Actions

1. **Get a wallet address** for self-custody transfer
2. **Browse track catalog** and select 2-4 tracks
3. **Compile conversation log** from planning docs + HANDOFF.md
4. **Write problem statement** (use template above)
5. **Create Moltbook post** announcing the project
6. **Record demo video** (3-5 min, follow script above)
7. **Complete self-custody transfer**
8. **Create project draft** via API
9. **Publish** (team admin only)
10. **Tweet** tagging @synthesis_md

---

## Security Reminders

- ✅ Never share your API key in public
- ✅ Never commit secrets to git
- ✅ Verify wallet address in transfer init response before confirming
- ✅ Store API key securely (already in `.synthesis` file)

---

**Questions or blockers?** Ask your human to join the Telegram group: https://nsb.dev/synthesis-updates

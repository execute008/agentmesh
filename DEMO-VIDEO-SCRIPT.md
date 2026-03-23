# AgentMesh - Demo Video Script

**Target Length:** 3-5 minutes  
**Platform:** YouTube (unlisted or public)  
**Recording Tool:** OBS Studio / QuickTime / Loom

---

## Script Breakdown

### 0. Pre-Recording Setup (00:00 - not shown)
- Terminal 1: agentmesh-contracts (for deployment)
- Terminal 2: agentmesh-cli (for registration)
- Terminal 3-5: zeroclaw-agents (scraper, analyzer, publisher)
- Browser: BaseScan explorer ready
- Font size: 18pt+ for readability

---

### 1. Hook (00:00 - 00:15)

**Visual:** Title slide or IDE with README.md open

**Narration:**
> "What if AI agents could find each other, negotiate work, and settle payments—completely peer-to-peer, with no central server?"

**On-screen text:**
- "AgentMesh: Decentralized Agent Coordination"
- "Built at Synthesis Hackathon 2026"

---

### 2. Problem Statement (00:15 - 00:45)

**Visual:** Diagram or bullet points showing current limitations

**Narration:**
> "Today, agents rely on centralized directories to find peers, manual configuration to connect, and off-chain payments with no verifiable reputation. This limits autonomy, creates trust dependencies, and makes multi-agent systems fragile."

**On-screen:**
- ❌ Centralized discovery
- ❌ Manual configuration
- ❌ Off-chain payments
- ❌ No reputation

---

### 3. Solution Overview (00:45 - 01:30)

**Visual:** Architecture diagram (from README)

**Narration:**
> "AgentMesh solves this with a deployable smart contract standard. Any agent can deploy a mesh, register on-chain with their capabilities and WebSocket endpoint, and discover peers by scanning the chain. Communication is peer-to-peer via the x402 protocol—no relay server. Payments are handled with on-chain escrow, and reputation is tracked automatically."

**On-screen:**
```
ON-CHAIN (Ethereum)
└─ AgentRegistry.sol
   ├─ Identity (ERC-8004)
   ├─ Discovery (capabilities)
   ├─ Task Escrow
   └─ Reputation

OFF-CHAIN (P2P WebSocket)
└─ x402 Protocol
   ├─ TaskRequest
   └─ TaskComplete
```

**Key points:**
- ✅ On-chain identity (ERC-8004)
- ✅ Decentralized discovery
- ✅ P2P communication (x402)
- ✅ Trustless escrow
- ✅ On-chain reputation

---

### 4. Live Demo Part 1: Deploy & Register (01:30 - 02:30)

**Terminal 1: Deploy Contract**

**Narration:**
> "Let's see it in action. First, I'll deploy the AgentRegistry contract to Sepolia testnet."

**Command:**
```bash
cd agentmesh-contracts
forge script script/Deploy.s.sol:Deploy \
  --rpc-url $SEPOLIA_RPC_URL \
  --private-key $PRIVATE_KEY \
  --broadcast
```

**Highlight:**
- Show contract address output
- Open BaseScan link (optional)

**Terminal 2: Register Agents**

**Narration:**
> "Now I'll register three agents: a scraper that fetches web pages, an analyzer that orchestrates tasks, and a publisher that outputs results."

**Commands:**
```bash
cd agentmesh-cli

# Register scraper
./target/release/agentmesh register \
  --contract 0xCONTRACT_ADDRESS \
  --agent-id scraper-001 \
  --capabilities web-scraping,html-parsing \
  --price 10000000000000000 \
  --endpoint ws://127.0.0.1:8080

# Register analyzer
./target/release/agentmesh register \
  --contract 0xCONTRACT_ADDRESS \
  --agent-id analyzer-001 \
  --capabilities orchestration,task-routing \
  --price 20000000000000000 \
  --endpoint ws://127.0.0.1:8081

# Register publisher
./target/release/agentmesh register \
  --contract 0xCONTRACT_ADDRESS \
  --agent-id publisher-001 \
  --capabilities result-publishing,formatting \
  --price 5000000000000000 \
  --endpoint ws://127.0.0.1:8082
```

**Highlight:**
- Each registration shows "Agent registered successfully"
- Point out capabilities, price, WebSocket endpoint

---

### 5. Live Demo Part 2: Agent Coordination (02:30 - 04:00)

**Terminal 3-5: Start Agents**

**Narration:**
> "Now I'll start all three agents. Each runs its own WebSocket server."

**Commands:**
```bash
# Terminal 3
cd zeroclaw-agents
cargo run --bin scraper

# Terminal 4
cargo run --bin analyzer

# Terminal 5
cargo run --bin publisher
```

**Show output:**
- Each agent: "WebSocket server listening on ws://127.0.0.1:XXXX"

**Narration:**
> "Watch what happens next. The analyzer queries the on-chain registry, discovers the scraper's WebSocket endpoint, and sends a signed TaskRequest via x402."

**Highlight in analyzer terminal:**
```
🔍 Querying registry for capability: web-scraping
✅ Found scraper at ws://127.0.0.1:8080
📤 Sending TaskRequest: task_id=12345, target=https://example.com
```

**Highlight in scraper terminal:**
```
📥 Received TaskRequest from 0xAnalyzerWallet
🌐 Fetching https://example.com...
✅ Fetched 1234 bytes, extracting <title>...
📤 Sending TaskComplete: result="Example Domain"
```

**Highlight in analyzer terminal:**
```
📥 Received TaskComplete from scraper
💰 Creating on-chain task with 0.01 ETH escrow...
✅ Task created on-chain: tx=0xABC123...
✅ Marking task complete...
✅ Releasing payment to scraper...
✅ Reputation updated: scraper +5 points
```

**Highlight in publisher terminal:**
```
📥 Received result from analyzer
📄 Title: "Example Domain"
✅ Published result
```

---

### 6. On-Chain Verification (04:00 - 04:30)

**Visual:** BaseScan explorer

**Narration:**
> "Let's verify the on-chain transactions. Here's the task creation, completion, and payment release—all on Sepolia. The scraper's reputation increased by 5 points, and the payment was transferred automatically."

**Show:**
- Transaction list: `createTask`, `completeTask`, `releasePayment`
- Scraper's reputation value in contract state

---

### 7. Tech Stack & Testing (04:30 - 04:50)

**Visual:** Code editor with tests or forge test output

**Narration:**
> "AgentMesh is built with Solidity and Foundry for the smart contracts—all 23 tests passing—Rust and alloy for the CLI, and tokio plus tungstenite for the P2P WebSocket agents. Full ERC-8004 compliance for on-chain identity."

**On-screen:**
```bash
forge test
# Show: [PASS] 23 tests
```

---

### 8. Call to Action (04:50 - 05:00)

**Visual:** README with repo URL

**Narration:**
> "Try it yourself at github.com/execute008/agentmesh. All code is open source. Built at the Synthesis Hackathon 2026. Thanks for watching!"

**On-screen:**
- github.com/execute008/agentmesh
- Built at Synthesis Hackathon 2026
- @synthesis_md

---

## Recording Tips

1. **Increase font size** to 18-20pt in terminals
2. **Narrate as you go** - explain each command before running it
3. **Pause briefly** after each key output (2-3 seconds)
4. **Use terminal splits** (tmux or iTerm2) to show multiple agents
5. **Pre-prepare transactions** if network is slow
6. **Test run** the demo before recording
7. **Keep cursor visible** when highlighting text
8. **Screen resolution:** 1920x1080 or 1280x720

## Editing Checklist

- [ ] Remove dead air and long pauses
- [ ] Add title cards for each section
- [ ] Highlight terminal output with boxes/arrows (optional)
- [ ] Add background music (low volume, non-distracting)
- [ ] Add captions/subtitles (optional but helpful)
- [ ] Export in 1080p MP4

## Upload Checklist

- [ ] Upload to YouTube
- [ ] Title: "AgentMesh: Decentralized Agent Coordination (Synthesis Hackathon 2026)"
- [ ] Description: Include repo link, tech stack, Synthesis mention
- [ ] Tags: blockchain, AI agents, web3, x402, ERC-8004, Synthesis, Ethereum
- [ ] Thumbnail: Screenshot of architecture diagram or agent coordination
- [ ] Set to Public or Unlisted
- [ ] Copy URL for submission

---

**Good luck! 🎬🚀**

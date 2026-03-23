# AgentMesh - Synthesis Submission Guide

**Quick Reference:** How to submit AgentMesh to the Synthesis Hackathon

---

## ✅ What You Already Have

- [x] **Registration complete** - Credentials in `.synthesis`
- [x] **README with Synthesis mention** - Line 3: "Built at Synthesis Hackathon 2026"
- [x] **Working demo** - 3 agents coordinating via x402 + on-chain payments
- [x] **GitHub repo** - https://github.com/execute008/agentmesh
- [x] **Complete test suite** - 23 Foundry tests, all passing
- [x] **Documentation** - README, HANDOFF, planning docs

---

## 🎯 What You Need To Do (In Order)

### 1️⃣ Get a Wallet Address (5 minutes)

You need an Ethereum wallet address for **self-custody transfer** (required before publishing).

**Option A: Use MetaMask**
- Install MetaMask browser extension
- Create/import wallet
- Copy your address (starts with 0x)

**Option B: Use a CLI wallet**
- `cast wallet new` (from Foundry)
- Save the address

**Wallet Setup Guide:** https://synthesis.md/wallet-setup/skill.md

---

### 2️⃣ Compile Conversation Log (10 minutes)

```bash
./compile-conversation-log.sh
```

This creates `CONVERSATION-LOG.md` from:
- PROMPT.md
- .planning/ directory docs
- HANDOFF.md
- Git commit history
- Technical decision narratives

**Review and edit** the generated file to ensure it accurately captures your human-agent collaboration.

---

### 3️⃣ Browse Tracks & Select UUIDs (10 minutes)

```bash
./submit-project.sh
# Select option 2: Browse tracks
```

**Recommended tracks:**
- **Synthesis Open Track** - All projects auto-qualify
- **Best Infrastructure/Protocol** - AgentMesh is core infra
- **x402 Protocol** - Direct P2P implementation
- **ERC-8004 Identity** - Full compliance

Save the track UUIDs for step 6.

---

### 4️⃣ Transfer to Self-Custody (15 minutes)

**REQUIRED before publishing.**

```bash
./submit-project.sh
# Select option 4: Initiate self-custody transfer
# Enter your wallet address from Step 1

# Verify the address in the response matches what you entered
# Save the transfer token

# Then select option 5: Confirm self-custody transfer
# Enter the transfer token and address
```

**What this does:**
- Transfers your ERC-8004 agent NFT from custodial to your wallet
- Required for all team members before publishing
- Can only be done once (permanent)
- Takes ~2 minutes for on-chain confirmation

---

### 5️⃣ Record Demo Video (60-90 minutes)

Follow `DEMO-VIDEO-SCRIPT.md` for full instructions.

**Quick version:**
1. Set up 5 terminals (deploy, CLI, scraper, analyzer, publisher)
2. Record with OBS Studio or QuickTime
3. Follow the 5-minute script:
   - Hook (15s)
   - Problem (30s)
   - Solution (45s)
   - Live Demo (2 min)
   - Tech Stack (20s)
   - Call to Action (15s)
4. Upload to YouTube
5. Copy the URL for step 6

**Tips:**
- Increase terminal font to 18-20pt
- Narrate as you go
- Test run before recording
- Keep it under 5 minutes

---

### 6️⃣ Post on Moltbook (15 minutes)

**Visit:** https://www.moltbook.com  
**Read skill:** https://www.moltbook.com/skill.md

**Post should include:**
- What AgentMesh does (decentralized agent coordination)
- Why it matters (trustless discovery + P2P messaging + on-chain settlement)
- Tracks you're competing in
- Link to repo: https://github.com/execute008/agentmesh
- Mention Synthesis Hackathon 2026

**After posting:**
- Copy the post URL
- You'll add this in step 7

---

### 7️⃣ Create Project Draft (10 minutes)

```bash
./submit-project.sh
# Select option 6: Create project draft
```

**You'll be prompted for:**
- Track UUIDs (from step 3)
- Agent harness (e.g., `openclaw`, `claude-code`)
- AI model (e.g., `claude-sonnet-4-6`)
- Agent framework (for the project itself, probably `other` → "smart contract protocol")
- Skills used (be honest - only list what you actually loaded)
- Tools used (e.g., `Foundry`, `Rust`, `Cargo`, `alloy`, `tokio`, `ngrok`)
- Video URL (from step 5)
- Deployed URL (if you have one - optional)
- Moltbook URL (from step 6)
- Post-hackathon intention (`continuing`/`exploring`/`one-time`)

**The script will:**
- Read `PROBLEM-STATEMENT.md` automatically
- Read `CONVERSATION-LOG.md` automatically
- Fill in all required fields
- Create the draft via API

**Save the project UUID** from the response!

---

### 8️⃣ Review & Publish (10 minutes)

```bash
./submit-project.sh
# Select option 9: View project
# Enter your project UUID to review

# If everything looks good:
# Select option 8: Publish project
# Enter your project UUID
# Confirm with "yes"
```

**Requirements to publish:**
- ✅ Self-custody transfer complete (step 4)
- ✅ All team members have self-custody (check with your team)
- ✅ You are the team admin
- ✅ Video URL is final (minor edits allowed post-publish, but aim to be done)

---

### 9️⃣ Tweet About It (5 minutes)

**After publishing**, tweet:

```
Just shipped AgentMesh at @synthesis_md! 🚀

Decentralized agent coordination:
✅ On-chain identity (ERC-8004)
✅ P2P messaging (x402)
✅ Task escrow & reputation

Agents can now discover each other, negotiate work, and settle payments—no central server needed.

Try it: github.com/execute008/agentmesh

#Synthesis2026 #Web3 #AIAgents
```

**Important:**
- Must tag **@synthesis_md**
- Include link to repo
- Mention key features
- Use relevant hashtags

---

## 🔧 Helper Scripts Reference

| Script | Purpose |
|--------|---------|
| `compile-conversation-log.sh` | Compile dev log from planning docs |
| `submit-project.sh` | Interactive API submission helper |
| `DEMO-VIDEO-SCRIPT.md` | Full 5-minute video script |
| `PROBLEM-STATEMENT.md` | Problem statement for submission |
| `SUBMISSION-CHECKLIST.md` | Detailed checklist of all requirements |

---

## 📋 Submission Checklist

Print this and check off as you go:

- [ ] **Step 1:** Got wallet address for self-custody
- [ ] **Step 2:** Compiled conversation log (`CONVERSATION-LOG.md`)
- [ ] **Step 3:** Browsed tracks and saved UUIDs
- [ ] **Step 4:** Completed self-custody transfer
- [ ] **Step 5:** Recorded and uploaded demo video
- [ ] **Step 6:** Posted on Moltbook and saved URL
- [ ] **Step 7:** Created project draft via API
- [ ] **Step 8:** Reviewed project and published
- [ ] **Step 9:** Tweeted tagging @synthesis_md

---

## 🆘 Troubleshooting

### "Cannot publish - self-custody required"
→ Run `./submit-project.sh` option 4 & 5 to complete transfer

### "Only team admin can publish"
→ Check team info (option 1) - if you're not admin, ask your teammate to publish

### "Track UUID not found"
→ Re-browse tracks (option 2) and verify UUIDs are correct

### "Conversation log too long"
→ Edit `CONVERSATION-LOG.md` to remove verbose sections (keep key decisions)

### "Video URL invalid"
→ Make sure it's a public YouTube/Loom URL (not unlisted if judges need access)

---

## 📞 Support

- **Synthesis Telegram:** https://nsb.dev/synthesis-updates (join this!)
- **API Documentation:** https://synthesis.md/submission/skill.md
- **Wallet Setup:** https://synthesis.md/wallet-setup/skill.md

---

## ⏰ Timeline Estimate

| Task | Time | Can Parallelize? |
|------|------|------------------|
| Get wallet | 5 min | - |
| Compile log | 10 min | ✅ (while video renders) |
| Browse tracks | 10 min | ✅ |
| Self-custody | 15 min | - |
| Record video | 90 min | - |
| Moltbook post | 15 min | ✅ |
| Create draft | 10 min | - |
| Publish | 10 min | - |
| Tweet | 5 min | - |
| **Total** | **~2.5 hours** | **~2 hours with parallelization** |

---

**Good luck! 🚀**

Questions? Check `SUBMISSION-CHECKLIST.md` for detailed requirements or run `./submit-project.sh` for interactive help.

# AgentMesh — Sepolia Deployment Guide

## Prerequisites

- **Foundry** installed (`forge`, `cast`): <https://book.getfoundry.sh/getting-started/installation>
- **Solidity 0.8.30** (pinned in `foundry.toml`)

---

## 1. Create a Foundry Keystore

Private keys must **never** be stored in plaintext. Create an encrypted keystore:

```bash
cast wallet new deployer
```

This prompts for a password, creates an encrypted keystore at `~/.foundry/keystores/deployer`.

Confirm the address:

```bash
cast wallet address --account deployer
```

The password is re-prompted at deploy time (not stored anywhere).

---

## 2. Fund the Deployer Wallet

**Recommended:** 0.5 ETH on Sepolia (covers deploy ~0.01 ETH + 3 agent registrations ~0.03 ETH each + escrow buffer).

### Faucets

| Faucet | URL | Daily Limit |
|--------|-----|-------------|
| Google Cloud Faucet | <https://cloud.google.com/application/web3/faucet/ethereum/sepolia> | 0.05 ETH/day |
| Alchemy Sepolia Faucet | <https://sepoliafaucet.com> | 0.5 ETH/day (requires free account) |
| Infura Faucet | <https://www.infura.io/faucet/sepolia> | 0.5 ETH/day |

Verify balance:

```bash
cast balance $(cast wallet address --account deployer) --rpc-url $RPC_URL
```

---

## 3. Configure Environment

```bash
cp .env.example .env
```

Edit `.env` with your values. `DEPLOYER_ACCOUNT` defaults to `"deployer"` matching the keystore name created in step 1.

See [.env.example](../.env.example) for all required variables.

---

## 4. Deploy AgentRegistry

```bash
source .env
cd agentmesh-contracts
forge script script/Deploy.s.sol:Deploy \
  --rpc-url $RPC_URL \
  --account $DEPLOYER_ACCOUNT \
  --broadcast \
  -vvv
```

Enter your keystore password when prompted.

---

## 5. Verify on Etherscan

```bash
forge verify-contract DEPLOYED_ADDRESS \
  src/AgentRegistry.sol:AgentRegistry \
  --chain sepolia \
  --etherscan-api-key $ETHERSCAN_API_KEY \
  --constructor-args $(cast abi-encode "constructor(string)" "AgentMesh-Demo")
```

Replace `DEPLOYED_ADDRESS` with the address printed by the deploy script.

---

## Security Notes

- **Never commit `.env`** — it is in `.gitignore`
- **Never pass `--private-key` on the command line** (shell history leaks)
- Foundry keystore at `~/.foundry/keystores/deployer` is encrypted; back up and remember your password

## Foundry

**Foundry is a blazing fast, portable and modular toolkit for Ethereum application development written in Rust.**

Foundry consists of:

-   **Forge**: Ethereum testing framework (like Truffle, Hardhat and DappTools).
-   **Cast**: Swiss army knife for interacting with EVM smart contracts, sending transactions and getting chain data.
-   **Anvil**: Local Ethereum node, akin to Ganache, Hardhat Network.
-   **Chisel**: Fast, utilitarian, and verbose solidity REPL.

## Documentation

https://book.getfoundry.sh/

## Usage

### Build

```shell
$ forge build
```

### Test

```shell
$ forge test
```

### Format

```shell
$ forge fmt
```

### Gas Snapshots

```shell
$ forge snapshot
```

### Anvil

```shell
$ anvil
```

### Deploy

Deploy a new `AgentRegistry` mesh to Sepolia (or any EVM chain):

```shell
# 1. Set up RPC (free, no key required)
export SEPOLIA_RPC_URL=https://ethereum-sepolia-rpc.publicnode.com

# 2. Create & store deployer key (encrypted keystore, prompted once)
cast wallet import deployer --interactive

# 3. Fund the deployer address with testnet ETH from a faucet:
#    - https://www.alchemy.com/faucets/ethereum-sepolia
#    - https://cloud.google.com/application/web3/faucet/ethereum/sepolia

# 4. Deploy (MESH_NAME is required)
$ MESH_NAME=AgentMesh-Demo forge script script/Deploy.s.sol:Deploy \
    --rpc-url $SEPOLIA_RPC_URL \
    --account deployer \
    --broadcast
```

Deployment metadata is written to `meshes.json`.

#### Verify on Etherscan

```shell
$ forge verify-contract <deployed_address> src/AgentRegistry.sol:AgentRegistry \
    --chain sepolia \
    --etherscan-api-key $ETHERSCAN_API_KEY
```

### Cast

```shell
$ cast <subcommand>
```

### Help

```shell
$ forge --help
$ anvil --help
$ cast --help
```

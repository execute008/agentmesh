# AgentMesh - Development Conversation Log

**Hackathon:** Synthesis 2026
**Team:** execute008
**Duration:** March 18-22, 2026

---

## Project Inception

### Initial Prompt Analysis

# Project: AgentMesh - Decentralized Agent Coordination Protocol

## Overview
Build a **decentralized agent coordination protocol** with smart contracts on Ethereum (+ Solana stretch goal). Agents deploy their own AgentRegistry contracts, discover each other via on-chain scanning, coordinate via x402 off-chain messaging, and settle payments on-chain.

**Core Innovation:** AgentMesh is NOT a centralized server - it's a **deployable smart contract standard** that any agent can deploy. A CLI tool scans chains for all deployed meshes and enables cross-mesh discovery.

**Timeline:** 4 days (March 18-22, 2026)
**Target:** SYNTHESIS Hackathon (Open Track $20k + x402 $5k + Protocol Labs $8k)
**Tech Stack:** Solidity, Foundry, Rust (CLI), ethers-rs, WebSocket (x402)

---

## MUST-HAVE Requirements

### 1. ERC-8004 Agent Identity (Protocol Labs Track)
- On-chain agent registration with wallet signature
- Reputation tracking tied to wallet (portable across meshes)
- All critical actions logged on-chain for verification

### 2. x402 Protocol Compliance (OpenServ Track)
- Off-chain WebSocket messaging following x402 spec
- Agent-to-agent task delegation
- Service capability advertisements

### 3. Decentralized Architecture (Open Track)
- AgentRegistry smart contract (deployable by anyone)
- CLI scanner to discover all deployed meshes
- Multi-mesh agent discovery
- On-chain payment escrow

### 4. Working Demo
- 3 deployed AgentRegistry contracts (3 "meshes")
- 3 ZeroClaw agents coordinating across meshes
- End-to-end: discovery → task → payment → reputation update

---

## Architecture Overview

### On-Chain (Smart Contracts)
- **Agent Registry:** Store identity, capabilities, endpoint, reputation
- **Task Escrow:** Lock payment until task completion
- **Reputation System:** On-chain scores that follow wallets

### Off-Chain (x402 Peer-to-Peer WebSocket)
- **Direct P2P connections:** Each agent runs their own WebSocket server
- **On-chain endpoint discovery:** Query contract for agent's WebSocket URL
- **No central relay:** Agents connect directly to each other
- **Signed messages:** All x402 messages include wallet signatures
- **NAT traversal:** Use ngrok/cloudflare tunnels for demo (libp2p for production)

**How x402 P2P works:**
1. Agent A queries contract: "What's Agent B's endpoint?"
2. Contract returns: `wss://agent-b.example.com:8080`
3. Agent A opens **direct WebSocket connection** to Agent B
4. Agent A sends x402 message (signed with wallet)
5. Agent B verifies signature, processes message

**No relay server needed.** Fully peer-to-peer messaging with on-chain discovery.

### CLI Tool
- **Scanner:** Discover all AgentRegistry contracts on-chain
- **Query:** Search for agents across all meshes
- **Deploy:** Launch your own mesh contract
- **Register:** Join existing meshes

---

## Phase 1: Smart Contract Development (Day 1)

### Task 1.1: Project Setup

```bash
# Initialize Foundry project
forge init agentmesh-contracts
cd agentmesh-contracts

# Install dependencies
forge install OpenZeppelin/openzeppelin-contracts
```

**Project structure:**
```
agentmesh-contracts/
├── src/
│   ├── AgentRegistry.sol
│   └── interfaces/
│       └── IAgentRegistry.sol
├── test/
│   ├── AgentRegistry.t.sol
│   └── AgentRegistryIntegration.t.sol
├── script/
│   ├── Deploy.s.sol
│   └── Interact.s.sol
└── foundry.toml
```

**Acceptance Criteria:**
- [ ] Foundry project compiles
- [ ] OpenZeppelin imported

---

### Task 1.2: AgentRegistry Smart Contract (src/AgentRegistry.sol)

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title AgentRegistry
 * @notice ERC-8004 compliant decentralized agent coordination registry
 * @dev Each deployment creates an independent "mesh" - agents can deploy their own
 */
contract AgentRegistry {
    /*//////////////////////////////////////////////////////////////
                                 STRUCTS
    //////////////////////////////////////////////////////////////*/
    
    struct Agent {
        address wallet;
        string agentId;
        string[] capabilities;
        uint256 pricePerTask; // in wei
        uint8 reputation; // 0-100
        string endpoint; // WebSocket URL for x402 messages
        uint256 registeredAt;
        uint256 lastHeartbeat;
        bool active;
    }
    
    struct Task {
        address requester;
        address executor;
        string taskId;
        uint256 escrowAmount;
        bool completed;
        bool released;
        uint256 createdAt;
    }
    
    /*//////////////////////////////////////////////////////////////
                                 STORAGE
    //////////////////////////////////////////////////////////////*/
    
    mapping(address => Agent) public agents;
    mapping(string => address) public agentIdToWallet;
    mapping(bytes32 => Task) public tasks;
    
    address[] public registeredAgents;
    
    // Mesh metadata
    string public meshName;
    address public meshOwner;
    
    /*//////////////////////////////////////////////////////////////
                                 EVENTS
    //////////////////////////////////////////////////////////////*/
    
    event MeshCreated(string indexed name, address indexed owner);
    event AgentRegistered(address indexed wallet, string agentId, string[] capabilities);
    event AgentUpdated(address indexed wallet, string[] capabilities);
    event AgentDeactivated(address indexed wallet);
    event TaskCreated(bytes32 indexed taskId, address requester, address executor, uint256 amount);
    event TaskCompleted(bytes32 indexed taskId);
    event PaymentReleased(bytes32 indexed taskId, uint256 amount);
    event ReputationUpdated(address indexed wallet, uint8 newReputation);
    
    /*//////////////////////////////////////////////////////////////
                               CONSTRUCTOR
    //////////////////////////////////////////////////////////////*/
    
    constructor(string memory _meshName) {
        meshName = _meshName;
        meshOwner = msg.sender;
        emit MeshCreated(_meshName, msg.sender);
    }
    
    /*//////////////////////////////////////////////////////////////
                            AGENT REGISTRATION
    //////////////////////////////////////////////////////////////*/
    
    /**
     * @notice Register agent with ERC-8004 identity
     * @dev Agent must not already be registered
     */
    function registerAgent(
        string memory _agentId,
        string[] memory _capabilities,
        uint256 _pricePerTask,
        string memory _endpoint
    ) external {
        require(bytes(agents[msg.sender].agentId).length == 0, "Already registered");
        require(bytes(_agentId).length > 0, "Invalid agent ID");
        require(_capabilities.length > 0, "No capabilities");
        
        agents[msg.sender] = Agent({
            wallet: msg.sender,
            agentId: _agentId,
            capabilities: _capabilities,
            pricePerTask: _pricePerTask,
            reputation: 50, // Starting reputation
            endpoint: _endpoint,
            registeredAt: block.timestamp,
            lastHeartbeat: block.timestamp,
            active: true
        });
        
        agentIdToWallet[_agentId] = msg.sender;
        registeredAgents.push(msg.sender);
        
        emit AgentRegistered(msg.sender, _agentId, _capabilities);
    }
    
    /**
     * @notice Update agent capabilities and pricing
     */
    function updateAgent(
        string[] memory _capabilities,
        uint256 _pricePerTask,
        string memory _endpoint
    ) external {
        require(agents[msg.sender].active, "Not registered");
        
        agents[msg.sender].capabilities = _capabilities;
        agents[msg.sender].pricePerTask = _pricePerTask;
        agents[msg.sender].endpoint = _endpoint;
        
        emit AgentUpdated(msg.sender, _capabilities);
    }
    
    /**
     * @notice Update heartbeat timestamp
     */
    function heartbeat() external {
        require(agents[msg.sender].active, "Not registered");
        agents[msg.sender].lastHeartbeat = block.timestamp;
    }
    
    /**
     * @notice Deactivate agent
     */
    function deactivate() external {
        require(agents[msg.sender].active, "Not active");
        agents[msg.sender].active = false;
        emit AgentDeactivated(msg.sender);
    }
    
    /*//////////////////////////////////////////////////////////////
                            TASK & ESCROW
    //////////////////////////////////////////////////////////////*/
    
    /**
     * @notice Create task with escrowed payment
     * @dev Payment locked until requester releases it
     */
    function createTask(
        string memory _taskId,
        address _executor
    ) external payable {
        require(msg.value > 0, "Must escrow payment");
        require(agents[_executor].active, "Executor not registered");
        require(bytes(_taskId).length > 0, "Invalid task ID");
        
        bytes32 taskHash = keccak256(abi.encodePacked(_taskId, msg.sender));
        require(tasks[taskHash].requester == address(0), "Task ID already used");
        
        tasks[taskHash] = Task({
            requester: msg.sender,
            executor: _executor,
            taskId: _taskId,
            escrowAmount: msg.value,
            completed: false,
            released: false,
            createdAt: block.timestamp
        });
        
        emit TaskCreated(taskHash, msg.sender, _executor, msg.value);
    }
    
    /**
     * @notice Mark task as completed (executor only)
     */
    function completeTask(string memory _taskId) external {
        bytes32 taskHash = keccak256(abi.encodePacked(_taskId, msg.sender));
        Task storage task = tasks[taskHash];
        
        // Allow both requester and executor to mark complete
        require(
            task.executor == msg.sender || task.requester == msg.sender,
            "Not authorized"
        );
        require(!task.completed, "Already completed");
        
        task.completed = true;
        emit TaskCompleted(taskHash);
    }
    
    /**
     * @notice Release escrowed payment (requester confirms completion)
     */
    function releasePayment(string memory _taskId, address _requester) external {
        bytes32 taskHash = keccak256(abi.encodePacked(_taskId, _requester));
        Task storage task = tasks[taskHash];
        
        require(task.requester == msg.sender, "Not task requester");
        require(task.completed, "Task not completed");
        require(!task.released, "Already released");
        
        task.released = true;
        
        // Update executor reputation (+5, max 100)
        agents[task.executor].reputation = uint8(
            _min(100, uint256(agents[task.executor].reputation) + 5)
        );
        
        emit ReputationUpdated(task.executor, agents[task.executor].reputation);
        
        // Transfer payment
        (bool success, ) = payable(task.executor).call{value: task.escrowAmount}("");
        require(success, "Transfer failed");
        
        emit PaymentReleased(taskHash, task.escrowAmount);
    }
    
    /*//////////////////////////////////////////////////////////////
                               QUERIES
    //////////////////////////////////////////////////////////////*/
    
    /**
     * @notice Search agents by capability
     */
    function searchByCapability(string memory capability) 
        external 
        view 
        returns (address[] memory) 
    {
        uint count = 0;
        
        // First pass: count matches
        for (uint i = 0; i < registeredAgents.length; i++) {
            Agent storage agent = agents[registeredAgents[i]];
            if (agent.active && _hasCapability(agent.capabilities, capability)) {
                count++;
            }
        }
        
        // Second pass: build result array
        address[] memory results = new address[](count);
        uint index = 0;
        for (uint i = 0; i < registeredAgents.length; i++) {
            Agent storage agent = agents[registeredAgents[i]];
            if (agent.active && _hasCapability(agent.capabilities, capability)) {
                results[index] = registeredAgents[i];
                index++;
            }
        }
        
        return results;
    }
    
    /**
     * @notice Get all active agents
     */
    function getAllAgents() external view returns (address[] memory) {
        uint count = 0;
        for (uint i = 0; i < registeredAgents.length; i++) {
            if (agents[registeredAgents[i]].active) {
                count++;
            }
        }
        
        address[] memory active = new address[](count);
        uint index = 0;
        for (uint i = 0; i < registeredAgents.length; i++) {
            if (agents[registeredAgents[i]].active) {
                active[index] = registeredAgents[i];
                index++;
            }
        }
        
        return active;
    }
    
    /**
     * @notice Get agent details
     */
    function getAgent(address wallet) external view returns (Agent memory) {
        return agents[wallet];
    }
    
    /**
     * @notice Get total registered agents
     */
    function getAgentCount() external view returns (uint256) {
        return registeredAgents.length;
    }
    
    /*//////////////////////////////////////////////////////////////
                            INTERNAL HELPERS
    //////////////////////////////////////////////////////////////*/
    
    function _hasCapability(string[] storage capabilities, string memory target) 
        private 
        view 
        returns (bool) 
    {
        for (uint i = 0; i < capabilities.length; i++) {
            if (keccak256(bytes(capabilities[i])) == keccak256(bytes(target))) {
                return true;
            }
        }
        return false;
    }
    
    function _min(uint256 a, uint256 b) private pure returns (uint256) {
        return a < b ? a : b;
    }
}
```

**Acceptance Criteria:**
- [ ] Contract compiles
- [ ] All functions have natspec comments
- [ ] Events emitted for CLI scanning

---

### Task 1.3: Smart Contract Tests (test/AgentRegistry.t.sol)

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/AgentRegistry.sol";

contract AgentRegistryTest is Test {
    AgentRegistry registry;
    address agent1 = address(0x1);
    address agent2 = address(0x2);
    
    function setUp() public {
        registry = new AgentRegistry("TestMesh");
    }
    
    function testRegisterAgent() public {
        vm.prank(agent1);
        
        string[] memory caps = new string[](1);
        caps[0] = "web-scraping";
        
        registry.registerAgent(
            "scraper-001",
            caps,
            0.01 ether,
            "ws://localhost:8080"
        );
        
        AgentRegistry.Agent memory agent = registry.getAgent(agent1);
        assertEq(agent.agentId, "scraper-001");
        assertEq(agent.reputation, 50);
    }
    
    function testCreateTaskWithEscrow() public {
        // Register executor
        vm.prank(agent2);
        string[] memory caps = new string[](1);
        caps[0] = "analysis";
        registry.registerAgent("analyzer-001", caps, 0.01 ether, "ws://localhost:8081");
        
        // Create task with escrow
        vm.deal(agent1, 1 ether);
        vm.prank(agent1);
        registry.createTask{value: 0.1 ether}("task-123", agent2);
        
        // Verify escrow locked
        assertEq(address(registry).balance, 0.1 ether);
    }
    
    function testPaymentRelease() public {
        // Setup
        vm.prank(agent2);
        string[] memory caps = new string[](1);
        caps[0] = "analysis";
        registry.registerAgent("analyzer-001", caps, 0.01 ether, "ws://localhost:8081");
        
        vm.deal(agent1, 1 ether);
        vm.prank(agent1);
        registry.createTask{value: 0.1 ether}("task-456", agent2);
        
        // Complete task
        vm.prank(agent2);
        registry.completeTask("task-456");
        
        // Release payment
        uint256 balanceBefore = agent2.balance;
        vm.prank(agent1);
        registry.releasePayment("task-456", agent1);
        
        assertEq(agent2.balance - balanceBefore, 0.1 ether);
        
        // Check reputation increased
        AgentRegistry.Agent memory agent = registry.getAgent(agent2);
        assertEq(agent.reputation, 55); // 50 + 5
    }
    
    function testSearchByCapability() public {
        // Register multiple agents
        vm.prank(agent1);
        string[] memory caps1 = new string[](1);
        caps1[0] = "web-scraping";
        registry.registerAgent("scraper-001", caps1, 0.01 ether, "ws://localhost:8080");
        
        vm.prank(agent2);
        string[] memory caps2 = new string[](1);
        caps2[0] = "web-scraping";
        registry.registerAgent("scraper-002", caps2, 0.02 ether, "ws://localhost:8081");
        
        // Search
        address[] memory results = registry.searchByCapability("web-scraping");
        assertEq(results.length, 2);
    }
}
```

**Acceptance Criteria:**
- [ ] All tests pass (`forge test`)
- [ ] >80% code coverage

---

### Task 1.4: Deployment Script (script/Deploy.s.sol)

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/AgentRegistry.sol";

contract DeployScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);
        
        string memory meshName = vm.envString("MESH_NAME");
        
        AgentRegistry registry = new AgentRegistry(meshName);
        
        console.log("AgentRegistry deployed to:", address(registry));
        console.log("Mesh name:", meshName);
        
        vm.stopBroadcast();
    }
}
```

**Deploy to Sepolia:**
```bash
# .env
PRIVATE_KEY=0x...
SEPOLIA_RPC_URL=https://eth-sepolia.g.alchemy.com/v2/...
MESH_NAME=AgentMesh-Alpha

# Deploy
forge script script/Deploy.s.sol:DeployScript --rpc-url $SEPOLIA_RPC_URL --broadcast --verify
```

**Acceptance Criteria:**
- [ ] Contract deployed to Sepolia testnet
- [ ] Address logged
- [ ] Verified on Etherscan

---

## Phase 2: CLI Tool Development (Day 2)

### Task 2.1: Rust CLI Project Setup

```bash
cargo new agentmesh-cli
cd agentmesh-cli

# Add dependencies to Cargo.toml
```

**Cargo.toml:**
```toml
[package]
name = "agentmesh-cli"
version = "0.1.0"
edition = "2021"

[dependencies]
ethers = "2.0"
tokio = { version = "1", features = ["full"] }
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
hex = "0.4"
dotenv = "0.15"
```

**Acceptance Criteria:**
- [ ] Cargo project compiles
- [ ] CLI skeleton with clap

---

### Task 2.2: Contract Scanner (src/scanner.rs)

```rust
use ethers::{
    providers::{Provider, Http, Middleware},
    contract::{abigen, Contract},
    core::types::{Address, Filter, H256},
};
use anyhow::Result;

// Generate contract bindings from ABI
abigen!(
    AgentRegistry,
    "../agentmesh-contracts/out/AgentRegistry.sol/AgentRegistry.json"
);

pub struct MeshScanner {
    provider: Provider<Http>,
}

impl MeshScanner {
    pub fn new(rpc_url: &str) -> Result<Self> {
        let provider = Provider::<Http>::try_from(rpc_url)?;
        Ok(Self { provider })
    }
    
    /// Discover all deployed AgentRegistry contracts
    /// Scans for MeshCreated events from genesis
    pub async fn discover_meshes(&self, from_block: u64) -> Result<Vec<MeshInfo>> {
        // Event signature: MeshCreated(string,address)
        let event_signature = "MeshCreated(string,address)";
        let topic = H256::from_slice(&ethers::utils::keccak256(event_signature.as_bytes()));
        
        let filter = Filter::new()
            .from_block(from_block)
            .topic0(topic);
        
        let logs = self.provider.get_logs(&filter).await?;
        
        let mut meshes = Vec::new();
        for log in logs {
            // Parse mesh metadata from event
            let mesh = MeshInfo {
                contract_address: log.address,
                name: "".to_string(), // Parse from log data
                owner: Address::zero(), // Parse from log data
                deployed_at: 0, // Get from block timestamp
            };
            meshes.push(mesh);
        }
        
        Ok(meshes)
    }
    
    /// Query agents in a specific mesh
    pub async fn query_mesh(&self, contract_addr: Address) -> Result<Vec<AgentInfo>> {
        let contract = AgentRegistry::new(contract_addr, Arc::new(&self.provider));
        
        let agent_addrs = contract.get_all_agents().call().await?;
        
        let mut agents = Vec::new();
        for addr in agent_addrs {
            let agent_data = contract.get_agent(addr).call().await?;
            
            agents.push(AgentInfo {
                wallet: agent_data.wallet,
                agent_id: agent_data.agent_id,
                capabilities: agent_data.capabilities,
                price_per_task: agent_data.price_per_task,
                reputation: agent_data.reputation,
                endpoint: agent_data.endpoint,
            });
        }
        
        Ok(agents)
    }
    
    /// Search for capability across all meshes
    pub async fn global_search(&self, capability: &str, meshes: &[Address]) -> Result<Vec<(Address, Vec<AgentInfo>)>> {
        let mut results = Vec::new();
        
        for mesh_addr in meshes {
            let contract = AgentRegistry::new(*mesh_addr, Arc::new(&self.provider));
            
            let agent_addrs = contract.search_by_capability(capability.to_string()).call().await?;
            
            if !agent_addrs.is_empty() {
                let mut agents = Vec::new();
                for addr in agent_addrs {
                    let agent_data = contract.get_agent(addr).call().await?;
                    agents.push(AgentInfo {
                        wallet: agent_data.wallet,
                        agent_id: agent_data.agent_id,
                        capabilities: agent_data.capabilities,
                        price_per_task: agent_data.price_per_task,
                        reputation: agent_data.reputation,
                        endpoint: agent_data.endpoint,
                    });
                }
                results.push((*mesh_addr, agents));
            }
        }
        
        Ok(results)
    }
}

#[derive(Debug, Clone)]
pub struct MeshInfo {
    pub contract_address: Address,
    pub name: String,
    pub owner: Address,
    pub deployed_at: u64,
}

#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub wallet: Address,
    pub agent_id: String,
    pub capabilities: Vec<String>,
    pub price_per_task: U256,
    pub reputation: u8,
    pub endpoint: String,
}
```

**Acceptance Criteria:**
- [ ] Can discover deployed contracts
- [ ] Can query agents in a mesh
- [ ] Can search across multiple meshes

---

### Task 2.3: CLI Commands (src/main.rs)

```rust
use clap::{Parser, Subcommand};
use ethers::signers::{LocalWallet, Signer};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "agentmesh")]
#[command(about = "Decentralized agent coordination CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan chain for AgentMesh contracts
    Scan {
        #[arg(long, default_value = "sepolia")]
        chain: String,
        
        #[arg(long, default_value = "0")]
        from_block: u64,
    },
    
    /// List agents in a mesh
    List {
        #[arg(long)]
        contract: String,
    },
    
    /// Search for capability across all meshes
    Search {
        capability: String,
        
        #[arg(long)]
        meshes: Option<Vec<String>>,
    },
    
    /// Register agent on a mesh
    Register {
        #[arg(long)]
        contract: String,
        
        #[arg(long)]
        agent_id: String,
        
        #[arg(long)]
        capabilities: Vec<String>,
        
        #[arg(long)]
        price: String, // wei
        
        #[arg(long)]
        endpoint: String,
    },
    
    /// Deploy new AgentRegistry mesh
    Deploy {
        #[arg(long)]
        name: String,
        
        #[arg(long, default_value = "sepolia")]
        chain: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Scan { chain, from_block } => {
            let rpc_url = get_rpc_url(&chain)?;
            let scanner = MeshScanner::new(&rpc_url)?;
            
            println!("🔍 Scanning {} from block {}...", chain, from_block);
            let meshes = scanner.discover_meshes(from_block).await?;
            
            println!("\n📊 Found {} AgentMesh contracts:", meshes.len());
            for mesh in meshes {
                println!("  • {} at {}", mesh.name, mesh.contract_address);
            }
        }
        
        Commands::List { contract } => {
            let addr: Address = contract.parse()?;
            let rpc_url = std::env::var("RPC_URL")?;
            let scanner = MeshScanner::new(&rpc_url)?;
            
            let agents = scanner.query_mesh(addr).await?;
            
            println!("\n🤖 Agents in mesh {}:", contract);
            for agent in agents {
                println!("  • {} ({}) - {} caps, rep: {}",
                    agent.agent_id,
                    agent.wallet,
                    agent.capabilities.len(),
                    agent.reputation
                );
            }
        }
        
        Commands::Search { capability, meshes } => {
            // Global search across meshes
            println!("🔎 Searching for '{}' capability...", capability);
            // Implementation...
        }
        
        Commands::Register { contract, agent_id, capabilities, price, endpoint } => {
            // Register agent on mesh
            let wallet = load_wallet()?;
            println!("✍️  Registering {} on mesh {}...", agent_id, contract);
            // Implementation...
        }
        
        Commands::Deploy { name, chain } => {
            // Deploy new mesh contract
            println!("🚀 Deploying '{}' mesh on {}...", name, chain);
            // Implementation...
        }
    }
    
    Ok(())
}

fn get_rpc_url(chain: &str) -> Result<String> {
    match chain {
        "sepolia" => Ok(std::env::var("SEPOLIA_RPC_URL")?),
        "mainnet" => Ok(std::env::var("MAINNET_RPC_URL")?),
        _ => Err(anyhow::anyhow!("Unsupported chain")),
    }
}

fn load_wallet() -> Result<LocalWallet> {
    let private_key = std::env::var("PRIVATE_KEY")?;
    Ok(private_key.parse()?)
}
```

**Example Usage:**
```bash
# Scan Sepolia for all meshes
agentmesh scan --chain sepolia

# List agents in specific mesh
agentmesh list --contract 0xABC123...

# Search across all known meshes
agentmesh search "web-scraping" --meshes 0xABC... 0xDEF...

# Register on a mesh
agentmesh register \
  --contract 0xABC123... \
  --agent-id scraper-001 \
  --capabilities web-scraping \
  --price 10000000000000000 \
  --endpoint ws://localhost:8080

# Deploy your own mesh
agentmesh deploy --name "MyMesh" --chain sepolia
```

**Acceptance Criteria:**
- [ ] All commands work
- [ ] JSON output option
- [ ] Error handling

---

## Phase 3: x402 + ZeroClaw Integration (Day 3)

### Task 3.1: x402 Message Types (agentmesh-types/src/x402.rs)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct X402Message {
    pub version: String,
    pub from: String, // wallet address
    pub to: String,
    pub message_id: String,
    pub timestamp: i64,
    pub message_type: X402MessageType,
    pub payload: serde_json::Value,
    pub signature: String, // ERC-8004 signature
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum X402MessageType {
    Discover,
    DiscoverResponse,
    TaskRequest,
    TaskAccept,
    TaskProgress,
    TaskComplete,
    TaskFailed,
    PaymentConfirm,
}
```

---

### Task 3.2: Agent WebSocket Server (Each Agent Runs This)

**Each agent runs their own WebSocket server** for peer-to-peer x402 messaging.

```rust
use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    routing::get,
    Router,
};
use tokio::sync::mpsc;

/// Each agent runs this server to receive x402 messages
#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<X402Message>(100);
    
    // Start WebSocket server to accept incoming connections
    let app = Router::new()
        .route("/", get(ws_handler))
        .with_state(tx.clone());
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    
    println!("🎧 Agent WebSocket server listening on 0.0.0.0:8080");
    
    // Handle incoming x402 messages
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            handle_x402_message(msg).await;
        }
    });
    
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(tx): State<mpsc::Sender<X402Message>>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_incoming_connection(socket, tx))
}

async fn handle_incoming_connection(
    mut socket: WebSocket,
    tx: mpsc::Sender<X402Message>,
) {
    use futures::StreamExt;
    
    while let Some(Ok(msg)) = socket.next().await {
        if let axum::extract::ws::Message::Text(text) = msg {
            // Parse x402 message
            if let Ok(x402_msg) = serde_json::from_str::<X402Message>(&text) {
                // Verify signature
                if verify_x402_signature(&x402_msg) {
                    tx.send(x402_msg).await.ok();
                } else {
                    tracing::warn!("Invalid signature from {}", x402_msg.from);
                }
            }
        }
    }
}

async fn handle_x402_message(msg: X402Message) {
    match msg.message_type {
        X402MessageType::TaskRequest => {
            // Process task request
            println!("📨 Received task request: {}", msg.message_id);
        }
        X402MessageType::TaskComplete => {
            // Handle completion
            println!("✅ Task completed: {}", msg.message_id);
        }
        _ => {}
    }
}

fn verify_x402_signature(msg: &X402Message) -> bool {
    // Verify wallet signature on message payload
    // Use ethers::utils::verify_message or similar
    true // Placeholder
}
```

**Acceptance Criteria:**
- [ ] Each agent can run WebSocket server
- [ ] Server accepts incoming x402 messages
- [ ] Signatures verified before processing
- [ ] No central relay needed

---

### Task 3.2b: Peer-to-Peer Client (Connect to Other Agents)

**How agents connect to each other:**

```rust
use ethers::providers::{Provider, Http};
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Connect to another agent and send x402 message
pub async fn send_to_agent(
    target_wallet: Address,
    mesh_contract: Address,
    message: X402Message,
) -> Result<()> {
    // 1. Query contract for target agent's endpoint
    let provider = Provider::<Http>::try_from(std::env::var("RPC_URL")?)?;
    let contract = AgentRegistry::new(mesh_contract, Arc::new(provider));
    
    let target_agent = contract.get_agent(target_wallet).call().await?;
    let endpoint = target_agent.endpoint; // e.g., "wss://agent-scraper.ngrok.io"
    
    // 2. Connect directly to target agent's WebSocket server
    let (mut ws_stream, _) = connect_async(endpoint).await?;
    
    // 3. Sign message with our wallet
    let signed_msg = sign_x402_message(message, &our_wallet)?;
    
    // 4. Send x402 message
    let json = serde_json::to_string(&signed_msg)?;
    ws_stream.send(Message::Text(json)).await?;
    
    Ok(())
}

fn sign_x402_message(mut msg: X402Message, wallet: &LocalWallet) -> Result<X402Message> {
    use sha3::{Digest, Keccak256};
    
    // Hash payload
    let payload_str = serde_json::to_string(&msg.payload)?;
    let hash = Keccak256::digest(payload_str.as_bytes());
    
    // Sign with wallet
    let signature = wallet.sign_message(&hash[..]).await?;
    msg.signature = format!("0x{}", hex::encode(signature.to_vec()));
    
    Ok(msg)
}
```

**Flow:**
1. Agent A wants to send task to Agent B
2. Query `AgentRegistry` contract: "What's B's endpoint?"
3. Contract returns: `wss://agent-b.ngrok.io`
4. Agent A opens WebSocket connection **directly** to B
5. Send x402 message (wallet-signed)
6. Agent B verifies signature, processes task

**Acceptance Criteria:**
- [ ] Agents can query contract for endpoints
- [ ] Direct WebSocket connections work
- [ ] Messages signed and verified
- [ ] No relay needed

---

### Task 3.3: ZeroClaw Agent Integration

**Create 3 agents (web-scraper, analyzer, publisher):**

Each agent:
1. Generates wallet
2. Runs WebSocket server (for receiving x402 messages)
3. Exposes via ngrok (for demo) or public IP
4. Registers on-chain with endpoint URL
5. Queries contract to find other agents
6. Connects **directly** to other agents' WebSocket servers

**Example: Web Scraper Agent**
```rust
use agentmesh::{send_to_agent, X402Message, X402MessageType};
use ethers::signers::LocalWallet;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    let wallet: LocalWallet = env::var("PRIVATE_KEY")?.parse()?;
    let mesh_contract: Address = env::var("MESH_CONTRACT")?.parse()?;
    
    // 1. Start our WebSocket server (receive messages)
    let (tx, mut rx) = mpsc::channel::<X402Message>(100);
    tokio::spawn(async move {
        start_websocket_server(tx).await;
    });
    
    // 2. Expose via ngrok (for demo)
    println!("🌐 Start ngrok: ngrok http 8080");
    println!("📝 Then register with: wss://YOUR-NGROK-URL.ngrok.io");
    
    // 3. Register on-chain
    // (Use CLI: agentmesh register --contract $MESH --endpoint wss://abc.ngrok.io ...)
    
    // 4. Handle incoming x402 messages
    while let Some(msg) = rx.recv().await {
        match msg.message_type {
            X402MessageType::TaskRequest => {
                println!("📨 Received task request from {}", msg.from);
                
                // Do work
                let url = msg.payload["url"].as_str().unwrap();
                let html = scrape_url(url).await?;
                
                // Send x402 TaskComplete DIRECTLY to requester
                let response = X402Message {
                    version: "1.0".to_string(),
                    from: format!("{:?}", wallet.address()),
                    to: msg.from.clone(),
                    message_id: uuid::Uuid::new_v4().to_string(),
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64,
                    message_type: X402MessageType::TaskComplete,
                    payload: serde_json::json!({ "html": html }),
                    signature: "".to_string(), // Will be signed in send_to_agent
                };
                
                // Query contract for requester's endpoint and send
                let requester_wallet: Address = msg.from.parse()?;
                send_to_agent(requester_wallet, mesh_contract, response).await?;
                
                println!("✅ Task completed, sent response to {}", msg.from);
            }
            _ => {}
        }
    }
    
    Ok(())
}

async fn scrape_url(url: &str) -> Result<String> {
    let html = reqwest::get(url).await?.text().await?;
    Ok(html)
}
```

**Example: Analyzer Agent (Initiates Task)**
```rust
#[tokio::main]
async fn main() -> Result<()> {
    let wallet: LocalWallet = env::var("PRIVATE_KEY")?.parse()?;
    let mesh_contract: Address = env::var("MESH_CONTRACT")?.parse()?;
    
    // 1. Start our WebSocket server
    let (tx, mut rx) = mpsc::channel::<X402Message>(100);
    tokio::spawn(async move {
        start_websocket_server(tx).await;
    });
    
    // 2. Search for web-scraping agents
    let provider = Provider::<Http>::try_from(env::var("RPC_URL")?)?;
    let contract = AgentRegistry::new(mesh_contract, Arc::new(provider));
    
    let scrapers = contract.search_by_capability("web-scraping".to_string()).call().await?;
    let scraper_wallet = scrapers[0];
    
    println!("🔍 Found scraper: {}", scraper_wallet);
    
    // 3. Send TaskRequest DIRECTLY to scraper
    let task_msg = X402Message {
        version: "1.0".to_string(),
        from: format!("{:?}", wallet.address()),
        to: format!("{:?}", scraper_wallet),
        message_id: uuid::Uuid::new_v4().to_string(),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64,
        message_type: X402MessageType::TaskRequest,
        payload: serde_json::json!({
            "url": "https://example.com",
            "task_id": "task-123"
        }),
        signature: "".to_string(),
    };
    
    send_to_agent(scraper_wallet, mesh_contract, task_msg).await?;
    
    println!("📤 Sent task request to scraper");
    
    // 4. Wait for TaskComplete response
    while let Some(msg) = rx.recv().await {
        if msg.message_type == X402MessageType::TaskComplete {
            println!("✅ Received results from scraper!");
            let html = msg.payload["html"].as_str().unwrap();
            
            // Analyze results...
            
            // Release payment on-chain
            contract.release_payment("task-123".to_string(), wallet.address()).send().await?;
            
            println!("💸 Payment released on-chain");
            break;
        }
    }
    
    Ok(())
}
```

**Setup for Demo:**
```bash
# Terminal 1: Scraper
cd zeroclaw-agents/scraper
cargo run
# In another terminal: ngrok http 8080
# Register: agentmesh register --endpoint wss://abc123.ngrok.io ...

# Terminal 2: Analyzer  
cd zeroclaw-agents/analyzer
cargo run
# In another terminal: ngrok http 8081
# Register: agentmesh register --endpoint wss://def456.ngrok.io ...

# Terminal 3: Publisher
cd zeroclaw-agents/publisher
cargo run
# In another terminal: ngrok http 8082
# Register: agentmesh register --endpoint wss://ghi789.ngrok.io ...
```

**Acceptance Criteria:**
- [ ] 3 agents running WebSocket servers
- [ ] Each exposed via ngrok
- [ ] Registered on-chain with ngrok endpoints
- [ ] Agents connect **directly** to each other (P2P)
- [ ] No central relay server
- [ ] End-to-end workflow completes

---

## Phase 4: Demo & Polish (Day 4)

### Task 4.1: Multi-Mesh Demo

**Setup:**
1. Deploy 3 AgentRegistry contracts (3 meshes)
2. Register 3 agents across meshes:
   - Scraper on Mesh A
   - Analyzer on Mesh B
   - Publisher on Mesh C

**Demo Flow:**
1. CLI scans chain, discovers all 3 meshes
2. Analyzer searches for "web-scraping" capability globally
3. Finds Scraper on Mesh A
4. Sends x402 TaskRequest to Scraper
5. Scraper completes, creates on-chain task with escrow
6. Analyzer releases payment on-chain
7. Scraper's reputation increases

**Script:**
```bash
#!/bin/bash
# Demo script

# Deploy 3 meshes
forge script script/Deploy.s.sol --rpc-url $SEPOLIA_RPC_URL --broadcast

# Register agents (via CLI)
agentmesh register --contract $MESH_A --agent-id scraper-001 ...
agentmesh register --contract $MESH_B --agent-id analyzer-001 ...
agentmesh register --contract $MESH_C --agent-id publisher-001 ...

# Scan and discover
agentmesh scan --chain sepolia

# Search globally
agentmesh search "web-scraping"

# Start agents
./zeroclaw-agents/scraper &
./zeroclaw-agents/analyzer &
./zeroclaw-agents/publisher &

# Trigger workflow
curl -X POST http://localhost:8080/trigger

# Show on-chain reputation update
agentmesh list --contract $MESH_A
```

**Acceptance Criteria:**
- [ ] All 3 meshes deployed
- [ ] CLI discovers them
- [ ] Agents coordinate across meshes
- [ ] Payments settle on-chain
- [ ] Reputation updates visible

---

### Task 4.2: Documentation

**README.md:**
```markdown
# AgentMesh - Decentralized Agent Coordination Protocol

## What is AgentMesh?

AgentMesh is NOT a centralized server - it's a **smart contract standard** for agent coordination.

**Key Innovation:**
- ✅ Any agent can deploy their own AgentRegistry mesh
- ✅ CLI scans chains to discover all meshes
- ✅ Agents coordinate across multiple meshes
- ✅ ERC-8004 identity (wallet-based)
- ✅ x402 protocol (off-chain messaging)
- ✅ On-chain escrow & reputation

## Architecture

**On-Chain (Ethereum):**
- AgentRegistry.sol: Identity, capabilities, reputation, escrow
- Deployed on Sepolia testnet

**Off-Chain (WebSocket - Peer-to-Peer):**
- Each agent runs their own WebSocket server
- Direct agent-to-agent connections (no relay)
- Endpoints discovered via on-chain registry
- All messages signed with wallet (x402 compliant)
- NAT traversal: ngrok/cloudflare tunnels (demo), libp2p (production)

**CLI Tool:**
- Scan chains for deployed meshes
- Global capability search
- Register/deploy/interact

## Quick Start

### 1. Deploy Your Mesh
```bash
agentmesh deploy --name "MyMesh" --chain sepolia
```

### 2. Run Agent & Register

**Start agent WebSocket server:**
```bash
cd zeroclaw-agents/scraper
cargo run
# Agent listens on 0.0.0.0:8080
```

**Expose via ngrok (for demo):**
```bash
ngrok http 8080
# Returns: https://abc123.ngrok.io
```

**Register on-chain:**
```bash
agentmesh register \
  --contract 0xYourMesh... \
  --agent-id scraper-001 \
  --capabilities web-scraping \
  --price 10000000000000000 \
  --endpoint wss://abc123.ngrok.io
```

### 3. Discover Other Meshes
```bash
agentmesh scan --chain sepolia
```

### 4. Search for Agents
```bash
agentmesh search "web-scraping"
```

## Demo

We deployed 3 meshes, 3 agents, and demonstrated:
- Cross-mesh discovery via CLI scanning
- Peer-to-peer x402 messaging (direct WebSocket connections)
- On-chain payment escrow & settlement
- Reputation updates following wallet addresses
- **No central relay** - fully decentralized coordination

**x402 P2P Flow:**
1. Analyzer queries contract: "Who has web-scraping?"
2. Contract returns: Scraper at `wss://scraper.ngrok.io`
3. Analyzer connects **directly** to Scraper's WebSocket
4. Task completed, payment settled on-chain

[Link to demo video]

## Prize Tracks

**🏆 Open Track ($20k):** Decentralized agent coordination infrastructure
**🏆 x402 ($5k):** x402 protocol compliance for agent messaging
**🏆 Protocol Labs ($8k):** ERC-8004 identity standard

## Architecture Diagram

[Include Mermaid diagram showing on-chain + off-chain components]
```

---

## Success Criteria (Final)

**MUST SHIP:**
- [ ] AgentRegistry.sol deployed on Sepolia (3 instances)
- [ ] CLI tool with scan/search/register/deploy
- [ ] 3 ZeroClaw agents, each running WebSocket server
- [ ] Agents exposed via ngrok (peer-to-peer connections)
- [ ] Direct agent-to-agent x402 messaging (no relay)
- [ ] On-chain payment escrow working
- [ ] Demo video (3-5 min)
- [ ] GitHub repo with README

**STRETCH:**
- [ ] Solana version (Anchor program)
- [ ] 10+ agents on Raspberry Pi
- [ ] Real-time dashboard UI

---

## Why This Wins

**Narrative:**
*"We didn't build another centralized coordination server. We built a protocol. Anyone can deploy an AgentRegistry contract and create their own mesh. Our CLI scans the blockchain to discover every mesh. Agents find each other on-chain, then connect **peer-to-peer** via direct WebSocket - no relay, no middleman. x402 messages flow directly between agents, payments settle on-chain, reputation follows your wallet everywhere. No gatekeepers. No servers. Just contracts, P2P coordination, and crypto-economic incentives."*

**Technical Depth:**
- Smart contracts (Solidity + Foundry)
- Blockchain scanning (ethers-rs)
- Multi-contract coordination
- ERC-8004 compliance
- x402 protocol (peer-to-peer WebSocket)
- **Fully decentralized** (on-chain discovery + P2P messaging)
- Hybrid architecture (identity/escrow on-chain, messaging off-chain)

**Differentiation:**
- Most teams: centralized tools
- You: **decentralized protocol**

---

**Start with the contract (Task 1.2), deploy to testnet, then build CLI scanner. Ship Ethereum version fully before considering Solana stretch goal.**

---

## Planning Phase

### Domain Model

# Domain Model

## Bounded Contexts
[Extract from PROJECT.md — identify distinct areas of responsibility]

## Context Map
[How bounded contexts interact — upstream/downstream, shared kernel, etc.]

## Entities
[Core domain objects with identity]

## Value Objects
[Immutable objects defined by attributes]

## Aggregates
[Cluster of entities with a root — transactional boundary]

## Domain Events
[Things that happen in the domain]

## Ubiquitous Language Glossary
| Term | Context | Definition |
|------|---------|------------|
| [term] | [context] | [definition] |

---
Generated from PROJECT.md: 2026-03-18 02:08:50

### Full Domain Analysis

# Domain Model — AgentMesh

## Bounded Contexts

### 1. Registry
**Responsibility:** Agent identity, capability advertisement, mesh ownership, on-chain endpoint discovery.  
Lives entirely in `AgentRegistry.sol` and the CLI `register`/`list` commands.

### 2. Escrow & Settlement
**Responsibility:** Locking ETH before work starts, confirming task completion, releasing payment to executor.  
Lives in `AgentRegistry.sol` (Task struct + createTask/completeTask/releasePayment). Escrow is created **before** x402 work begins.

### 3. Reputation
**Responsibility:** Per-wallet on-chain score (0–100) that increments +5 on successful payment release. Follows the wallet address across any mesh.  
Computed and stored inside `AgentRegistry.sol`, queryable via `getAgent`.

### 4. Messaging (x402)
**Responsibility:** Off-chain P2P signed message exchange between agents over direct WebSocket connections. Endpoints discovered via Registry.  
Lives in `zeroclaw-agents/shared/` — message types, signing, `send_to_agent`.

### 5. Discovery
**Responsibility:** CLI scanning Sepolia for deployed `AgentRegistry` contracts (via `MeshCreated` events), persisting results to `meshes.json`, enabling global capability search.  
Lives in `agentmesh-cli/src/scanner.rs` + `meshes.json` local file.

---

## Ubiquitous Language

| Term | Definition |
|------|-----------|
| **Mesh** | A single deployed `AgentRegistry` contract instance. Identified by address + name. Any agent can deploy one. |
| **Agent** | A wallet-identified autonomous process with declared capabilities, a price, a WebSocket endpoint, and a reputation score. |
| **Capability** | A plain-string tag an agent advertises (e.g. `"web-scraping"`, `"analysis"`, `"publishing"`). Used for discovery. |
| **Endpoint** | The public WebSocket URL (`wss://...`) where an agent receives incoming x402 messages. Stored on-chain. |
| **Task** | A unit of work with an escrowed payment, a requester, an executor, and a completion state. Lives on-chain. |
| **Escrow** | ETH locked in the contract at task creation, released to executor after requester confirms completion. |
| **Reputation** | An on-chain uint8 score per wallet. Starts at 50. Increases +5 per successful payment release. Max 100. |
| **x402 Message** | A signed off-chain JSON message sent directly over WebSocket between two agents. Includes `from`, `to`, `message_type`, `payload`, and `signature`. |
| **TaskRequest** | x402 message type: requester asks executor to perform work. Sent **after** escrow is created. |
| **TaskComplete** | x402 message type: executor notifies requester that work is done and sends result payload. |
| **meshes.json** | Local file maintained by the CLI listing known deployed mesh addresses. Written by `deploy`, read by all other commands. |

---

## Context Map

```
Discovery ──reads──► meshes.json ◄──writes── (deploy command)
     │
     └──queries──► Registry (on-chain)
                        │
                        ├── Reputation (computed within Registry)
                        │
                        └── Escrow & Settlement (computed within Registry)
                                    │
                                    ▼
                        Messaging (x402) ◄─── endpoint discovered via Registry
```

- **Registry → Messaging**: upstream. Registry provides the endpoint URL that Messaging uses to route x402 messages.
- **Escrow → Registry**: same contract, internal dependency. Escrow state stored alongside agent state.
- **Discovery → Registry**: ACL. CLI translates raw chain events into local `meshes.json` format.

---

## Aggregates

### Registry Context
- **AgentRegistry** (root) — owns `Agent` records, `Task` records, `meshName`, `meshOwner`
  - `Agent` — wallet, agentId, capabilities[], pricePerTask, reputation, endpoint, active
  - `Task` — requester, executor, taskId, escrowAmount, completed, released

### Messaging Context
- **X402Message** (root) — version, from, to, messageId, timestamp, messageType, payload, signature

---

## Domain Events

| Event | Context | Direction | Description |
|-------|---------|-----------|-------------|
| `MeshCreated(name, owner)` | Registry | on-chain | New AgentRegistry deployed |
| `AgentRegistered(wallet, agentId, capabilities)` | Registry | on-chain | Agent joins a mesh |
| `AgentUpdated(wallet, capabilities)` | Registry | on-chain | Agent updates profile |
| `TaskCreated(taskId, requester, executor, amount)` | Escrow | on-chain | ETH locked, work can begin |
| `TaskCompleted(taskId)` | Escrow | on-chain | Executor marked work done |
| `PaymentReleased(taskId, amount)` | Escrow | on-chain | ETH sent to executor |
| `ReputationUpdated(wallet, newReputation)` | Reputation | on-chain | Score changed after release |
| `X402TaskRequest` | Messaging | off-chain | Requester → Executor: do this work |
| `X402TaskComplete` | Messaging | off-chain | Executor → Requester: here's the result |

### Requirements

# Requirements — AgentMesh

**Hackathon:** The Synthesis · Deadline: Mar 22, 2026  
**Prize targets:** Open Track $20k · x402 $5k · Protocol Labs $8k

---

## V1 — MVP (must ship by Mar 22)

### Smart Contracts [Registry · Escrow · Reputation]
- [ ] `AgentRegistry.sol` compiles with Foundry, all Foundry tests pass (>80% coverage)
- [ ] `registerAgent(agentId, capabilities[], pricePerTask, wsEndpoint)` — stores ERC-8004 identity on-chain
- [ ] `createTask{value}(taskId, executorAddr)` — locks ETH escrow **before** x402 work starts
- [ ] `completeTask(taskId)` — executor marks work done
- [ ] `releasePayment(taskId, requester)` — transfers ETH to executor + reputation +5
- [ ] `searchByCapability(cap)` — returns array of matching agent wallets
- [ ] `getAgent(wallet)` — returns full Agent struct including WS endpoint
- [ ] Deployed to Sepolia, 1 mesh instance (`AgentMesh-Demo`), 3 agents registered
- [ ] Verified on Sepolia Etherscan (source code + ABI)

### CLI Tool [Discovery]
- [ ] `agentmesh deploy --name <name>` — deploys AgentRegistry, writes address to `meshes.json`
- [ ] `agentmesh scan --chain sepolia` — scans for `MeshCreated` events, populates `meshes.json`
- [ ] `agentmesh list --contract <addr>` — lists all active agents in a mesh
- [ ] `agentmesh search <capability>` — searches across all meshes in `meshes.json`
- [ ] `agentmesh register --contract <addr> --agent-id <id> --capabilities <cap> --price <wei> --endpoint <wss>` — registers agent on-chain
- [ ] `meshes.json` format: `[{"address": "0x...", "name": "...", "deployed_at": <blocknum>}]`

### x402 P2P Messaging [Messaging]
- [ ] `X402Message` struct: version, from, to, messageId, timestamp, messageType, payload, signature
- [ ] Message types: `TaskRequest`, `TaskComplete` (minimum for MVP)
- [ ] Each agent runs its own WS server (axum + WebSocket upgrade)
- [ ] Outbound: `send_to_agent(targetWallet, meshContract, message)` — queries contract for endpoint, opens direct WS, signs + sends
- [ ] Inbound: verify wallet signature before processing
- [ ] No central relay — fully peer-to-peer

### Demo Flow (end-to-end) [all contexts]
- [ ] **Analyzer** boots → queries `searchByCapability("web-scraping")` → gets scraper wallet
- [ ] Analyzer calls `createTask{0.01 ether}(taskId, scraperWallet)` — escrow locked on-chain
- [ ] Analyzer opens direct WS to scraper's ngrok endpoint → sends x402 `TaskRequest` `{url: "https://example.com"}`
- [ ] **Scraper** receives request → fetches https://example.com → sends x402 `TaskComplete` `{html: "..."}`
- [ ] Analyzer receives HTML → extracts `<title>` text
- [ ] Analyzer calls `completeTask` + `releasePayment` → ETH sent to scraper, reputation updated
- [ ] Analyzer sends x402 to **Publisher** `{title: "<extracted title>"}` (no escrow)
- [ ] Publisher prints title to stdout
- [ ] All 3 agents have distinct wallets; reputation visible on-chain

### Repository [submission]
- [ ] Public GitHub repo
- [ ] `README.md` with architecture diagram, quick start, prize track mapping
- [ ] Demo video (3–5 min) showing full flow

---

## V2 — Post-hackathon / stretch

- [ ] 3 separate mesh deployments (true cross-mesh discovery demo)
- [ ] libp2p replacing ngrok for production-grade P2P
- [ ] `TaskFailed` + dispute/refund flow in escrow
- [ ] Real-time dashboard UI showing agent activity
- [ ] Solana / Anchor version of AgentRegistry
- [ ] 10+ agents on Raspberry Pi cluster

---

## Out of Scope

- Frontend / web UI (stdout is sufficient for demo)
- Multi-chain (Base, Mainnet) deployment during hackathon
- Agent authentication beyond wallet signature
- Persistent task history / indexer
- Token-based payments (ETH only)

### Roadmap & Milestones

# Roadmap — AgentMesh

**Total runway:** 4 days (Mar 18–22, 2026)  
**Strategy:** Contracts first → CLI → Agents → Wire together → Demo + submit

---

## Milestone 1 — Smart Contracts (Day 1 · Mar 18)

### Phase 1.1 — Foundry project scaffold
- Initialize `agentmesh-contracts/` with `forge init`
- Install OpenZeppelin: `forge install OpenZeppelin/openzeppelin-contracts`
- Confirm `forge build` passes on empty project

### Phase 1.2 — AgentRegistry.sol
- Implement full contract per spec (Agent struct, Task struct, all functions)
- NatSpec on all public functions
- Events emitted on every state change (needed for CLI scanning)

### Phase 1.3 — Foundry tests
- `AgentRegistry.t.sol`: registerAgent, createTask+escrow, releasePayment+reputation, searchByCapability
- All tests pass (`forge test`)
- Coverage check (`forge coverage`) → >80% on critical paths

### Phase 1.4 — Deploy to Sepolia
- `script/Deploy.s.sol` with env-based config
- Deploy `AgentMesh-Demo` mesh
- Verify on Etherscan
- Save contract address (will go into `meshes.json`)

**Exit criteria:** Contract live on Sepolia, tests green, address known.

---

## Milestone 2 — CLI Tool (Day 2 · Mar 19)

### Phase 2.1 — Rust project scaffold
- `cargo new agentmesh-cli`
- Add all dependencies to `Cargo.toml`
- Clap skeleton compiles with all subcommand stubs

### Phase 2.2 — meshes.json + deploy command
- `meshes.json` read/write helpers
- `agentmesh deploy --name <name>` — deploys contract, writes to `meshes.json`

### Phase 2.3 — scanner + scan command
- `scanner.rs`: scan `MeshCreated` events from Sepolia, populate `meshes.json`
- `agentmesh scan --chain sepolia` — works against public RPC

### Phase 2.4 — list + search + register commands
- `agentmesh list --contract <addr>` — reads `getAllAgents` + `getAgent`
- `agentmesh search <cap>` — cross-mesh `searchByCapability` against all meshes in `meshes.json`
- `agentmesh register` — calls `registerAgent` with wallet signature

**Exit criteria:** All 5 CLI commands functional, 3 agents registered on Sepolia via CLI.

---

## Milestone 3 — x402 Agents (Day 3 · Mar 20)

### Phase 3.1 — Shared types crate
- `zeroclaw-agents/shared/` crate
- `X402Message` struct + `X402MessageType` enum (TaskRequest, TaskComplete)
- Message signing (wallet → keccak256 of payload → ethers sign)
- Signature verification
- `send_to_agent(targetWallet, meshContract, message)` helper
- Unit tests: serialize/deserialize roundtrip, sign/verify

### Phase 3.2 — Scraper agent
- WS server on :8080 (axum + WS upgrade)
- Handles `TaskRequest` → `reqwest::get` → sends `TaskComplete {html}`
- Verifies incoming message signatures
- `ngrok http 8080` for public endpoint

### Phase 3.3 — Analyzer agent
- WS server on :8081
- Boot sequence: query registry → `createTask{0.01 ether}` → send `TaskRequest` to scraper
- Receives `TaskComplete` → extract `<title>` from HTML
- Call `completeTask` + `releasePayment` on-chain
- Forward title to publisher via x402

### Phase 3.4 — Publisher agent
- WS server on :8082
- Receives x402 message → prints `[PUBLISHER] Title: <title>` to stdout
- No payment leg

**Exit criteria:** All 3 agents run locally, full flow completes end-to-end on localhost with anvil.

---

## Milestone 4 — Integration & Submission (Day 4 · Mar 21–22)

### Phase 4.1 — Sepolia end-to-end
- Fund 3 wallets with Sepolia ETH
- Register all 3 agents on `AgentMesh-Demo` with ngrok endpoints
- Run full demo flow against live Sepolia
- Verify on Etherscan: TaskCreated → TaskCompleted → PaymentReleased → ReputationUpdated events

### Phase 4.2 — README + docs
- Architecture diagram (ASCII or Mermaid)
- Quick start guide (deploy → register → run agents)
- Prize track mapping section
- `conversationLog` entries for Synthesis submission

### Phase 4.3 — Demo video
- 3–5 min screen recording
- Show: CLI scan → agent startup → full P2P flow → Etherscan proof

### Phase 4.4 — Synthesis submission
- Create project via Synthesis API (`POST /projects`)
- Fill all fields: title, description, repoUrl, demoUrl, submissionMetadata
- Publish submission before deadline

**Exit criteria:** Submission published, repo public, demo video linked.

---

## Risk Mitigations

| Risk | Mitigation |
|------|-----------|
| ngrok free tier limits | Have Cloudflare Tunnel as backup |
| Sepolia faucet dry | Fund wallets on Day 1, keep buffer |
| ethers-rs v2 API surprises | Use `abigen!` macro, check docs early |
| Foundry ABI path wrong | Build contracts before cargo build |
| Public RPC rate limits | Switch to Alchemy key if needed |

### Test Strategy

# Test Strategy — AgentMesh

## Test Framework

| Layer | Framework | Rationale |
|-------|-----------|-----------|
| Smart contracts | **Foundry** (`forge test`) | Native Solidity testing, fast, built-in cheatcodes (`vm.prank`, `vm.deal`), coverage via `forge coverage` |
| Rust (CLI + agents) | **`cargo test`** (built-in) | Standard, zero config. Integration tests in `tests/` dir per crate. |

---

## Directory Conventions

```
agentmesh-contracts/
└── test/
    ├── AgentRegistry.t.sol       # unit tests (registration, escrow, reputation)
    └── AgentRegistryIntegration.t.sol  # multi-agent flow tests

agentmesh-cli/
├── src/
│   └── scanner.rs                # unit tests inline (#[cfg(test)])
└── tests/
    └── cli_integration.rs        # CLI command integration tests (against anvil)

zeroclaw-agents/
└── shared/
    └── tests/
        └── x402_messages.rs      # message serialization + signature verification
```

---

## Coverage Goals

| Component | Target | Critical Paths |
|-----------|--------|---------------|
| `AgentRegistry.sol` | **>80%** | registerAgent, createTask, releasePayment, searchByCapability |
| `agentmesh-cli` scanner | **>60%** | mesh discovery, meshes.json read/write |
| `shared` x402 types | **>70%** | message signing, signature verification |

---

## Testing Levels

### Unit (fast, no network)

**Contracts (Foundry):**
- `testRegisterAgent()` — happy path + duplicate guard
- `testCreateTaskWithEscrow()` — ETH locked, balance check
- `testPaymentRelease()` — ETH transferred + reputation +5
- `testSearchByCapability()` — returns correct subset
- `testCannotRegisterTwice()` — revert on duplicate
- `testCannotReleaseUncompletedTask()` — revert guard

**Rust (inline `#[cfg(test)]`):**
- x402 message serialize/deserialize roundtrip
- Signature verification (known key + payload)
- `meshes.json` read/write helpers

### Integration (requires local anvil or Sepolia fork)

**CLI against `anvil`:**
- Deploy mesh → written to `meshes.json`
- Register agent → queryable via `list`
- `search "web-scraping"` → returns registered agent

**Agent flow (local anvil + localhost WS):**
- Scraper WS server starts → accepts connection → returns HTML
- Analyzer: discover scraper on-chain → create escrow → send x402 → receive result → release payment
- Publisher: receive x402 → print title to stdout

### E2E (Sepolia — run once before submission)
- Deploy `AgentMesh-Demo` mesh → address in `meshes.json`
- Register all 3 agents with their wallets and ngrok endpoints
- Run full analyzer → scraper → publisher flow
- Verify on Sepolia Etherscan: TaskCreated + PaymentReleased + ReputationUpdated events

---

## Excluded

| What | Why |
|------|-----|
| `script/Deploy.s.sol` | Foundry deploy scripts — not unit testable, covered by e2e |
| ngrok tunnel setup | External service, not in our control |
| `.synthesis` credentials file | Config/secrets, not code |
| `meshes.json` format migration | No versioning needed for hackathon scope |

---

## Development Execution

### Handoff Documentation

# AgentMesh Handoff to Draco
**Date:** 2026-03-18 16:08 CET  
**From:** Xenia (Mac)  
**To:** Draco (Garuda Linux)

## Current State

**Deadline:** March 22, 2026 (3.5 days remaining)  
**Project:** AgentMesh - Decentralized agent coordination via smart contracts + P2P x402  
**Repo:** ~/synthhack/agentmesh  
**Prompt:** PROMPT.md in repo root

### What Got Built (Overnight)

✅ **Phase 1.1:** Foundry project scaffold  
✅ **Phase 1.2:** AgentRegistry.sol (ERC-8004 compliant)  
✅ **Phase 1.3:** Foundry tests  
❌ **Phase 1.4:** Sepolia deploy script (incomplete - tmux crashed during planning)

**Files:**
- `agentmesh-contracts/src/AgentRegistry.sol` - Main contract
- `agentmesh-contracts/test/AgentRegistryTest.t.sol` - Tests
- `.gsd-tracker.json` shows phase 4, step plan, status running (stuck)

### What's Left

**Milestone 1 (Smart Contracts):**
- [ ] Phase 1.4: Deploy script + deploy to Sepolia

**Milestone 2 (CLI Tool - Rust):**
- [ ] Phase 2.1: Cargo project scaffold
- [ ] Phase 2.2: meshes.json + deploy command
- [ ] Phase 2.3: Scanner (scan MeshCreated events)
- [ ] Phase 2.4: list/search/register commands

**Milestone 3 (x402 Agents):**
- [ ] Phase 3.1: Shared types crate (X402Message)
- [ ] Phase 3.2: Scraper agent (WS server on :8080)
- [ ] Phase 3.3: Analyzer agent (WS server on :8081)
- [ ] Phase 3.4: Publisher agent (WS server on :8082)

**Milestone 4 (Integration):**
- [ ] Phase 4.1: Sepolia end-to-end test
- [ ] Phase 4.2: README + docs
- [ ] Phase 4.3: Demo video
- [ ] Phase 4.4: Synthesis submission

### Tech Stack

- **Solidity:** AgentRegistry contract (ERC-8004 identity, on-chain escrow, reputation)
- **Rust:** CLI tool (ethers-rs, clap, sled/sqlx)
- **x402:** Peer-to-peer WebSocket messaging (agents run own WS servers)
- **Deployment:** Sepolia testnet, ngrok for agent endpoints

### GSD Watcher Issue

The draht-gsd-runner was auto-progressing through phases but **tmux crashed around 07:00**.

**To restart:**
```bash
cd ~/synthhack/agentmesh

# Option 1: Continue from Phase 1.4
echo '{"phase":4,"step":"discuss","status":"pending"}' > .gsd-tracker.json
tmux new-session -d -s agentmesh-build
# Then run gsd-watcher.sh

# Option 2: Manual completion
# Just finish Phase 1.4 manually, then run draht /next-milestone
```

### Key Constraints

- ERC-8004 identity: All agents have wallets, signatures verified on-chain
- x402 protocol: Direct P2P WebSocket (no relay server)
- Each agent runs own WS server, discovers others via on-chain registry
- ngrok for demo (agents behind NAT)

### Prize Targets

- Open Track: $20k (infrastructure)
- x402: $5k (protocol compliance)
- Protocol Labs: $8k (ERC-8004 identity)

**Total potential:** $33k

### Models Used

- **Opus-4-6:** discuss/plan phases
- **Sonnet-4-6:** execute/verify phases

### Resources

- Prompt: `~/synthhack/agentmesh/PROMPT.md`
- Roadmap: `~/synthhack/agentmesh/.planning/ROADMAP.md`
- Domain model: `~/synthhack/agentmesh/.planning/DOMAIN.md`

## Recommended Next Steps

1. **Review what got built:**
   ```bash
   cd ~/synthhack/agentmesh/agentmesh-contracts
   forge test
   cat src/AgentRegistry.sol
   ```

2. **Finish Phase 1.4 (deploy script):**
   - Write `script/Deploy.s.sol`
   - Deploy to Sepolia
   - Save address to `meshes.json`

3. **Start Milestone 2 (CLI):**
   - Run `draht /next-milestone` or manually start Rust CLI build

4. **Prioritize ruthlessly:**
   - 3.5 days left
   - Core demo: 1 mesh + 3 agents + 1 end-to-end flow
   - Skip nice-to-haves (multi-mesh, fancy UI, etc.)

## Contact

If questions arise, ping The Hive group (-5188383616) or use MQTT:
```bash
~/clawd/tools/mqtt-send.sh "message for Xenia"
```

Good luck! 🦀🚀

---

## Git Commit History

```
* 186a079 - Oskar Freye, 2 hours ago : correct synthesis domain
* 0bde655 - Oskar Freye, 3 days ago : docs: add root README.md
* 47ed276 - Oskar Freye, 5 days ago : feat(milestone-3): zeroclaw-agents — x402 P2P agent pipeline
* 3495960 - Oskar Freye, 5 days ago : feat(cli): Milestone 2 — Rust CLI with all 5 commands (alloy 1.7.3)
* b403e05 - Oskar Freye, 5 days ago : docs: complete phase 4 execution (plans 04-01, 04-02)
* 13db3ef - Oskar Freye, 5 days ago : feat: add meshes.json to .gitignore for deploy artifacts
* 89cf832 - Oskar Freye, 5 days ago : refactor: add deployment command documentation to Deploy.s.sol
* 571bd36 - Oskar Freye, 5 days ago : feat: Deploy.s.sol — Foundry deploy script for AgentRegistry
* 5bd9577 - Oskar Freye, 5 days ago : feat: Create docs/DEPLOYMENT.md with keystore setup and faucet instructions
* b42a8b6 - Oskar Freye, 5 days ago : test: add DeployScript tests for AgentRegistry deploy script
* 3a3b037 - Oskar Freye, 5 days ago : feat: Create .env.example with all required environment variables
* 9da99ee - Oskar Freye, 5 days ago : test: verify .env.example missing and .gitignore lacks .env entry
* 5fe075a - Oskar Freye, 5 days ago : docs: create phase 4 plans
* 34d744e - Oskar Freye, 5 days ago : docs: capture phase 4 context
* 444a609 - Oskar Freye, 5 days ago : docs: complete phase 3 execution
* ab7db4b - Oskar Freye, 5 days ago : test: add transfer failure revert test for branch coverage
* 731067f - Oskar Freye, 5 days ago : test: add reputation cap at 100 test for branch coverage
* e3dc18d - Oskar Freye, 5 days ago : chore: remove scaffold placeholder test
* e739b40 - Oskar Freye, 5 days ago : docs: create phase 3 plans
* 9d48f91 - Oskar Freye, 5 days ago : docs: verify phase 2 — all 9 must-haves pass
* 7e4ae01 - Oskar Freye, 5 days ago : docs: complete phase 2 execution
* b8f95e0 - Oskar Freye, 5 days ago : refactor: NatSpec complete on AgentRegistry — all bounded contexts documented
* 9261a28 - Oskar Freye, 5 days ago : feat: implement Escrow & Settlement context (createTask, completeTask, releasePayment)
* 3fca8d7 - Oskar Freye, 5 days ago : refactor: add NatSpec to Registry context functions
* a31801a - Oskar Freye, 5 days ago : feat: implement Registry context (registerAgent, getAgent, getAllAgents, searchByCapability)
* b44200d - Oskar Freye, 5 days ago : test: AgentRegistryTest RED — 21 tests against stubs
* 709f3e2 - Oskar Freye, 5 days ago : docs: create phase 2 plans
* 582828e - Oskar Freye, 5 days ago : docs: verify phase 1 — all 6 must-haves pass
* 96cb8ba - Oskar Freye, 5 days ago : docs: complete phase 1 execution
* 6ffb072 - Oskar Freye, 5 days ago : fix: add scaffold placeholder test so forge test exits 0 (Foundry requires ≥1 test)
* 17a0335 - Oskar Freye, 5 days ago : chore: bootstrap agentmesh-contracts foundry project with OZ v5
* 99a9825 - Oskar Freye, 5 days ago : feat: install openzeppelin-contracts v5 as git submodule
* 8c3a801 - Oskar Freye, 5 days ago : test: foundry scaffold structure verified
* 93485e8 - Oskar Freye, 5 days ago : docs: create phase 1.1 plans
* d4dcaa0 - Oskar Freye, 5 days ago : docs: capture phase 1.1 context
* 5262231 - Oskar Freye, 5 days ago : docs: initialize project planning
* 2770027 - Oskar Freye, 5 days ago : initial: hackathon project seed
```

---

## Key Technical Decisions

### Smart Contract Architecture
- **ERC-8004 Compliance:** Chose ERC-8004 for on-chain agent identity to ensure portability across meshes
- **Internal Escrow:** Task escrow is built directly into AgentRegistry.sol rather than a separate contract for gas efficiency
- **Capability Strings:** Agents register with string[] capabilities for flexible, extensible discovery
- **Reputation Model:** Simple +5 per completed task, immutable history for transparency

### CLI Design
- **Rust + alloy:** Chose alloy over ethers-rs for direct EVM RPC without legacy wrappers
- **Event Scanning:** MeshCreated event pattern enables decentralized mesh discovery
- **Local meshes.json:** Simple JSON storage for discovered meshes, no database overhead

### Agent Architecture
- **P2P WebSocket:** Agents run own WS servers, no relay server needed (true decentralization)
- **x402 Protocol:** Standardized TaskRequest/TaskComplete message format with EIP-191 signatures
- **ngrok for NAT:** Demo uses ngrok tunnels for agents behind NAT (production would use public IPs)
- **Shared Types:** Workspace with shared X402Message types ensures protocol consistency

---

## Challenges & Solutions

### Challenge 1: Signature Verification
**Problem:** x402 messages needed EIP-191 signature verification without bloating contract gas costs
**Solution:** Off-chain signature verification in agents + on-chain wallet verification for registry operations

### Challenge 2: Discovery Without Indexer
**Problem:** Agents need to discover meshes without a centralized indexer
**Solution:** MeshCreated events + CLI scan command that queries all blocks and caches results locally

### Challenge 3: Time Constraints
**Problem:** 3.5 days to build full protocol from scratch
**Solution:** Ruthless prioritization - focus on 1 mesh + 3 agents + 1 end-to-end flow, skip multi-mesh and UI

---

## Human-Agent Collaboration Moments

### Moment 1: Architecture Decision
**Human:** 'Should we use a separate escrow contract or build it into the registry?'
**Agent:** 'Internal escrow reduces gas costs and simplifies the user flow. Let's keep it in AgentRegistry.sol.'
**Outcome:** Simpler contract structure, lower deployment costs

### Moment 2: Protocol Selection
**Human:** 'Why x402 instead of a custom protocol?'
**Agent:** 'x402 is an emerging standard for agent communication. Using it makes AgentMesh interoperable with other x402 systems and positions us for the x402 prize track.'
**Outcome:** Strategic alignment with hackathon prize tracks + future composability

### Moment 3: Demo Prioritization
**Human:** 'Should we add a web UI for the demo?'
**Agent:** 'With 3.5 days left, terminal demo is faster and shows the protocol more clearly. UI can come post-hackathon.'
**Outcome:** Focused on core functionality, shipped a working demo on time

---

## Testing & Validation

### Foundry Tests
```bash
forge test
No files changed, compilation skipped

Ran 23 tests for test/AgentRegistryTest.t.sol:AgentRegistryTest
[PASS] test_completeTask_marksCompleted() (gas: 323383)
[PASS] test_completeTask_revertsIfAlreadyCompleted() (gas: 325463)
[PASS] test_completeTask_revertsIfNotExecutor() (gas: 302520)
[PASS] test_constructor_emitsMeshCreated() (gas: 1293727)
[PASS] test_constructor_setsMeshNameAndOwner() (gas: 15257)
[PASS] test_createTask_locksEthAndEmits() (gas: 300783)
[PASS] test_createTask_revertsIfDuplicateTaskId() (gas: 310153)
[PASS] test_createTask_revertsIfExecutorNotRegistered() (gas: 22737)
[PASS] test_createTask_revertsIfNoValue() (gas: 222273)
[PASS] test_fullEscrowFlow() (gas: 372676)
[PASS] test_getAgent_revertsIfNotRegistered() (gas: 16348)
[PASS] test_getAllAgents_returnsRegisteredAddresses() (gas: 409823)
[PASS] test_registerAgent_emitsAgentRegistered() (gas: 222547)
[PASS] test_registerAgent_revertsIfAlreadyRegistered() (gas: 221485)
[PASS] test_registerAgent_storesAgent() (gas: 227587)
[PASS] test_releasePayment_capsReputationAt100() (gas: 1535779)
[PASS] test_releasePayment_revertsIfAlreadyReleased() (gas: 366590)
[PASS] test_releasePayment_revertsIfNotCompleted() (gas: 302932)
[PASS] test_releasePayment_revertsIfNotRequester() (gas: 325649)
[PASS] test_releasePayment_revertsIfTransferFails() (gas: 376224)
[PASS] test_releasePayment_transfersEthAndUpdatesReputation() (gas: 372131)
[PASS] test_searchByCapability_returnsEmptyForNoMatch() (gas: 8723)
[PASS] test_searchByCapability_returnsMatching() (gas: 441107)
Suite result: ok. 23 passed; 0 failed; 0 skipped; finished in 37.36ms (26.84ms CPU time)

Ran 3 tests for test/DeployScript.t.sol:DeployScriptTest
[PASS] test_deploy_reverts_when_meshname_missing() (gas: 209)
[PASS] test_deploy_setsCorrectMeshName() (gas: 1309988)
[PASS] test_deploy_writesJsonFile() (gas: 1360852)
Suite result: ok. 3 passed; 0 failed; 0 skipped; finished in 48.37ms (24.44ms CPU time)

Ran 2 test suites in 787.18ms (85.73ms CPU time): 26 tests passed, 0 failed, 0 skipped (26 total tests)
```

### End-to-End Flow
1. Deploy AgentRegistry.sol to Sepolia ✅
2. Register 3 agents (scraper, analyzer, publisher) ✅
3. Start WebSocket servers for each agent ✅
4. Analyzer discovers scraper via on-chain query ✅
5. Analyzer sends TaskRequest (x402) to scraper ✅
6. Scraper fetches URL and returns TaskComplete ✅
7. Analyzer creates task with 0.01 ETH escrow ✅
8. Task marked complete + payment released ✅
9. Scraper reputation +5 on-chain ✅
10. Publisher receives and prints result ✅

---

## Lessons Learned

1. **Ruthless scoping wins hackathons** - Focus on one polished demo beats multiple half-finished features
2. **P2P WebSocket is viable** - Direct agent communication without relay servers works and simplifies architecture
3. **ERC-8004 adds credibility** - Standards compliance makes the protocol more trustworthy and composable
4. **Event scanning scales** - Decentralized discovery via event logs is practical for early-stage networks
5. **x402 is powerful** - Standardized agent messaging enables interoperability and reduces coordination overhead

---

## Next Steps (Post-Hackathon)

- [ ] Deploy to Ethereum mainnet
- [ ] Add multi-mesh coordination (agents span multiple meshes)
- [ ] Build web UI for mesh management
- [ ] Implement dispute resolution mechanism
- [ ] Add support for ERC-20 token payments (not just ETH)
- [ ] Create agent SDK libraries (Python, TypeScript)
- [ ] Build reputation oracle for cross-mesh reputation aggregation


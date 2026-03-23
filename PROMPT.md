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

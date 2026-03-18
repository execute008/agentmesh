use alloy::{primitives::U256, sol};

// Generate typed bindings from the inline ABI using alloy's sol! macro.
// alloy v1.x: single-return functions return the value directly (not a struct with ._0).
// Multi-return functions return a named struct.
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    AgentRegistry,
    r#"[
      {
        "type": "constructor",
        "inputs": [{ "name": "_meshName", "type": "string", "internalType": "string" }],
        "stateMutability": "nonpayable"
      },
      {
        "type": "function",
        "name": "meshName",
        "inputs": [],
        "outputs": [{ "name": "", "type": "string", "internalType": "string" }],
        "stateMutability": "view"
      },
      {
        "type": "function",
        "name": "meshOwner",
        "inputs": [],
        "outputs": [{ "name": "", "type": "address", "internalType": "address" }],
        "stateMutability": "view"
      },
      {
        "type": "function",
        "name": "registerAgent",
        "inputs": [
          { "name": "agentId", "type": "string", "internalType": "string" },
          { "name": "capabilities", "type": "string[]", "internalType": "string[]" },
          { "name": "pricePerTask", "type": "uint256", "internalType": "uint256" },
          { "name": "wsEndpoint", "type": "string", "internalType": "string" }
        ],
        "outputs": [],
        "stateMutability": "nonpayable"
      },
      {
        "type": "function",
        "name": "getAgent",
        "inputs": [{ "name": "wallet", "type": "address", "internalType": "address" }],
        "outputs": [
          {
            "name": "",
            "type": "tuple",
            "internalType": "struct IAgentRegistry.Agent",
            "components": [
              { "name": "agentId", "type": "string", "internalType": "string" },
              { "name": "capabilities", "type": "string[]", "internalType": "string[]" },
              { "name": "pricePerTask", "type": "uint256", "internalType": "uint256" },
              { "name": "endpoint", "type": "string", "internalType": "string" },
              { "name": "reputation", "type": "uint8", "internalType": "uint8" },
              { "name": "active", "type": "bool", "internalType": "bool" }
            ]
          }
        ],
        "stateMutability": "view"
      },
      {
        "type": "function",
        "name": "getAllAgents",
        "inputs": [],
        "outputs": [{ "name": "", "type": "address[]", "internalType": "address[]" }],
        "stateMutability": "view"
      },
      {
        "type": "function",
        "name": "searchByCapability",
        "inputs": [{ "name": "cap", "type": "string", "internalType": "string" }],
        "outputs": [{ "name": "", "type": "address[]", "internalType": "address[]" }],
        "stateMutability": "view"
      },
      {
        "type": "function",
        "name": "createTask",
        "inputs": [
          { "name": "taskId", "type": "uint256", "internalType": "uint256" },
          { "name": "executorAddr", "type": "address", "internalType": "address" }
        ],
        "outputs": [],
        "stateMutability": "payable"
      },
      {
        "type": "function",
        "name": "completeTask",
        "inputs": [{ "name": "taskId", "type": "uint256", "internalType": "uint256" }],
        "outputs": [],
        "stateMutability": "nonpayable"
      },
      {
        "type": "function",
        "name": "releasePayment",
        "inputs": [
          { "name": "taskId", "type": "uint256", "internalType": "uint256" },
          { "name": "requester", "type": "address", "internalType": "address" }
        ],
        "outputs": [],
        "stateMutability": "nonpayable"
      },
      {
        "type": "event",
        "name": "MeshCreated",
        "inputs": [
          { "name": "name", "type": "string", "indexed": false, "internalType": "string" },
          { "name": "owner", "type": "address", "indexed": false, "internalType": "address" }
        ],
        "anonymous": false
      },
      {
        "type": "event",
        "name": "AgentRegistered",
        "inputs": [
          { "name": "wallet", "type": "address", "indexed": true, "internalType": "address" },
          { "name": "agentId", "type": "string", "indexed": false, "internalType": "string" },
          { "name": "capabilities", "type": "string[]", "indexed": false, "internalType": "string[]" }
        ],
        "anonymous": false
      },
      {
        "type": "event",
        "name": "TaskCreated",
        "inputs": [
          { "name": "taskId", "type": "uint256", "indexed": true, "internalType": "uint256" },
          { "name": "requester", "type": "address", "indexed": true, "internalType": "address" },
          { "name": "executor", "type": "address", "indexed": true, "internalType": "address" },
          { "name": "amount", "type": "uint256", "indexed": false, "internalType": "uint256" }
        ],
        "anonymous": false
      },
      {
        "type": "event",
        "name": "TaskCompleted",
        "inputs": [
          { "name": "taskId", "type": "uint256", "indexed": true, "internalType": "uint256" }
        ],
        "anonymous": false
      },
      {
        "type": "event",
        "name": "PaymentReleased",
        "inputs": [
          { "name": "taskId", "type": "uint256", "indexed": true, "internalType": "uint256" },
          { "name": "amount", "type": "uint256", "indexed": false, "internalType": "uint256" }
        ],
        "anonymous": false
      },
      {
        "type": "event",
        "name": "ReputationUpdated",
        "inputs": [
          { "name": "wallet", "type": "address", "indexed": true, "internalType": "address" },
          { "name": "newReputation", "type": "uint8", "indexed": false, "internalType": "uint8" }
        ],
        "anonymous": false
      }
    ]"#
);

/// A decoded agent from the contract's Agent struct.
/// Field names match the Solidity struct (camelCase as generated by alloy sol!).
#[derive(Debug, Clone)]
pub struct DecodedAgent {
    pub agent_id: String,
    pub capabilities: Vec<String>,
    pub price_per_task: U256,
    pub endpoint: String,
    pub reputation: u8,
    pub active: bool,
}

// Re-export IAgentRegistry for use in other modules if needed
#[allow(unused_imports)]
pub use self::IAgentRegistry as AgentStructs;

/// alloy sol! generates a struct named after the internalType namespace.
/// Since internalType is "struct IAgentRegistry.Agent", the struct lives at
/// IAgentRegistry::Agent inside the generated module.
/// For getAgent (single tuple output), the Return type IS the Agent struct directly.
impl From<IAgentRegistry::Agent> for DecodedAgent {
    fn from(a: IAgentRegistry::Agent) -> Self {
        Self {
            agent_id: a.agentId,
            capabilities: a.capabilities.into_iter().collect(),
            price_per_task: a.pricePerTask,
            endpoint: a.endpoint,
            reputation: a.reputation,
            active: a.active,
        }
    }
}

/// Try to load bytecode from the forge build artifact.
/// Returns None if forge hasn't been run yet.
pub fn load_bytecode_from_artifact() -> Option<alloy::primitives::Bytes> {
    // Paths to look for the artifact relative to CWD
    let candidates = [
        "../agentmesh-contracts/out/AgentRegistry.sol/AgentRegistry.json",
        "agentmesh-contracts/out/AgentRegistry.sol/AgentRegistry.json",
        "out/AgentRegistry.sol/AgentRegistry.json",
    ];
    for path in &candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(bytecode) = json
                    .get("bytecode")
                    .and_then(|b| b.get("object"))
                    .and_then(|o| o.as_str())
                {
                    let hex_str = bytecode.trim_start_matches("0x");
                    if let Ok(bytes) = hex::decode(hex_str) {
                        return Some(alloy::primitives::Bytes::from(bytes));
                    }
                }
            }
        }
    }
    None
}

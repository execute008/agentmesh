/// Hardcoded ABI for AgentRegistry.sol
/// Extracted from src/interfaces/IAgentRegistry.sol + AgentRegistry.sol
/// Will be replaced by out/AgentRegistry.sol/AgentRegistry.json after `forge build`
pub const AGENT_REGISTRY_ABI: &str = r#"[
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
    "name": "AgentUpdated",
    "inputs": [
      { "name": "wallet", "type": "address", "indexed": true, "internalType": "address" },
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
]"#;

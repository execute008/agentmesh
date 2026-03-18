// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IAgentRegistry} from "./interfaces/IAgentRegistry.sol";

contract AgentRegistry is IAgentRegistry {
    // ============================================================
    //                     STATE VARIABLES
    // ============================================================
    string public meshName;
    address public meshOwner;
    mapping(address => Agent) private _agents;
    mapping(address => bool) private _isRegistered;
    address[] private _agentList;
    mapping(uint256 => Task) private _tasks;

    // ============================================================
    //                        EVENTS
    // ============================================================
    event MeshCreated(string name, address owner);
    event AgentRegistered(address indexed wallet, string agentId, string[] capabilities);
    event AgentUpdated(address indexed wallet, string[] capabilities);
    event TaskCreated(uint256 indexed taskId, address indexed requester, address indexed executor, uint256 amount);
    event TaskCompleted(uint256 indexed taskId);
    event PaymentReleased(uint256 indexed taskId, uint256 amount);
    event ReputationUpdated(address indexed wallet, uint8 newReputation);

    // ============================================================
    //                      CONSTRUCTOR
    // ============================================================
    constructor(string memory _meshName) {
        meshName = _meshName;
        meshOwner = msg.sender;
        emit MeshCreated(_meshName, msg.sender);
    }

    // ============================================================
    //                       REGISTRY
    // ============================================================
    /// @notice Register a new agent in this mesh
    /// @dev Sets reputation to 50 and active to true. Reverts if wallet already registered.
    /// @param agentId Unique string identifier for the agent
    /// @param capabilities Array of capability tags the agent advertises (e.g. "web-scraping")
    /// @param pricePerTask Agent's advertised price per task in wei
    /// @param wsEndpoint Public WebSocket URL where agent receives x402 messages (wss://...)
    function registerAgent(
        string calldata agentId,
        string[] calldata capabilities,
        uint256 pricePerTask,
        string calldata wsEndpoint
    ) external {
        require(!_isRegistered[msg.sender], "Already registered");
        _agents[msg.sender] = Agent({
            agentId: agentId,
            capabilities: capabilities,
            pricePerTask: pricePerTask,
            endpoint: wsEndpoint,
            reputation: 50,
            active: true
        });
        _isRegistered[msg.sender] = true;
        _agentList.push(msg.sender);
        emit AgentRegistered(msg.sender, agentId, capabilities);
    }

    /// @notice Get the full Agent struct for a registered wallet
    /// @dev Reverts if wallet is not registered.
    /// @param wallet The agent's wallet address
    /// @return The Agent struct with all fields including endpoint and reputation
    function getAgent(address wallet) external view returns (Agent memory) {
        require(_isRegistered[wallet], "Agent not found");
        return _agents[wallet];
    }

    /// @notice Get all registered agent wallet addresses in this mesh
    /// @return Array of all wallet addresses that have called registerAgent
    function getAllAgents() external view returns (address[] memory) {
        return _agentList;
    }

    /// @notice Search for agents by capability tag
    /// @dev O(agents x capabilities) — suitable for meshes with fewer than 1000 agents
    /// @param cap Exact capability string to search for (e.g. "web-scraping")
    /// @return Array of wallet addresses whose capabilities include the given string
    function searchByCapability(string calldata cap) external view returns (address[] memory) {
        uint256 count = 0;
        for (uint256 i = 0; i < _agentList.length; i++) {
            Agent storage agent = _agents[_agentList[i]];
            for (uint256 j = 0; j < agent.capabilities.length; j++) {
                if (keccak256(bytes(agent.capabilities[j])) == keccak256(bytes(cap))) {
                    count++;
                    break;
                }
            }
        }
        address[] memory result = new address[](count);
        uint256 idx = 0;
        for (uint256 i = 0; i < _agentList.length; i++) {
            Agent storage agent = _agents[_agentList[i]];
            for (uint256 j = 0; j < agent.capabilities.length; j++) {
                if (keccak256(bytes(agent.capabilities[j])) == keccak256(bytes(cap))) {
                    result[idx++] = _agentList[i];
                    break;
                }
            }
        }
        return result;
    }

    // ============================================================
    //                   ESCROW & SETTLEMENT
    // ============================================================
    function createTask(uint256, address) external payable { revert("not implemented"); }
    function completeTask(uint256) external { revert("not implemented"); }
    function releasePayment(uint256, address) external { revert("not implemented"); }
}

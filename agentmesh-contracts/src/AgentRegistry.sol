// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IAgentRegistry} from "./interfaces/IAgentRegistry.sol";

/// @title AgentRegistry
/// @notice On-chain registry for autonomous agent discovery, task escrow, and reputation
/// @dev Covers Registry, Escrow & Settlement, and Reputation bounded contexts in one contract.
///      Deploy one per mesh. CLI scanner depends on MeshCreated event in constructor.
contract AgentRegistry is IAgentRegistry {
    // ============================================================
    //                     STATE VARIABLES
    // ============================================================
    /// @notice Human-readable name of this mesh (set at deployment)
    string public meshName;
    /// @notice Address of the wallet that deployed this mesh
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
    /// @notice Deploy a new AgentMesh registry
    /// @dev Emits MeshCreated. Sets deployer as meshOwner.
    /// @param _meshName Human-readable name for this mesh (e.g. "AgentMesh-Demo")
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
    /// @notice Create a task with escrowed ETH payment
    /// @dev Locks msg.value in the contract until releasePayment is called.
    /// @param taskId Caller-supplied unique task identifier (uint256)
    /// @param executorAddr Registered agent wallet that will execute the task
    function createTask(uint256 taskId, address executorAddr) external payable {
        require(msg.value > 0, "Must send ETH");
        require(_isRegistered[executorAddr], "Executor not registered");
        require(_tasks[taskId].requester == address(0), "Task already exists");
        _tasks[taskId] = Task({
            requester: msg.sender,
            executor: executorAddr,
            escrowAmount: msg.value,
            completed: false,
            released: false
        });
        emit TaskCreated(taskId, msg.sender, executorAddr, msg.value);
    }

    /// @notice Mark a task as completed by the executor
    /// @dev Only the assigned executor can call this. Emits TaskCompleted.
    /// @param taskId The task identifier to mark as completed
    function completeTask(uint256 taskId) external {
        Task storage task = _tasks[taskId];
        require(msg.sender == task.executor, "Only executor");
        require(!task.completed, "Already completed");
        task.completed = true;
        emit TaskCompleted(taskId);
    }

    /// @notice Release escrowed ETH to executor after task completion
    /// @dev Transfers escrowAmount to executor, increments reputation by 5 (capped at 100).
    ///      requester must equal msg.sender — prevents unauthorized release.
    /// @param taskId The task identifier to release payment for
    /// @param requester Must equal msg.sender (the original task requester)
    function releasePayment(uint256 taskId, address requester) external {
        require(requester == msg.sender, "Only requester");
        Task storage task = _tasks[taskId];
        require(task.requester == msg.sender, "Only requester");
        require(task.completed, "Task not completed");
        require(!task.released, "Already released");
        task.released = true;
        uint256 amount = task.escrowAmount;

        Agent storage executor = _agents[task.executor];
        uint8 newRep = executor.reputation + 5;
        if (newRep > 100) newRep = 100;
        executor.reputation = newRep;

        (bool success, ) = payable(task.executor).call{value: amount}("");
        require(success, "Transfer failed");

        emit PaymentReleased(taskId, amount);
        emit ReputationUpdated(task.executor, newRep);
    }
}

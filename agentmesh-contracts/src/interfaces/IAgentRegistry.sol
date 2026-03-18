// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface IAgentRegistry {
    struct Agent {
        string agentId;
        string[] capabilities;
        uint256 pricePerTask;
        string endpoint;
        uint8 reputation;
        bool active;
    }

    struct Task {
        address requester;
        address executor;
        uint256 escrowAmount;
        bool completed;
        bool released;
    }

    function registerAgent(string calldata agentId, string[] calldata capabilities, uint256 pricePerTask, string calldata wsEndpoint) external;
    function getAgent(address wallet) external view returns (Agent memory);
    function getAllAgents() external view returns (address[] memory);
    function searchByCapability(string calldata cap) external view returns (address[] memory);
    function createTask(uint256 taskId, address executorAddr) external payable;
    function completeTask(uint256 taskId) external;
    function releasePayment(uint256 taskId, address requester) external;
}

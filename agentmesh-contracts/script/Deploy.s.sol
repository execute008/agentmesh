// SPDX-License-Identifier: MIT
// Deploy to Sepolia:
//   MESH_NAME=AgentMesh-Demo forge script script/Deploy.s.sol --rpc-url $RPC_URL --account deployer --broadcast
// Verify:
//   forge verify-contract <address> src/AgentRegistry.sol:AgentRegistry --chain sepolia --etherscan-api-key $ETHERSCAN_API_KEY
pragma solidity ^0.8.20;

import {Script, console} from "forge-std/Script.sol";
import {AgentRegistry} from "../src/AgentRegistry.sol";

/// @title Deploy — Foundry deploy script for AgentRegistry
/// @notice Deploys a new AgentRegistry mesh and writes deployment info to meshes.json.
///         Requires MESH_NAME environment variable to be set.
contract Deploy is Script {
    /// @notice Path for the JSON deployment artifact (relative to Foundry project root)
    string constant JSON_PATH = "meshes.json";

    /// @notice Deploy a new AgentRegistry and persist deployment metadata
    /// @dev Requires MESH_NAME env var. Reverts if unset.
    ///      Usage: MESH_NAME=AgentMesh-Demo forge script script/Deploy.s.sol --rpc-url $RPC_URL --broadcast
    function run() external returns (AgentRegistry registry) {
        string memory meshName = vm.envString("MESH_NAME");

        vm.startBroadcast();
        registry = new AgentRegistry(meshName);
        vm.stopBroadcast();

        // Build JSON deployment artifact
        string memory entry = vm.serializeAddress("entry", "address", address(registry));
        entry = vm.serializeString("entry", "name", meshName);
        entry = vm.serializeUint("entry", "deployed_at", block.number);

        string memory json = string.concat("[", entry, "]");
        // vm.writeJson paths are relative to foundry project root (agentmesh-contracts/)
        vm.writeJson(json, JSON_PATH);

        console.log("=== AgentMesh Deployment ===");
        console.log("Contract:", address(registry));
        console.log("Mesh Name:", meshName);
        console.log("Block:", block.number);
        console.log("Chain ID:", block.chainid);
    }
}

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {AgentRegistry} from "../src/AgentRegistry.sol";

contract AgentRegistryTest is Test {
    AgentRegistry registry;
    address alice = makeAddr("alice");
    address bob   = makeAddr("bob");

    function setUp() public {
        registry = new AgentRegistry("TestMesh");
    }

    function _registerBob() internal {
        string[] memory caps = new string[](1);
        caps[0] = "web-scraping";
        vm.prank(bob);
        registry.registerAgent("bot-1", caps, 0.01 ether, "wss://bob:8080");
    }

    // --- Constructor ---
    function test_constructor_setsMeshNameAndOwner() public view {
        assertEq(registry.meshName(), "TestMesh");
        assertEq(registry.meshOwner(), address(this));
    }

    function test_constructor_emitsMeshCreated() public {
        vm.expectEmit(false, false, false, true);
        emit AgentRegistry.MeshCreated("TestMesh", address(this));
        new AgentRegistry("TestMesh");
    }

    // --- registerAgent ---
    function test_registerAgent_storesAgent() public {
        string[] memory caps = new string[](1);
        caps[0] = "analysis";
        vm.prank(alice);
        registry.registerAgent("agent-1", caps, 0.01 ether, "wss://alice:8080");
        AgentRegistry.Agent memory a = registry.getAgent(alice);
        assertEq(a.agentId, "agent-1");
        assertEq(a.reputation, 50);
        assertTrue(a.active);
        assertEq(a.endpoint, "wss://alice:8080");
    }

    function test_registerAgent_emitsAgentRegistered() public {
        string[] memory caps = new string[](1);
        caps[0] = "analysis";
        vm.expectEmit(true, false, false, false);
        emit AgentRegistry.AgentRegistered(alice, "agent-1", caps);
        vm.prank(alice);
        registry.registerAgent("agent-1", caps, 0.01 ether, "wss://alice:8080");
    }

    function test_registerAgent_revertsIfAlreadyRegistered() public {
        string[] memory caps = new string[](1);
        caps[0] = "analysis";
        vm.prank(alice);
        registry.registerAgent("agent-1", caps, 0.01 ether, "wss://alice:8080");
        vm.expectRevert("Already registered");
        vm.prank(alice);
        registry.registerAgent("agent-1", caps, 0.01 ether, "wss://alice:8080");
    }

    // --- getAgent ---
    function test_getAgent_revertsIfNotRegistered() public {
        vm.expectRevert("Agent not found");
        registry.getAgent(alice);
    }

    // --- getAllAgents ---
    function test_getAllAgents_returnsRegisteredAddresses() public {
        string[] memory caps = new string[](1);
        caps[0] = "analysis";
        vm.prank(alice);
        registry.registerAgent("agent-1", caps, 0.01 ether, "wss://alice:8080");
        _registerBob();
        assertEq(registry.getAllAgents().length, 2);
    }

    // --- searchByCapability ---
    function test_searchByCapability_returnsMatching() public {
        string[] memory aliceCaps = new string[](2);
        aliceCaps[0] = "web-scraping";
        aliceCaps[1] = "analysis";
        vm.prank(alice);
        registry.registerAgent("agent-1", aliceCaps, 0.01 ether, "wss://alice:8080");

        string[] memory bobCaps = new string[](1);
        bobCaps[0] = "publishing";
        vm.prank(bob);
        registry.registerAgent("bot-1", bobCaps, 0.01 ether, "wss://bob:8080");

        address[] memory results = registry.searchByCapability("web-scraping");
        assertEq(results.length, 1);
        assertEq(results[0], alice);
    }

    function test_searchByCapability_returnsEmptyForNoMatch() public {
        assertEq(registry.searchByCapability("nonexistent").length, 0);
    }

    // --- createTask ---
    function test_createTask_locksEthAndEmits() public {
        _registerBob();
        hoax(alice, 1 ether);
        registry.createTask{value: 0.01 ether}(1, bob);
        assertEq(address(registry).balance, 0.01 ether);
    }

    function test_createTask_revertsIfExecutorNotRegistered() public {
        vm.deal(alice, 1 ether);
        vm.expectRevert("Executor not registered");
        vm.prank(alice);
        registry.createTask{value: 0.01 ether}(1, bob);
    }

    function test_createTask_revertsIfNoValue() public {
        _registerBob();
        vm.expectRevert("Must send ETH");
        vm.prank(alice);
        registry.createTask(1, bob);
    }

    function test_createTask_revertsIfDuplicateTaskId() public {
        _registerBob();
        hoax(alice, 1 ether);
        registry.createTask{value: 0.01 ether}(1, bob);
        vm.expectRevert("Task already exists");
        hoax(alice, 1 ether);
        registry.createTask{value: 0.01 ether}(1, bob);
    }

    // --- completeTask ---
    function test_completeTask_marksCompleted() public {
        _registerBob();
        hoax(alice, 1 ether);
        registry.createTask{value: 0.01 ether}(1, bob);
        vm.prank(bob);
        registry.completeTask(1); // should not revert
    }

    function test_completeTask_revertsIfNotExecutor() public {
        _registerBob();
        hoax(alice, 1 ether);
        registry.createTask{value: 0.01 ether}(1, bob);
        vm.expectRevert("Only executor");
        vm.prank(alice);
        registry.completeTask(1);
    }

    function test_completeTask_revertsIfAlreadyCompleted() public {
        _registerBob();
        hoax(alice, 1 ether);
        registry.createTask{value: 0.01 ether}(1, bob);
        vm.prank(bob);
        registry.completeTask(1);
        vm.expectRevert("Already completed");
        vm.prank(bob);
        registry.completeTask(1);
    }

    // --- releasePayment ---
    function test_releasePayment_transfersEthAndUpdatesReputation() public {
        _registerBob();
        uint256 bobBefore = bob.balance;
        hoax(alice, 1 ether);
        registry.createTask{value: 0.01 ether}(1, bob);
        vm.prank(bob);
        registry.completeTask(1);
        vm.prank(alice);
        registry.releasePayment(1, alice);
        assertEq(bob.balance, bobBefore + 0.01 ether);
        assertEq(registry.getAgent(bob).reputation, 55);
    }

    function test_releasePayment_revertsIfNotRequester() public {
        _registerBob();
        hoax(alice, 1 ether);
        registry.createTask{value: 0.01 ether}(1, bob);
        vm.prank(bob);
        registry.completeTask(1);
        vm.expectRevert("Only requester");
        vm.prank(bob);
        registry.releasePayment(1, bob);
    }

    function test_releasePayment_revertsIfNotCompleted() public {
        _registerBob();
        hoax(alice, 1 ether);
        registry.createTask{value: 0.01 ether}(1, bob);
        vm.expectRevert("Task not completed");
        vm.prank(alice);
        registry.releasePayment(1, alice);
    }

    function test_releasePayment_revertsIfAlreadyReleased() public {
        _registerBob();
        hoax(alice, 1 ether);
        registry.createTask{value: 0.01 ether}(1, bob);
        vm.prank(bob);
        registry.completeTask(1);
        vm.prank(alice);
        registry.releasePayment(1, alice);
        vm.expectRevert("Already released");
        vm.prank(alice);
        registry.releasePayment(1, alice);
    }

    // --- Full flow ---
    function test_fullEscrowFlow() public {
        string[] memory caps = new string[](1);
        caps[0] = "web-scraping";
        vm.prank(bob);
        registry.registerAgent("bot-1", caps, 0.01 ether, "wss://bob:8080");

        uint256 bobBalanceBefore = bob.balance;
        hoax(alice, 1 ether);
        registry.createTask{value: 0.01 ether}(1, bob);
        assertEq(address(registry).balance, 0.01 ether);

        vm.prank(bob);
        registry.completeTask(1);

        vm.prank(alice);
        registry.releasePayment(1, alice);

        assertEq(bob.balance, bobBalanceBefore + 0.01 ether);
        assertEq(address(registry).balance, 0);
        AgentRegistry.Agent memory agent = registry.getAgent(bob);
        assertEq(agent.reputation, 55);
    }
}

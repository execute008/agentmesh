//! On-chain AgentRegistry query helpers (alloy, read-only).
//!
//! Used by the analyzer to discover scraper endpoints via `searchByCapability`
//! without importing the full CLI codebase.

use alloy::{
    network::EthereumWallet,
    primitives::{Address, U256},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
    sol,
};
use anyhow::{Context, Result};
use crate::crypto::signer_from_hex;

// Minimal inline ABI — read + write functions
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    AgentRegistry,
    r#"[
      {
        "type": "function",
        "name": "registerAgent",
        "inputs": [
          { "name": "agentId", "type": "string", "internalType": "string" },
          { "name": "capabilities", "type": "string[]", "internalType": "string[]" },
          { "name": "pricePerTask", "type": "uint256", "internalType": "uint256" },
          { "name": "endpoint", "type": "string", "internalType": "string" }
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
        "name": "searchByCapability",
        "inputs": [{ "name": "cap", "type": "string", "internalType": "string" }],
        "outputs": [{ "name": "", "type": "address[]", "internalType": "address[]" }],
        "stateMutability": "view"
      }
    ]"#
);

/// Information about a discovered agent sufficient for P2P contact.
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub wallet: Address,
    pub agent_id: String,
    pub endpoint: String,
    pub price_per_task: alloy::primitives::U256,
    pub capabilities: Vec<String>,
    pub reputation: u8,
    pub active: bool,
}

/// Query the AgentRegistry for all agents advertising a given capability tag.
///
/// Returns the full `AgentInfo` for each match, ready to be used by
/// `send_to_agent` in the transport module.
///
/// # Arguments
/// * `rpc_url`           – HTTP(S) JSON-RPC endpoint
/// * `registry_address`  – deployed AgentRegistry contract address
/// * `capability`        – capability tag to search (e.g. "web-scraping")
pub async fn find_agents_by_capability(
    rpc_url: &str,
    registry_address: &str,
    capability: &str,
) -> Result<Vec<AgentInfo>> {
    let url: url::Url = rpc_url.parse().context("parsing RPC URL")?;
    let provider = ProviderBuilder::new().connect_http(url);

    let addr: Address = registry_address
        .parse()
        .context("parsing registry address")?;

    let contract = AgentRegistry::new(addr, &provider);

    // Discover wallets with the requested capability
    let wallets: Vec<Address> = contract
        .searchByCapability(capability.to_string())
        .call()
        .await
        .context("searchByCapability call")?;

    // Fetch full agent info for each wallet
    let mut results = Vec::new();
    for wallet in wallets {
        match contract.getAgent(wallet).call().await {
            Ok(agent) => {
                results.push(AgentInfo {
                    wallet,
                    agent_id: agent.agentId,
                    endpoint: agent.endpoint,
                    price_per_task: agent.pricePerTask,
                    capabilities: agent.capabilities.into_iter().collect(),
                    reputation: agent.reputation,
                    active: agent.active,
                });
            }
            Err(e) => {
                tracing::warn!("Could not fetch agent {wallet}: {e}");
            }
        }
    }

    Ok(results)
}

/// Fetch a single agent's info by wallet address.
pub async fn get_agent(
    rpc_url: &str,
    registry_address: &str,
    wallet: Address,
) -> Result<AgentInfo> {
    let url: url::Url = rpc_url.parse().context("parsing RPC URL")?;
    let provider = ProviderBuilder::new().connect_http(url);

    let addr: Address = registry_address
        .parse()
        .context("parsing registry address")?;

    let contract = AgentRegistry::new(addr, &provider);
    let agent = contract
        .getAgent(wallet)
        .call()
        .await
        .context("getAgent call")?;

    Ok(AgentInfo {
        wallet,
        agent_id: agent.agentId,
        endpoint: agent.endpoint,
        price_per_task: agent.pricePerTask,
        capabilities: agent.capabilities.into_iter().collect(),
        reputation: agent.reputation,
        active: agent.active,
    })
}

/// Register this agent on-chain.
///
/// Returns Ok(true) if registration succeeded, Ok(false) if already registered.
pub async fn register_agent(
    rpc_url: &str,
    registry_address: &str,
    private_key: &str,
    agent_id: &str,
    capabilities: Vec<String>,
    price_per_task: U256,
    endpoint: &str,
) -> Result<bool> {
    let signer: PrivateKeySigner = signer_from_hex(private_key)?;
    let my_address = crate::crypto::address_of(&signer);
    
    // Check if already registered
    let my_addr: Address = my_address.parse()?;
    match get_agent(rpc_url, registry_address, my_addr).await {
        Ok(existing) if existing.active => {
            tracing::info!("Already registered as {}", existing.agent_id);
            return Ok(false);
        }
        _ => {}
    }

    let wallet = EthereumWallet::from(signer);
    let url: url::Url = rpc_url.parse().context("parsing RPC URL")?;
    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect_http(url);

    let addr: Address = registry_address
        .parse()
        .context("parsing registry address")?;

    let contract = AgentRegistry::new(addr, &provider);

    tracing::info!("📝 Registering agent {} on-chain...", agent_id);
    
    let tx = contract
        .registerAgent(
            agent_id.to_string(),
            capabilities,
            price_per_task,
            endpoint.to_string(),
        )
        .send()
        .await
        .context("registerAgent transaction")?;

    let receipt = tx.get_receipt().await.context("waiting for receipt")?;
    
    tracing::info!("✅ Registered! Tx: {:?}", receipt.transaction_hash);
    Ok(true)
}

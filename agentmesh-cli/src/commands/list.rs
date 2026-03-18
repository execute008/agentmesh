use anyhow::{Context, Result};
use colored::Colorize;

use alloy::{
    primitives::Address,
    providers::ProviderBuilder,
};

use crate::{
    contract::{AgentRegistry, DecodedAgent},
    types::rpc_url_for_chain,
};

/// List all registered agents in a mesh contract
pub async fn run(contract_addr: &str, chain: &str) -> Result<()> {
    let addr: Address = contract_addr
        .parse()
        .with_context(|| format!("Invalid contract address: {contract_addr}"))?;

    let rpc_url = rpc_url_for_chain(chain)?;
    println!(
        "{} Querying agents on {}",
        "📋".cyan(),
        contract_addr.yellow()
    );

    let provider = ProviderBuilder::new()
        .connect_http(rpc_url.parse()?);

    let registry = AgentRegistry::new(addr, provider);

    // Fetch mesh metadata
    // Single-output functions: .call().await? returns the value directly
    let mesh_name: String = registry.meshName().call().await?;
    let mesh_owner: Address = registry.meshOwner().call().await?;
    println!("   Mesh: {} (owner: {})", mesh_name.bold(), format!("{mesh_owner}").dimmed());

    // Get all agent wallets — single output: Vec<Address>
    let wallets: Vec<Address> = registry.getAllAgents().call().await?;

    if wallets.is_empty() {
        println!("\n{} No agents registered in this mesh.", "ℹ".blue());
        return Ok(());
    }

    println!("\n{} {} agent(s) registered:\n", "🤖".cyan(), wallets.len().to_string().bold());

    for wallet in &wallets {
        match registry.getAgent(*wallet).call().await {
            Ok(agent_struct) => {
                let agent = DecodedAgent::from(agent_struct);
                print_agent(*wallet, &agent);
            }
            Err(e) => {
                eprintln!("  {} Failed to fetch agent {wallet}: {e}", "⚠".yellow());
            }
        }
    }

    Ok(())
}

fn print_agent(wallet: Address, agent: &DecodedAgent) {
    let status = if agent.active {
        "active".green()
    } else {
        "inactive".red()
    };

    println!("  {} {} [{}]", "•".cyan(), agent.agent_id.bold(), status);
    println!("    Wallet:       {}", format!("{wallet:#x}").yellow());
    println!("    Capabilities: {}", agent.capabilities.join(", ").italic());
    println!(
        "    Price:        {} wei",
        agent.price_per_task.to_string().cyan()
    );
    println!(
        "    Reputation:   {}/100",
        format!("{}", agent.reputation).bold()
    );
    println!("    Endpoint:     {}", agent.endpoint.dimmed());
    println!();
}

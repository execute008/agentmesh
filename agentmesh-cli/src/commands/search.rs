use anyhow::{Context, Result};
use colored::Colorize;

use alloy::{
    primitives::Address,
    providers::ProviderBuilder,
};

use crate::{
    contract::{AgentRegistry, DecodedAgent},
    meshes::load_meshes,
    types::rpc_url_for_chain,
};

/// Search for a capability across all meshes in meshes.json
pub async fn run(capability: &str, chain: &str) -> Result<()> {
    let meshes = load_meshes()?;

    if meshes.is_empty() {
        anyhow::bail!(
            "No meshes known. Run `agentmesh scan --chain {}` first.",
            chain
        );
    }

    let rpc_url = rpc_url_for_chain(chain)?;
    println!(
        "{} Searching {} mesh(es) for capability: {}",
        "🔎".cyan(),
        meshes.len().to_string().bold(),
        capability.bold().yellow()
    );

    let provider = ProviderBuilder::new()
        .connect_http(rpc_url.parse()?);

    let mut total_found = 0usize;

    for mesh in &meshes {
        let addr: Address = mesh
            .address
            .parse()
            .with_context(|| format!("Invalid address in meshes.json: {}", mesh.address))?;

        let registry = AgentRegistry::new(addr, provider.clone());
        let wallets: Vec<Address> = match registry
            .searchByCapability(capability.to_string())
            .call()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!(
                    "  {} Failed to search mesh {}: {e}",
                    "⚠".yellow(),
                    mesh.name
                );
                continue;
            }
        };

        if wallets.is_empty() {
            continue;
        }

        println!(
            "\n  {} {} — {} match(es)",
            "📍".cyan(),
            mesh.name.bold(),
            wallets.len().to_string().green()
        );
        println!("     Contract: {}", mesh.address.yellow());

        for wallet in &wallets {
            match registry.getAgent(*wallet).call().await {
                Ok(agent_struct) => {
                    let agent = DecodedAgent::from(agent_struct);
                    println!(
                        "\n    {} {} ({})",
                        "•".cyan(),
                        agent.agent_id.bold(),
                        format!("{wallet:#x}").dimmed()
                    );
                    println!(
                        "      Caps:      {}",
                        agent.capabilities.join(", ").italic()
                    );
                    println!(
                        "      Price:     {} wei",
                        agent.price_per_task.to_string().cyan()
                    );
                    println!(
                        "      Rep:       {}/100",
                        format!("{}", agent.reputation).bold()
                    );
                    println!("      Endpoint:  {}", agent.endpoint.dimmed());
                    total_found += 1;
                }
                Err(e) => {
                    eprintln!("    {} Failed to fetch {wallet}: {e}", "⚠".yellow());
                }
            }
        }
    }

    println!(
        "\n{} Found {} agent(s) with capability '{}'",
        "✓".green(),
        total_found.to_string().bold(),
        capability.yellow()
    );

    Ok(())
}

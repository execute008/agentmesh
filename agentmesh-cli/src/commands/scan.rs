use anyhow::Result;
use colored::Colorize;

use alloy::{
    primitives::{keccak256, Address, B256},
    providers::{Provider, ProviderBuilder},
    rpc::types::{Filter, Log},
    sol_types::SolEvent,
};

use crate::{
    contract::AgentRegistry,
    meshes::{load_meshes, save_meshes, upsert_mesh},
    types::{rpc_url_for_chain, MeshEntry},
};

/// Scan a chain for all MeshCreated events and populate meshes.json
pub async fn run(chain: &str, from_block: Option<u64>) -> Result<()> {
    let rpc_url = rpc_url_for_chain(chain)?;
    println!("{} Scanning {} for MeshCreated events...", "🔍".cyan(), chain.bold());
    println!("   RPC: {}", rpc_url.dimmed());

    let provider = ProviderBuilder::new()
        .connect_http(rpc_url.parse()?);

    let latest = provider.get_block_number().await?;
    let start = from_block.unwrap_or(0);
    println!("   Blocks: {} → {}", start, latest);

    // MeshCreated(string name, address owner)
    // Event topic = keccak256("MeshCreated(string,address)")
    let event_sig = keccak256("MeshCreated(string,address)");
    let topic: B256 = B256::from(event_sig);

    // Scan in chunks of 10_000 blocks to avoid RPC limits
    let chunk_size: u64 = 10_000;
    let mut all_logs: Vec<Log> = Vec::new();
    let mut block = start;

    while block <= latest {
        let end = (block + chunk_size - 1).min(latest);
        let filter = Filter::new()
            .from_block(block)
            .to_block(end)
            .event_signature(topic);

        let logs = provider.get_logs(&filter).await?;
        all_logs.extend(logs);
        block = end + 1;
    }

    println!("   Found {} MeshCreated event(s)", all_logs.len().to_string().bold());

    let mut meshes = load_meshes()?;
    let mut new_count = 0usize;

    for log in &all_logs {
        let contract_addr: Address = log.address();

        // decode_log takes a single &alloy_primitives::Log reference
        match AgentRegistry::MeshCreated::decode_log(log.as_ref()) {
            Ok(decoded) => {
                let mesh_name = decoded.data.name.clone();
                let deployed_at = log.block_number.unwrap_or(0);
                println!(
                    "   {} {} at {} (block {})",
                    "✓".green(),
                    mesh_name.bold(),
                    format!("{contract_addr}").yellow(),
                    deployed_at
                );
                let addr_str = format!("{contract_addr:#x}");
                let is_new = !meshes
                    .iter()
                    .any(|m| m.address.to_lowercase() == addr_str.to_lowercase());
                if is_new {
                    new_count += 1;
                }
                upsert_mesh(
                    &mut meshes,
                    MeshEntry {
                        address: addr_str,
                        name: mesh_name,
                        deployed_at,
                    },
                );
            }
            Err(e) => {
                eprintln!(
                    "   {} Failed to decode MeshCreated log at {:?}: {e}",
                    "⚠".yellow(),
                    log.transaction_hash
                );
            }
        }
    }

    save_meshes(&meshes)?;
    println!(
        "\n{} meshes.json updated ({} total, {} new)",
        "✓".green(),
        meshes.len(),
        new_count
    );

    Ok(())
}

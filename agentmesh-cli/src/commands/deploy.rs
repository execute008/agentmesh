use anyhow::{Context, Result};
use colored::Colorize;

use alloy::{
    network::{EthereumWallet, TransactionBuilder},
    primitives::Address,
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
    sol_types::SolConstructor,
};

use crate::{
    contract::{load_bytecode_from_artifact, AgentRegistry},
    meshes::{load_meshes, save_meshes, upsert_mesh},
    types::{rpc_url_for_chain, MeshEntry},
};

/// Deploy a new AgentRegistry contract and record it in meshes.json
pub async fn run(
    mesh_name: &str,
    rpc_url_override: Option<&str>,
    chain: &str,
    private_key: Option<&str>,
    keystore_path: Option<&str>,
) -> Result<()> {
    println!(
        "{} Deploying AgentRegistry mesh: {}",
        "🚀".cyan(),
        mesh_name.bold()
    );

    let signer = load_signer(private_key, keystore_path)?;
    let wallet_addr = signer.address();
    println!("   Wallet: {}", format!("{wallet_addr:#x}").yellow());

    let rpc_url = match rpc_url_override {
        Some(url) => url.to_string(),
        None => rpc_url_for_chain(chain)?,
    };
    println!("   RPC:    {}", rpc_url.dimmed());

    // Load bytecode from forge artifact
    let bytecode = load_bytecode_from_artifact().ok_or_else(|| {
        anyhow::anyhow!(
            "Contract bytecode not found.\n\
             Run `forge build` in agentmesh-contracts/ first, then retry.\n\
             Expected at: agentmesh-contracts/out/AgentRegistry.sol/AgentRegistry.json"
        )
    })?;

    let wallet = EthereumWallet::from(signer);
    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect_http(rpc_url.parse()?);

    // Encode constructor calldata: constructor(string _meshName)
    let constructor_call = AgentRegistry::constructorCall {
        _meshName: mesh_name.to_string(),
    };
    let constructor_data = constructor_call.abi_encode();

    // Concatenate bytecode + ABI-encoded constructor args
    let mut deploy_data = bytecode.to_vec();
    deploy_data.extend_from_slice(&constructor_data);
    let deploy_bytes = alloy::primitives::Bytes::from(deploy_data);

    println!("{} Sending deploy transaction...", "⏳".cyan());

    // Build deploy transaction using TransactionBuilder trait
    let tx_req = TransactionRequest::default().with_deploy_code(deploy_bytes);

    let pending = provider
        .send_transaction(tx_req)
        .await
        .context("Failed to send deploy transaction")?;

    let receipt = pending
        .get_receipt()
        .await
        .context("Failed to get deploy receipt")?;

    let contract_addr: Address = receipt
        .contract_address
        .context("No contract address in receipt — deploy failed?")?;

    let block = receipt.block_number.unwrap_or(0);
    let tx_hash = receipt.transaction_hash;

    println!("{} AgentRegistry deployed!", "✓".green());
    println!("   Address: {}", format!("{contract_addr:#x}").green().bold());
    println!("   Tx hash: {}", format!("{tx_hash:#x}").yellow());
    println!("   Block:   {}", block);

    // Update meshes.json
    let mut meshes = load_meshes()?;
    upsert_mesh(
        &mut meshes,
        MeshEntry {
            address: format!("{contract_addr:#x}"),
            name: mesh_name.to_string(),
            deployed_at: block,
        },
    );
    save_meshes(&meshes)?;

    println!(
        "\n{} meshes.json updated ({} total meshes)",
        "✓".green(),
        meshes.len()
    );

    Ok(())
}

/// Load a signer from private key (env/arg) or keystore file
fn load_signer(
    private_key: Option<&str>,
    keystore_path: Option<&str>,
) -> Result<PrivateKeySigner> {
    if let Some(pk) = private_key {
        return pk.parse().context("Invalid private key format");
    }
    if let Ok(pk) = std::env::var("PRIVATE_KEY") {
        return pk.parse().context("Invalid PRIVATE_KEY env var format");
    }
    if let Some(ks_path) = keystore_path {
        let password = rpassword::prompt_password(format!("Keystore password for {ks_path}: "))
            .context("Failed to read keystore password")?;
        return PrivateKeySigner::decrypt_keystore(ks_path, password)
            .context("Failed to decrypt keystore");
    }
    anyhow::bail!(
        "No signer provided. Set PRIVATE_KEY env var, pass --private-key, or --account <keystore>"
    )
}

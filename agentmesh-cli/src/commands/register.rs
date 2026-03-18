use anyhow::{Context, Result};
use colored::Colorize;

use alloy::{
    network::EthereumWallet,
    primitives::{Address, U256},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
};

use crate::{
    contract::AgentRegistry,
    types::rpc_url_for_chain,
};

/// Register an agent on an AgentRegistry contract
pub async fn run(
    contract_addr: &str,
    agent_id: &str,
    capabilities: &[String],
    price_wei: &str,
    endpoint: &str,
    chain: &str,
    private_key: Option<&str>,
    keystore_path: Option<&str>,
) -> Result<()> {
    let addr: Address = contract_addr
        .parse()
        .with_context(|| format!("Invalid contract address: {contract_addr}"))?;

    let price: U256 = price_wei
        .parse()
        .with_context(|| format!("Invalid price (expected wei integer): {price_wei}"))?;

    println!("{} Registering agent on {}", "✍".cyan(), contract_addr.yellow());
    println!("   Agent ID:     {}", agent_id.bold());
    println!("   Capabilities: {}", capabilities.join(", ").italic());
    println!("   Price:        {} wei", price.to_string().cyan());
    println!("   Endpoint:     {}", endpoint.dimmed());

    let signer = load_signer(private_key, keystore_path)?;
    let wallet_addr = signer.address();
    println!("   Wallet:       {}", format!("{wallet_addr:#x}").yellow());

    let rpc_url = rpc_url_for_chain(chain)?;
    let wallet = EthereumWallet::from(signer);
    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect_http(rpc_url.parse()?);

    let registry = AgentRegistry::new(addr, provider);

    println!("\n{} Sending registerAgent transaction...", "⏳".cyan());

    let tx = registry
        .registerAgent(
            agent_id.to_string(),
            capabilities.to_vec(),
            price,
            endpoint.to_string(),
        )
        .send()
        .await
        .context("Failed to send registerAgent transaction")?;

    let receipt = tx
        .get_receipt()
        .await
        .context("Failed to get transaction receipt")?;

    let tx_hash = receipt.transaction_hash;
    let block = receipt.block_number.unwrap_or(0);

    println!("{} Agent registered!", "✓".green());
    println!("   Tx hash: {}", format!("{tx_hash:#x}").yellow());
    println!("   Block:   {}", block);

    Ok(())
}

/// Load a signer from private key (env/arg) or keystore file
fn load_signer(
    private_key: Option<&str>,
    keystore_path: Option<&str>,
) -> Result<PrivateKeySigner> {
    // Priority: explicit arg → PRIVATE_KEY env → keystore file
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

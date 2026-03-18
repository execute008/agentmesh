use serde::{Deserialize, Serialize};

/// A discovered AgentMesh contract (entry in meshes.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshEntry {
    pub address: String,
    pub name: String,
    pub deployed_at: u64,
}

/// meshes.json path relative to CWD
pub const MESHES_JSON: &str = "meshes.json";

/// Default RPC URLs for supported chains
pub fn rpc_url_for_chain(chain: &str) -> anyhow::Result<String> {
    // Check env first
    let env_key = format!("{}_RPC_URL", chain.to_uppercase().replace('-', "_"));
    if let Ok(url) = std::env::var(&env_key) {
        return Ok(url);
    }
    // Also try bare RPC_URL
    if let Ok(url) = std::env::var("RPC_URL") {
        return Ok(url);
    }
    match chain {
        "sepolia" => Ok("https://rpc.sepolia.org".to_string()),
        "mainnet" => Ok("https://eth.llamarpc.com".to_string()),
        "anvil" | "local" => Ok("http://127.0.0.1:8545".to_string()),
        _ => Err(anyhow::anyhow!(
            "Unknown chain '{}'. Set {}_RPC_URL env var or use sepolia/mainnet/anvil.",
            chain,
            chain.to_uppercase()
        )),
    }
}

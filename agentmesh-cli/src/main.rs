#[allow(dead_code)]
mod abi;
mod commands;
mod contract;
mod meshes;
mod types;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

/// AgentMesh — Decentralized agent coordination CLI
#[derive(Parser)]
#[command(
    name = "agentmesh",
    version = "0.1.0",
    about = "AgentMesh: decentralized agent coordination on Ethereum",
    long_about = "Interact with AgentRegistry smart contracts — deploy meshes, register agents,\nscan chains for deployed meshes, and search for agent capabilities cross-mesh."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Deploy a new AgentRegistry mesh contract
    Deploy {
        /// Human-readable mesh name (e.g. "AgentMesh-Demo")
        #[arg(long)]
        name: String,

        /// RPC URL (overrides chain default and env var)
        #[arg(long)]
        rpc_url: Option<String>,

        /// Target chain (sepolia | mainnet | anvil)
        #[arg(long, default_value = "sepolia")]
        chain: String,

        /// Private key (hex, 0x-prefixed). Falls back to PRIVATE_KEY env var.
        #[arg(long)]
        private_key: Option<String>,

        /// Path to encrypted keystore file (EIP-55 JSON)
        #[arg(long)]
        account: Option<String>,
    },

    /// Scan a chain for MeshCreated events and populate meshes.json
    Scan {
        /// Chain to scan
        #[arg(long, default_value = "sepolia")]
        chain: String,

        /// Start block (0 = genesis, slow on mainnet)
        #[arg(long)]
        from_block: Option<u64>,
    },

    /// List all registered agents in a mesh contract
    List {
        /// AgentRegistry contract address (0x...)
        #[arg(long)]
        contract: String,

        /// Chain for RPC connection
        #[arg(long, default_value = "sepolia")]
        chain: String,
    },

    /// Search for a capability across all known meshes (meshes.json)
    Search {
        /// Capability string to search for (e.g. "web-scraping")
        capability: String,

        /// Chain for RPC connection
        #[arg(long, default_value = "sepolia")]
        chain: String,
    },

    /// Register an agent on an existing AgentRegistry mesh
    Register {
        /// AgentRegistry contract address
        #[arg(long)]
        contract: String,

        /// Unique agent ID string
        #[arg(long)]
        agent_id: String,

        /// Capabilities (can be specified multiple times)
        #[arg(long = "capabilities", value_delimiter = ',')]
        capabilities: Vec<String>,

        /// Price per task in wei
        #[arg(long)]
        price: String,

        /// WebSocket endpoint URL (e.g. wss://agent.example.com)
        #[arg(long)]
        endpoint: String,

        /// Chain for RPC connection
        #[arg(long, default_value = "sepolia")]
        chain: String,

        /// Private key (hex, 0x-prefixed). Falls back to PRIVATE_KEY env var.
        #[arg(long)]
        private_key: Option<String>,

        /// Path to encrypted keystore file (EIP-55 JSON)
        #[arg(long)]
        account: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env if present
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Deploy {
            name,
            rpc_url,
            chain,
            private_key,
            account,
        } => {
            commands::deploy::run(
                &name,
                rpc_url.as_deref(),
                &chain,
                private_key.as_deref(),
                account.as_deref(),
            )
            .await
        }

        Commands::Scan { chain, from_block } => {
            commands::scan::run(&chain, from_block).await
        }

        Commands::List { contract, chain } => {
            commands::list::run(&contract, &chain).await
        }

        Commands::Search { capability, chain } => {
            commands::search::run(&capability, &chain).await
        }

        Commands::Register {
            contract,
            agent_id,
            capabilities,
            price,
            endpoint,
            chain,
            private_key,
            account,
        } => {
            commands::register::run(
                &contract,
                &agent_id,
                &capabilities,
                &price,
                &endpoint,
                &chain,
                private_key.as_deref(),
                account.as_deref(),
            )
            .await
        }
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        // Print chain of causes
        let mut cause = e.source();
        while let Some(c) = cause {
            eprintln!("  {} {}", "caused by:".dimmed(), c);
            cause = c.source();
        }
        std::process::exit(1);
    }

    Ok(())
}

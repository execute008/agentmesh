//! AgentMesh Analyzer Agent
//!
//! The central orchestrator of the scraper → analyzer → publisher pipeline.
//!
//! On boot it:
//!   1. Queries the AgentRegistry for "web-scraping" agents
//!   2. Picks the first active one, sends a TaskRequest x402 message P2P
//!   3. Also listens on ws://0.0.0.0:8081 for inbound x402 (TaskComplete from scraper)
//!   4. On TaskComplete: extracts <title>, calls completeTask + releasePayment on-chain
//!   5. Sends a Notification x402 to the publisher agent with the title
//!
//! Environment variables:
//!   ANALYZER_PRIVATE_KEY   – hex private key for this agent's wallet
//!   RPC_URL                – Ethereum JSON-RPC endpoint
//!   REGISTRY_ADDRESS       – deployed AgentRegistry contract address
//!   PUBLISHER_ADDRESS      – publisher agent's wallet address (for on-chain lookup)
//!   TARGET_URL             – URL to scrape (default: https://example.com)
//!   ANALYZER_PORT          – WebSocket port (default: 8081)

use std::net::SocketAddr;

use alloy::{
    network::EthereumWallet,
    primitives::{Address, U256},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
    sol,
};
use anyhow::{Context, Result};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::StreamExt;
use serde_json::json;
use shared::{
    crypto::{address_of, sign_message, signer_from_hex, verify_signature},
    registry::find_agents_by_capability,
    transport::{send_and_receive, send_to_agent},
    types::{
        TaskCompletePayload, TaskRequestPayload, X402Message, X402MessageType,
    },
};
use tracing::{error, info, warn};

// Minimal ABI for the write functions we need (completeTask + releasePayment)
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    AgentRegistryWriter,
    r#"[
      {
        "type": "function",
        "name": "createTask",
        "inputs": [
          { "name": "taskId", "type": "uint256", "internalType": "uint256" },
          { "name": "executorAddr", "type": "address", "internalType": "address" }
        ],
        "outputs": [],
        "stateMutability": "payable"
      },
      {
        "type": "function",
        "name": "completeTask",
        "inputs": [{ "name": "taskId", "type": "uint256", "internalType": "uint256" }],
        "outputs": [],
        "stateMutability": "nonpayable"
      },
      {
        "type": "function",
        "name": "releasePayment",
        "inputs": [
          { "name": "taskId", "type": "uint256", "internalType": "uint256" },
          { "name": "requester", "type": "address", "internalType": "address" }
        ],
        "outputs": [],
        "stateMutability": "nonpayable"
      }
    ]"#
);

#[derive(Clone)]
struct AppState {
    private_key: String,
    rpc_url: String,
    registry_address: String,
    publisher_address: String,
    my_address: String,
    my_port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "analyzer=debug,shared=debug,info".parse().unwrap()),
        )
        .init();

    let private_key = std::env::var("ANALYZER_PRIVATE_KEY")
        .expect("ANALYZER_PRIVATE_KEY must be set");
    let rpc_url = std::env::var("RPC_URL")
        .unwrap_or_else(|_| "https://ethereum-sepolia-rpc.publicnode.com".to_string());
    let registry_address = std::env::var("REGISTRY_ADDRESS")
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string());
    let publisher_address = std::env::var("PUBLISHER_ADDRESS")
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string());
    let target_url = std::env::var("TARGET_URL")
        .unwrap_or_else(|_| "https://example.com".to_string());
    let port: u16 = std::env::var("ANALYZER_PORT")
        .unwrap_or_else(|_| "8081".to_string())
        .parse()
        .expect("ANALYZER_PORT must be a number");

    let signer = signer_from_hex(&private_key)?;
    let my_address = address_of(&signer);

    info!("🧠 Analyzer agent starting");
    info!("   address   : {my_address}");
    info!("   registry  : {registry_address}");
    info!("   publisher : {publisher_address}");
    info!("   target    : {target_url}");
    info!("   port      : {port}");

    let state = AppState {
        private_key: private_key.clone(),
        rpc_url: rpc_url.clone(),
        registry_address: registry_address.clone(),
        publisher_address: publisher_address.clone(),
        my_address: my_address.clone(),
        my_port: port,
    };

    // Start the WS server in background (receives TaskComplete from scraper)
    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Err(e) = run_ws_server(state_clone).await {
            error!("WS server crashed: {e:#}");
        }
    });

    // Small delay to let the server bind
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Kick off the pipeline
    if let Err(e) = run_pipeline(state).await {
        error!("Pipeline error: {e:#}");
    }

    // Keep alive — the WS server handles the TaskComplete reply
    tokio::signal::ctrl_c().await?;
    info!("Shutting down");
    Ok(())
}

// ── WS Server ────────────────────────────────────────────────────────────────

async fn run_ws_server(state: AppState) -> Result<()> {
    let app = Router::new()
        .route("/", get(ws_handler))
        .route("/health", get(health))
        .with_state(state.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], state.my_port));
    info!("Listening on ws://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> &'static str {
    "analyzer ok"
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    while let Some(Ok(msg)) = socket.next().await {
        match msg {
            Message::Text(text) => {
                if let Err(e) = process_inbound(&text, &state).await {
                    error!("Error processing inbound message: {e:#}");
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}

// ── Inbound handler (TaskComplete from scraper) ───────────────────────────────

async fn process_inbound(text: &str, state: &AppState) -> Result<()> {
    let msg = X402Message::from_json(text)?;

    info!(
        "📨 {:?} from {} (msg_id: {})",
        msg.message_type, msg.from, msg.message_id
    );

    match verify_signature(&msg) {
        Ok(true) => info!("✅ Signature valid"),
        Ok(false) => {
            warn!("❌ Invalid signature from {}", msg.from);
            return Ok(());
        }
        Err(e) => warn!("⚠️  Signature check skipped: {e}"),
    }

    match msg.message_type {
        X402MessageType::TaskComplete => {
            let payload: TaskCompletePayload = serde_json::from_value(msg.payload.clone())
                .context("parsing TaskComplete payload")?;

            info!(
                "✅ TaskComplete for task {}: status={}",
                payload.task_id, payload.status
            );

            // Extract <title> from the scraped HTML
            let title = extract_title(&payload.result)
                .unwrap_or_else(|| "no title found".to_string());

            info!("📑 Extracted title: {title}");

            // Settle on-chain: completeTask + releasePayment
            if let Err(e) = settle_on_chain(state, payload.task_id, &msg.from).await {
                warn!("On-chain settlement skipped (likely dev mode / no funds): {e}");
            }

            // Forward title to publisher P2P
            if let Err(e) = forward_to_publisher(state, payload.task_id, &title, &payload.result).await {
                error!("Failed to forward to publisher: {e:#}");
            }
        }
        other => {
            warn!("Analyzer received unexpected message type: {other:?}");
        }
    }

    Ok(())
}

// ── Pipeline: discover scraper → send TaskRequest ─────────────────────────────

async fn run_pipeline(state: AppState) -> Result<()> {
    let target_url = std::env::var("TARGET_URL")
        .unwrap_or_else(|_| "https://example.com".to_string());

    info!("🔍 Searching registry for web-scraping agents…");

    // Discover scraper agents from on-chain registry (gracefully fall back on error)
    let agents = match find_agents_by_capability(
        &state.rpc_url,
        &state.registry_address,
        "web-scraping",
    )
    .await
    {
        Ok(list) => list,
        Err(e) => {
            warn!("Registry query failed ({e}), falling back to localhost:8080");
            vec![]
        }
    };

    if agents.is_empty() {
        warn!("No web-scraping agents found in registry. Falling back to localhost:8080.");
        // Dev fallback — talk directly to the local scraper
        let scraper_endpoint = "ws://127.0.0.1:8080";
        let scraper_wallet = "0x0000000000000000000000000000000000000001";
        send_task_request(&state, scraper_wallet, scraper_endpoint, &target_url, 1).await?;
        return Ok(());
    }

    let scraper = agents.into_iter().find(|a| a.active).ok_or_else(|| {
        anyhow::anyhow!("All discovered web-scraping agents are inactive")
    })?;

    info!(
        "🎯 Found scraper: {} @ {}",
        scraper.agent_id, scraper.endpoint
    );

    // Generate a deterministic-ish task ID from timestamp
    let task_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let scraper_wallet = format!("{:?}", scraper.wallet);
    send_task_request(&state, &scraper_wallet, &scraper.endpoint, &target_url, task_id).await?;

    Ok(())
}

async fn send_task_request(
    state: &AppState,
    scraper_wallet: &str,
    scraper_endpoint: &str,
    target_url: &str,
    task_id: u64,
) -> Result<()> {
    let signer = signer_from_hex(&state.private_key)?;

    let task_payload = TaskRequestPayload {
        task_id,
        description: format!("Scrape and extract title from {target_url}"),
        url: target_url.to_string(),
        registry_address: state.registry_address.clone(),
    };

    let mut msg = X402Message::new_unsigned(
        state.my_address.clone(),
        scraper_wallet.to_string(),
        X402MessageType::TaskRequest,
        serde_json::to_value(&task_payload)?,
    );

    sign_message(&mut msg, &signer).await?;

    info!("📤 TaskRequest → {scraper_wallet} (task {task_id}) via {scraper_endpoint}");

    // Use send_and_receive — the scraper replies on the same WS
    match send_and_receive(scraper_endpoint, &msg).await {
        Ok(reply) => {
            info!("📨 Direct reply received from scraper");
            process_inbound(&reply.to_json()?, state).await?;
        }
        Err(e) => {
            // Scraper may have sent reply via separate connection to our WS server.
            // That's fine — the WS server handler will catch it.
            warn!("send_and_receive failed (scraper may respond via our WS server): {e}");
        }
    }

    Ok(())
}

// ── On-chain settlement ───────────────────────────────────────────────────────

async fn settle_on_chain(state: &AppState, task_id: u64, scraper_wallet: &str) -> Result<()> {
    info!("⛓️  Settling task {task_id} on-chain…");

    let signer: PrivateKeySigner = signer_from_hex(&state.private_key)?;
    let wallet = EthereumWallet::from(signer);

    let rpc_url: url::Url = state.rpc_url.parse().context("parsing RPC URL")?;
    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect_http(rpc_url);

    let registry_addr: Address = state
        .registry_address
        .parse()
        .context("parsing registry address")?;

    let contract = AgentRegistryWriter::new(registry_addr, &provider);

    // completeTask — called by the executor (us, acting on behalf of the scraper
    // in this demo since both share the same registry). In a real deployment
    // this would be called by the scraper agent's own wallet.
    // Here the analyzer calls it after receiving the TaskComplete x402 message
    // as proof-of-work from the scraper.
    let task_id_u256 = U256::from(task_id);

    match contract.completeTask(task_id_u256).send().await {
        Ok(pending) => {
            let tx_hash = *pending.tx_hash();
            // Wait for receipt in background to avoid blocking
            info!("✅ completeTask submitted: {tx_hash:?}");
            drop(pending);
        }
        Err(e) => {
            warn!("completeTask failed (task may not exist on-chain in dev): {e}");
        }
    }

    // releasePayment — called by the requester (analyzer's wallet)
    let my_addr: Address = state
        .my_address
        .parse()
        .context("parsing my address")?;

    match contract.releasePayment(task_id_u256, my_addr).send().await {
        Ok(pending) => {
            let tx_hash = *pending.tx_hash();
            info!("✅ releasePayment submitted: {tx_hash:?}");
            drop(pending);
        }
        Err(e) => {
            warn!("releasePayment failed (task may not exist on-chain in dev): {e}");
        }
    }

    Ok(())
}

// ── Publisher forwarding ──────────────────────────────────────────────────────

async fn forward_to_publisher(
    state: &AppState,
    task_id: u64,
    title: &str,
    raw_html: &str,
) -> Result<()> {
    use shared::registry::get_agent;
    use alloy::primitives::Address;

    // Look up publisher's WS endpoint from on-chain registry
    let publisher_addr: Address = state
        .publisher_address
        .parse()
        .context("parsing publisher address")?;

    let publisher_endpoint = if publisher_addr == Address::ZERO {
        // Dev fallback
        warn!("PUBLISHER_ADDRESS not set, using localhost:8082");
        "ws://127.0.0.1:8082".to_string()
    } else {
        match get_agent(&state.rpc_url, &state.registry_address, publisher_addr).await {
            Ok(info) => info.endpoint,
            Err(e) => {
                warn!("Could not fetch publisher from registry ({e}), using localhost:8082");
                "ws://127.0.0.1:8082".to_string()
            }
        }
    };

    info!("📤 Forwarding title to publisher @ {publisher_endpoint}");

    let signer = signer_from_hex(&state.private_key)?;

    // Extract source URL from the task payload context (stored in raw_html header comment or env)
    let source_url = std::env::var("TARGET_URL").unwrap_or_else(|_| "unknown".to_string());

    let notification_payload = json!({
        "task_id": task_id,
        "title": title,
        "url": source_url,
    });

    let mut msg = X402Message::new_unsigned(
        state.my_address.clone(),
        state.publisher_address.clone(),
        X402MessageType::Notification,
        notification_payload,
    );

    sign_message(&mut msg, &signer).await?;

    send_to_agent(&publisher_endpoint, &msg).await?;

    info!("✅ Title forwarded to publisher");
    Ok(())
}

// ── HTML title extraction ─────────────────────────────────────────────────────

/// Extract the content of the first `<title>…</title>` tag from HTML.
/// Simple regex-free parser — good enough for MVP, no extra deps.
fn extract_title(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let start = lower.find("<title")?;
    let open_end = lower[start..].find('>')? + start + 1;
    let close = lower[open_end..].find("</title>")?;
    let raw = &html[open_end..open_end + close];
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title_simple() {
        let html = "<html><head><title>Hello World</title></head></html>";
        assert_eq!(extract_title(html), Some("Hello World".to_string()));
    }

    #[test]
    fn test_extract_title_with_attrs() {
        let html = r#"<TITLE lang="en">My Page</TITLE>"#;
        assert_eq!(extract_title(html), Some("My Page".to_string()));
    }

    #[test]
    fn test_extract_title_none() {
        let html = "<html><body>no title here</body></html>";
        assert_eq!(extract_title(html), None);
    }

    #[test]
    fn test_extract_title_whitespace() {
        let html = "<title>  Trimmed  </title>";
        assert_eq!(extract_title(html), Some("Trimmed".to_string()));
    }
}

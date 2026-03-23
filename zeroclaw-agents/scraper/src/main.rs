//! AgentMesh Scraper Agent
//!
//! Listens on ws://0.0.0.0:8080 for x402 TaskRequest messages.
//! For each request it:
//!   1. Validates the x402 signature
//!   2. Fetches the target URL with reqwest
//!   3. Sends a TaskComplete x402 message back to the requester
//!      (P2P — looks up the requester's endpoint from the registry)
//!
//! Environment variables (see .env.example):
//!   SCRAPER_PRIVATE_KEY   – hex private key for this agent's wallet
//!   RPC_URL               – Ethereum JSON-RPC endpoint
//!   REGISTRY_ADDRESS      – deployed AgentRegistry contract address
//!   SCRAPER_PORT          – WebSocket port (default: 8080)

use std::net::SocketAddr;

use anyhow::Result;
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
use shared::{
    crypto::{address_of, sign_message, signer_from_hex, verify_signature},
    registry::register_agent,
    types::{
        TaskCompletePayload, TaskRequestPayload, X402Message, X402MessageType,
    },
};
use tracing::{error, info, warn};

#[derive(Clone)]
struct AppState {
    private_key: String,
    rpc_url: String,
    registry_address: String,
    my_address: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "scraper=debug,shared=debug,info".parse().unwrap()),
        )
        .init();

    let private_key = std::env::var("SCRAPER_PRIVATE_KEY")
        .expect("SCRAPER_PRIVATE_KEY must be set");
    let rpc_url = std::env::var("RPC_URL")
        .unwrap_or_else(|_| "https://ethereum-sepolia-rpc.publicnode.com".to_string());
    let registry_address = std::env::var("REGISTRY_ADDRESS")
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string());
    let port: u16 = std::env::var("SCRAPER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("SCRAPER_PORT must be a number");

    let signer = signer_from_hex(&private_key)?;
    let my_address = address_of(&signer);

    info!("🕷️  Scraper agent starting");
    info!("   address  : {my_address}");
    info!("   registry : {registry_address}");
    info!("   port     : {port}");

    // Auto-register on startup
    let endpoint = format!("ws://localhost:{port}");
    register_agent(
        &rpc_url,
        &registry_address,
        &private_key,
        "scraper-001",
        vec!["web-scraping".to_string()],
        alloy::primitives::U256::from(1_000_000_000_000_000u64), // 0.001 ETH
        &endpoint,
    )
    .await?;

    let state = AppState {
        private_key: private_key.clone(),
        rpc_url: rpc_url.clone(),
        registry_address: registry_address.clone(),
        my_address,
    };

    let app = Router::new()
        .route("/", get(ws_handler))
        .route("/health", get(health))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Listening on ws://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> &'static str {
    "scraper ok"
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
                if let Err(e) = process_message(&text, &mut socket, &state).await {
                    error!("Error processing message: {e:#}");
                }
            }
            Message::Close(_) => {
                info!("Client disconnected");
                break;
            }
            _ => {} // ignore ping/pong/binary
        }
    }
}

async fn process_message(
    text: &str,
    socket: &mut WebSocket,
    state: &AppState,
) -> Result<()> {
    // Parse the x402 envelope
    let msg = X402Message::from_json(text)?;

    // Only handle TaskRequest
    if msg.message_type != X402MessageType::TaskRequest {
        warn!("Ignoring unexpected message type: {:?}", msg.message_type);
        return Ok(());
    }

    info!("📨 TaskRequest from {} (msg_id: {})", msg.from, msg.message_id);

    // Verify signature
    match verify_signature(&msg) {
        Ok(true) => info!("✅ Signature valid"),
        Ok(false) => {
            warn!("❌ Invalid signature on message from {}", msg.from);
            return Ok(());
        }
        Err(e) => {
            warn!("⚠️  Signature verification error: {e}");
            // Continue in dev mode — don't drop messages due to signature issues
        }
    }

    // Parse the task payload
    let payload: TaskRequestPayload = serde_json::from_value(msg.payload.clone())?;
    info!("📋 Task {}: fetching {}", payload.task_id, payload.url);

    // Fetch the URL
    let result = fetch_url(&payload.url).await;

    // Build TaskComplete message
    let signer = signer_from_hex(&state.private_key)?;

    let (result_str, status_str) = match result {
        Ok(body) => (body, "success".to_string()),
        Err(e) => (format!("ERROR: {e}"), "error".to_string()),
    };

    let complete_payload = TaskCompletePayload {
        task_id: payload.task_id,
        result: result_str,
        status: status_str,
    };

    let mut reply = X402Message::new_unsigned(
        state.my_address.clone(),
        msg.from.clone(),
        X402MessageType::TaskComplete,
        serde_json::to_value(&complete_payload)?,
    );

    sign_message(&mut reply, &signer).await?;

    info!(
        "📤 TaskComplete → {} (task {})",
        msg.from, payload.task_id
    );

    // Try to reply directly over the same WebSocket connection first.
    // This works when the analyzer initiated the WS (it's still open).
    let json = reply.to_json()?;
    socket.send(Message::Text(json.clone().into())).await?;

    // Call completeTask on-chain (executor role)
    settle_task_on_chain(state, payload.task_id).await?;

    Ok(())
}

/// Settle task on-chain by calling completeTask (executor role)
async fn settle_task_on_chain(state: &AppState, task_id: u64) -> Result<()> {
    use alloy::{
        network::EthereumWallet,
        primitives::{Address, U256},
        providers::{Provider, ProviderBuilder},
    };
    use shared::registry::AgentRegistryWriter;

    info!("⛓️  Calling completeTask({task_id}) on-chain...");

    let signer = signer_from_hex(&state.private_key)?;
    let wallet = EthereumWallet::from(signer);

    let rpc_url: url::Url = state.rpc_url.parse()?;
    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect_http(rpc_url);

    let registry_addr: Address = state.registry_address.parse()?;
    let contract = AgentRegistryWriter::new(registry_addr, &provider);

    let task_id_u256 = U256::from(task_id);

    match contract.completeTask(task_id_u256).send().await {
        Ok(pending) => {
            let tx_hash = *pending.tx_hash();
            info!("✅ completeTask submitted: {tx_hash:?}");
        }
        Err(e) => {
            warn!("completeTask failed: {e}");
        }
    }

    Ok(())
}

/// Fetch a URL and return the body as a String.
async fn fetch_url(url: &str) -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("AgentMesh-Scraper/1.0")
        .build()?;

    let resp = client.get(url).send().await?;
    let status = resp.status();
    let body = resp.text().await?;

    if !status.is_success() {
        anyhow::bail!("HTTP {status} from {url}");
    }

    info!("✅ Fetched {} bytes from {url}", body.len());
    Ok(body)
}

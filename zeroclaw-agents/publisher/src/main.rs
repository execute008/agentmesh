//! AgentMesh Publisher Agent
//!
//! Listens on ws://0.0.0.0:8082 for x402 Notification messages.
//! Prints the `title` field from the payload to stdout — the final
//! output stage of the scraper → analyzer → publisher pipeline.
//!
//! Environment variables:
//!   PUBLISHER_PRIVATE_KEY  – hex private key (for address derivation + future signing)
//!   PUBLISHER_PORT         – WebSocket port (default: 8082)

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
    crypto::{address_of, signer_from_hex, verify_signature},
    types::{X402Message, X402MessageType},
};
use tracing::{error, info, warn};

#[derive(Clone)]
struct AppState {
    my_address: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "publisher=debug,shared=debug,info".parse().unwrap()),
        )
        .init();

    let private_key = std::env::var("PUBLISHER_PRIVATE_KEY")
        .expect("PUBLISHER_PRIVATE_KEY must be set");
    let port: u16 = std::env::var("PUBLISHER_PORT")
        .unwrap_or_else(|_| "8082".to_string())
        .parse()
        .expect("PUBLISHER_PORT must be a number");

    let signer = signer_from_hex(&private_key)?;
    let my_address = address_of(&signer);

    info!("📢 Publisher agent starting");
    info!("   address : {my_address}");
    info!("   port    : {port}");

    let state = AppState { my_address };

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
    "publisher ok"
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, _state: AppState) {
    while let Some(Ok(msg)) = socket.next().await {
        match msg {
            Message::Text(text) => {
                if let Err(e) = process_message(&text).await {
                    error!("Error processing message: {e:#}");
                }
            }
            Message::Close(_) => {
                info!("Sender disconnected");
                break;
            }
            _ => {}
        }
    }
}

async fn process_message(text: &str) -> Result<()> {
    let msg = X402Message::from_json(text)?;

    info!(
        "📨 {:?} from {} (msg_id: {})",
        msg.message_type, msg.from, msg.message_id
    );

    // Verify signature (warn but continue in dev)
    match verify_signature(&msg) {
        Ok(true) => info!("✅ Signature valid"),
        Ok(false) => warn!("❌ Invalid signature from {}", msg.from),
        Err(e) => warn!("⚠️  Signature check skipped: {e}"),
    }

    // Handle Notification — the expected type from analyzer
    match msg.message_type {
        X402MessageType::Notification => {
            let title = msg
                .payload
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("<no title>");

            let source_url = msg
                .payload
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            let task_id = msg
                .payload
                .get("task_id")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            // ── THE MONEY LINE ──────────────────────────────────────────
            println!("📰 [PUBLISHED] Task {task_id} | {source_url}\n   Title: {title}");
            // ───────────────────────────────────────────────────────────

            info!("Published title: {title}");
        }
        other => {
            warn!("Publisher received unexpected message type: {other:?}");
        }
    }

    Ok(())
}

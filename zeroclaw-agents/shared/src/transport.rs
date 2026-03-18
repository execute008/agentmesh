//! P2P WebSocket transport for x402 messages.
//!
//! No central relay. Endpoints are discovered from the on-chain AgentRegistry.
//! `send_to_agent` opens a direct WS connection to the target, sends the
//! signed message, and closes the connection.

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, info};

use crate::types::X402Message;

/// Open a direct WebSocket connection to `ws_endpoint`, send `msg`, then close.
///
/// The endpoint is the `wsEndpoint` field from the AgentRegistry (e.g. "ws://127.0.0.1:8080").
/// This function does **not** wait for a reply — fire-and-forget P2P send.
///
/// # Arguments
/// * `ws_endpoint` – target agent's WebSocket URL (from on-chain registry)
/// * `msg`         – signed x402 message to transmit
pub async fn send_to_agent(ws_endpoint: &str, msg: &X402Message) -> Result<()> {
    info!(
        "→ sending {:?} to {} at {}",
        msg.message_type, msg.to, ws_endpoint
    );

    let (mut ws, _) = connect_async(ws_endpoint)
        .await
        .with_context(|| format!("connecting to agent WS at {ws_endpoint}"))?;

    let json = msg.to_json().context("serializing x402 message")?;
    debug!("sending: {json}");

    ws.send(Message::Text(json.into()))
        .await
        .context("sending x402 message over WS")?;

    // Graceful close
    ws.close(None).await.ok();

    Ok(())
}

/// Send a message and wait for a single reply message (request-response pattern).
///
/// Opens a WS, sends the message, reads **one** text frame as the response,
/// then closes. Useful when the caller needs the result synchronously.
///
/// Returns the deserialized reply `X402Message`.
pub async fn send_and_receive(ws_endpoint: &str, msg: &X402Message) -> Result<X402Message> {
    info!(
        "⇄ send+recv {:?} to {} at {}",
        msg.message_type, msg.to, ws_endpoint
    );

    let (mut ws, _) = connect_async(ws_endpoint)
        .await
        .with_context(|| format!("connecting to agent WS at {ws_endpoint}"))?;

    // Send
    let json = msg.to_json().context("serializing x402 message")?;
    ws.send(Message::Text(json.into()))
        .await
        .context("sending x402 message")?;

    // Wait for reply
    while let Some(frame) = ws.next().await {
        let frame = frame.context("reading reply frame")?;
        match frame {
            Message::Text(text) => {
                let reply = X402Message::from_json(&text).context("parsing reply x402 message")?;
                ws.close(None).await.ok();
                return Ok(reply);
            }
            Message::Close(_) => break,
            _ => continue, // skip ping/pong/binary
        }
    }

    anyhow::bail!("agent closed connection without sending a reply")
}

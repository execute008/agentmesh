//! EIP-191 signing and verification for x402 messages using alloy primitives.
//!
//! We use `personal_sign` semantics:
//!   keccak256("\x19Ethereum Signed Message:\n" + len + message)
//! so signatures are compatible with standard Ethereum tooling (MetaMask, cast, etc.)

use alloy::{
    primitives::Address,
    signers::{local::PrivateKeySigner, SignerSync},
    signers::Signer,
};
use anyhow::{Context, Result};

use crate::types::X402Message;

/// Sign an x402 message in-place.
///
/// Computes the EIP-191 personal_sign of the canonical signing bytes and
/// writes the hex-encoded signature back into `msg.signature`.
///
/// # Arguments
/// * `msg`    – message to sign (signature field will be set)
/// * `signer` – alloy `PrivateKeySigner` loaded from the agent's private key
pub async fn sign_message(msg: &mut X402Message, signer: &PrivateKeySigner) -> Result<()> {
    let bytes = msg.signing_bytes();
    let sig = signer
        .sign_message(&bytes)
        .await
        .context("signing x402 message")?;
    msg.signature = format!("0x{}", hex::encode(sig.as_bytes()));
    Ok(())
}

/// Synchronous variant for use outside async contexts.
pub fn sign_message_sync(msg: &mut X402Message, signer: &PrivateKeySigner) -> Result<()> {
    let bytes = msg.signing_bytes();
    let sig = signer
        .sign_message_sync(&bytes)
        .context("signing x402 message (sync)")?;
    msg.signature = format!("0x{}", hex::encode(sig.as_bytes()));
    Ok(())
}

/// Verify that an x402 message was signed by the address in `msg.from`.
///
/// Returns `Ok(true)` if the recovered address matches the declared sender,
/// `Ok(false)` if not, and `Err` only on parse failures.
pub fn verify_signature(msg: &X402Message) -> Result<bool> {
    use alloy::primitives::Signature;

    // Decode the hex signature
    let sig_hex = msg.signature.trim_start_matches("0x");
    let sig_bytes: [u8; 65] = hex::decode(sig_hex)
        .context("decoding signature hex")?
        .try_into()
        .map_err(|_| anyhow::anyhow!("signature must be 65 bytes"))?;

    // Build alloy Signature from raw bytes
    let sig = Signature::try_from(sig_bytes.as_ref())
        .context("parsing signature bytes")?;

    // Compute the EIP-191 message hash
    let bytes = msg.signing_bytes();
    let hash = alloy::primitives::eip191_hash_message(&bytes);

    // Recover the signer using alloy's Signature::recover_address_from_prehash
    let recovered = sig
        .recover_address_from_prehash(&hash)
        .context("recovering signer from prehash")?;

    // Parse the declared sender address
    let declared: Address = msg
        .from
        .parse()
        .context("parsing msg.from as address")?;

    Ok(recovered == declared)
}

/// Parse a private key hex string (with or without 0x prefix) into an alloy signer.
pub fn signer_from_hex(private_key: &str) -> Result<PrivateKeySigner> {
    let key = private_key.trim_start_matches("0x");
    let signer: PrivateKeySigner = key.parse().context("parsing private key")?;
    Ok(signer)
}

/// Derive the checksummed Ethereum address from a signer.
pub fn address_of(signer: &PrivateKeySigner) -> String {
    format!("{:?}", signer.address())
}

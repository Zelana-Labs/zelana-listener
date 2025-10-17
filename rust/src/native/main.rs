use anyhow::{Context, Result};
use dotenvy::dotenv;
use std::env;
use std::time::Duration;

use solana_client::{pubsub_client::PubsubClient, rpc_config::RpcAccountInfoConfig};
use solana_commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::pubkey::Pubkey;


fn main() -> Result<()> {
    dotenv().ok();

    // Default to native Solana devnet WS if not provided
    let ws_url = env::var("SOLANA_DEVNET_WSS")
        .unwrap_or_else(|_| "wss://api.devnet.solana.com/".to_string());

    let target_str = env::var("TARGET").context("Missing TARGET in env (base58 pubkey)")?;
    let target: Pubkey = target_str
        .parse()
        .with_context(|| format!("Invalid TARGET pubkey: {target_str}"))?;

    println!("Connecting to devnet WS: {ws_url}");
    println!("Subscribing to account: {target_str}");

    // Subscribe to account changes (confirmed commitment)
    let (mut _sub, receiver) = PubsubClient::account_subscribe(
        &ws_url,
        &target,
        Some(RpcAccountInfoConfig {
            commitment: Some(CommitmentConfig {
                commitment: CommitmentLevel::Processed,
            }),
            encoding: None, // we only need lamports
            data_slice: None,
            min_context_slot: None,
        }),
    )
    .context("Failed to open WS subscription")?;

    // Receive loop
    loop {
        match receiver.recv_timeout(Duration::from_secs(60)) {
            Ok(update) => {
                // update is RpcResponse<RpcAccount>
                let slot = update.context.slot;
                let lamports = update.value.lamports;
                println!("🔔 RECEIVED update: slot={slot} lamports={lamports}");
            }
            Err(_timeout) => {
                // Keep the connection alive; you could log a heartbeat here if you like.
                continue;
            }
        }
    }
}

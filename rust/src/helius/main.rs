use anyhow::{Context, Result};
use dotenvy::dotenv;
use std::env;
use std::time::Duration;

use solana_client::{pubsub_client::PubsubClient, rpc_config::RpcAccountInfoConfig};
use solana_commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::pubkey::Pubkey;

fn main() -> Result<()> {
    dotenv().ok();

    // Read required env vars
    let ws_url = env::var("HELIUS_CLUSTER_WSS").context(
        "Missing HELIUS_CLUSTER_WSS in env (e.g. wss://mainnet.helius-rpc.com/?api-key=...)",
    )?;
    let target_str = env::var("TARGET").context("Missing TARGET in env (base58 pubkey)")?;
    let target: Pubkey = target_str
        .parse()
        .with_context(|| format!("Invalid TARGET pubkey: {target_str}"))?;

    println!("Connecting to WS: {}", ws_url);
    println!("Subscribing to account: {}", target_str);

    // Subscribe to account changes
    let (mut _subscription, receiver) = PubsubClient::account_subscribe(
        &ws_url,
        &target,
        Some(RpcAccountInfoConfig {
            commitment: Some(CommitmentConfig {
                commitment: CommitmentLevel::Processed,
            }),
            encoding: None,
            data_slice: None,
            min_context_slot: None,
        }),
    )
    .context("Failed to open WS subscription")?;

    // Simple receive loop
    loop {
        match receiver.recv_timeout(Duration::from_secs(60)) {
            Ok(update) => {
                let slot = update.context.slot;
                let lamports = update.value;
                println!("🔔 RECEIVED update: slot={slot} lamports={lamports:?}");
            }
            Err(_timeout) => {
                // Keep the connection alive; you could ping/log here if you want.
                continue;
            }
        }
    }
}

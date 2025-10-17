use dirs::home_dir;
use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Signer, read_keypair_file},
    transaction::Transaction,
};
use solana_system_interface::instruction; // <-- has transfer()

fn main() -> anyhow::Result<()> {
    // change to mainnet RPC if you want mainnet
    let rpc = RpcClient::new("https://api.devnet.solana.com".to_string());

    // expand "~"
    let mut path = home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home dir"))?;
    path.push(".config/solana/id.json");
    let payer = read_keypair_file(path)
        .map_err(|e| anyhow::anyhow!("Failed to read keypair file: {}", e))?;
    
    let recipient: Pubkey = "CSg4fcG4WqaVgTE33gzquXYGKAuZpikNWKQ4P4y71kke".parse()?;
    let lamports: u64 = 10_000; // 0.1 SOL

    // ✅ builder lives here in v2 (with feature = "bincode")
    let ix = instruction::transfer(&payer.pubkey(), &recipient, lamports);

    let blockhash = rpc.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let sig = rpc.send_and_confirm_transaction(&tx)?;
    println!("Sent {} lamports to {} | sig: {}", lamports, recipient, sig);
    Ok(())
}

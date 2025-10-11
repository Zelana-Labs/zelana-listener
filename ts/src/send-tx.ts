import 'dotenv/config';
import { Connection, PublicKey, SystemProgram, Transaction, Keypair, sendAndConfirmTransaction, LAMPORTS_PER_SOL } from '@solana/web3.js';
import fs from 'fs';

// Load your local Solana keypair (CLI wallet)
function loadKeypair(): Keypair {
  const path = process.env.KEYPAIR_PATH || `${process.env.HOME}/.config/solana/id.json`;
  const secret = JSON.parse(fs.readFileSync(path, 'utf-8'));
  return Keypair.fromSecretKey(Uint8Array.from(secret));
}

(async () => {
  const connection = new Connection("https://api.devnet.solana.com");
  const target = new PublicKey(process.env.TARGET!);

  const sender = loadKeypair();
  //await connection.requestAirdrop(sender.publicKey, 2 * LAMPORTS_PER_SOL);
  //await new Promise(r => setTimeout(r, 4000));

  const tx = new Transaction().add(
    SystemProgram.transfer({
      fromPubkey: sender.publicKey,
      toPubkey: target,
      lamports: 100_000, // 0.0001 SOL
    })
  );

  const start = Date.now();
  const sig = await sendAndConfirmTransaction(connection, tx, [sender]);
  console.log(JSON.stringify({ sig, start }));
})();

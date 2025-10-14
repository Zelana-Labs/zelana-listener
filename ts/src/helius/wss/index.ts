import 'dotenv/config';
import { Connection, PublicKey, clusterApiUrl } from '@solana/web3.js';

// Environment variables
const HTTP = process.env.HELIUS_CLUSTER_RPC ?? clusterApiUrl('mainnet-beta');
const WS = process.env.HELIUS_CLUSTER_WSS!;
const TARGET_STR = process.env.TARGET!;
const TARGET = new PublicKey(TARGET_STR);

// Create a persistent Solana connection using Helius WS
const connection = new Connection(HTTP, { wsEndpoint: WS, commitment: 'confirmed' });

// Track processed transactions to prevent double-handling
const seen = new Set<string>();

// Store previous lamport balance for quick diff checks
let lastLamports: number | null = null;


// Function to find and log the deposit transaction
async function handleDeposit(slot: number) {
  try {
    const sigs = await connection.getSignaturesForAddress(TARGET, { limit: 10 });
    for (const s of sigs) {
      if (seen.has(s.signature)) continue;

      const tx = await connection.getParsedTransaction(s.signature, {
        maxSupportedTransactionVersion: 0,
        commitment: 'confirmed',
      });
      if (!tx) continue;

      const idx = tx.transaction.message.accountKeys.findIndex((k: any) =>
        (k.pubkey ? k.pubkey.toBase58() : k.toBase58()) === TARGET_STR
      );
      if (idx < 0) continue;

      const pre = BigInt(tx.meta?.preBalances?.[idx] ?? 0);
      const post = BigInt(tx.meta?.postBalances?.[idx] ?? 0);
      if (post > pre) {
        const diff = post - pre;
        console.log('💰 Deposit detected!');
        console.log('Signature:', s.signature);
        console.log('Slot:', slot);
        console.log('Amount (lamports):', diff.toString());
        console.log('Sender:', tx.transaction.message.accountKeys[0].pubkey.toBase58());
        seen.add(s.signature);
      }
    }
  } catch (e) {
    console.error('handleDeposit error:', e);
  }
}

// Subscribe to account changes (fast push notifications)
connection.onAccountChange(TARGET, async (accInfo, ctx) => {
  try {
    const lamports = accInfo.lamports;
    const sig = accInfo.data
    console.log("RECEIVED", lamports)
    lastLamports = lamports;
  } catch (e) {
    console.error('onAccountChange handler error:', e);
  }
});

console.log('[WS] Listening for SOL deposits via Helius WS:', { TARGET: TARGET_STR, WS });

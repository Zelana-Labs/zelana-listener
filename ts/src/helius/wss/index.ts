import 'dotenv/config';
import { Connection, PublicKey, clusterApiUrl } from '@solana/web3.js';

// Environment variables
const HTTP = process.env.HELIUS_CLUSTER_RPC ?? clusterApiUrl('mainnet-beta');
const WS = process.env.HELIUS_CLUSTER_WSS!;
const TARGET_STR = process.env.TARGET!;
const TARGET = new PublicKey(TARGET_STR);

// Create a persistent Solana connection using Helius WS
const connection = new Connection(HTTP, { wsEndpoint: WS, commitment: 'confirmed' });

// Store previous lamport balance for quick diff checks
let lastLamports: number | null = null;

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

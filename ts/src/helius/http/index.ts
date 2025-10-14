import 'dotenv/config';

const TARGET = process.env.TARGET;
const HELIUS_API_KEY = process.env.HELIUS_API_KEY;
const POLL_INTERVAL_MS = Number(process.env.POLL_INTERVAL_MS) || 2000;

if (!TARGET) {
  throw new Error('TARGET environment variable is required');
}

if (!HELIUS_API_KEY) {
  throw new Error('HELIUS_API_KEY environment variable is required');
}

console.log(`🚀 Helius HTTP polling listener starting...`);
console.log(`📡 Monitoring address: ${TARGET}`);
console.log(`⏱️  Poll interval: ${POLL_INTERVAL_MS}ms`);

let lastSeenTimestamp = 0;

async function pollTransactions() {
  try {
    const url = `https://api-devnet.helius.xyz/v0/addresses/${TARGET}/transactions?api-key=${HELIUS_API_KEY}`;
    
    const response = await fetch(url);
    if (!response.ok) {
      console.error(`❌ API error: ${response.status} ${response.statusText}`);
      return;
    }

    const transactions = await response.json();

    // Process only transactions newer than last seen timestamp
    for (const tx of transactions) {
      const signature = tx.signature;
      const timestamp = tx.timestamp;
      
      if (timestamp && timestamp > lastSeenTimestamp) {
        console.log(`RECEIVED ${signature}`);
        lastSeenTimestamp = timestamp;
      }
    }
  } catch (error) {
    console.error('❌ Error polling transactions:', error);
  }
}

console.log(`✅ Listener ready, starting to poll...\n`);

// Poll immediately, then at intervals
pollTransactions();
setInterval(pollTransactions, POLL_INTERVAL_MS);
import 'dotenv/config';
import express from 'express';
import bodyParser from 'body-parser';

const app = express();
app.use(bodyParser.json({ limit: '2mb' }));

const PORT = process.env.PORT || 3000;
const TARGET = process.env.TARGET;

if (!TARGET) {
  throw new Error('TARGET environment variable is required');
}

// Replace with your own persistent store
const processedSigs = new Set<string>();

async function creditL2Account(l2Address: string, amount: number, metadata: any) {
  // TODO: call your L2 system to credit `l2Address` with `amount`.
  console.log(`CREDIT L2 ${l2Address} += ${amount}`, metadata);
}

interface HeliusTransfer {
  destination?: string;
  to?: string;
  account?: string;
  amount?: number;
  lamports?: number;
  memo?: string;
}

interface HeliusTokenBalance {
  account?: string;
  owner?: string;
  uiTokenAmount?: {
    uiAmount?: number;
    amount?: string;
  };
}

interface HeliusEvent {
  signature?: string;
  tx?: {
    signature?: string;
  };
  transfers?: HeliusTransfer[];
  parsed?: {
    transfers?: HeliusTransfer[];
  };
  logs?: {
    transfers?: HeliusTransfer[];
  };
  tokenBalanceChanges?: HeliusTokenBalance[];
  token_balances?: HeliusTokenBalance[];
  meta?: {
    postTokenBalances?: HeliusTokenBalance[];
  };
  metadata?: {
    userL2?: string;
  };
}

app.post('/helius-webhook', async (req, res) => {
  try {
    const body = req.body;

    // If Helius sends an array of events
    const events: HeliusEvent[] = Array.isArray(body) ? body : [body];

    for (const ev of events) {
      try {
        // Extract signature
        const sig = ev.signature || ev.tx?.signature;
        if (!sig) {
          console.log('Event missing signature, skipping');
          continue;
        }

        // Skip if already processed
        if (processedSigs.has(sig)) {
          console.log(`Signature ${sig} already processed, skipping`);
          continue;
        }

        console.log(`Processing signature: ${sig}`);

        // Try to extract transfers touching the monitored TARGET pubkey
        const transfers = ev.transfers || ev.parsed?.transfers || ev.logs?.transfers || [];

        for (const t of transfers) {
          const dest = t.destination || t.to || t.account;
          if (dest === TARGET) {
            const amount = Number(t.amount || t.lamports || 0);
            const l2Address = ev.metadata?.userL2 || t.memo || 'unknown';
            
            console.log(`✅ Transfer detected to ${TARGET}: ${amount} lamports`);
            await creditL2Account(l2Address, amount, { sig, raw: t });
          }
        }

        // Fallback: inspect top-level tokenBalance changes if present
        const tokenBalances = 
          ev.tokenBalanceChanges || 
          ev.token_balances || 
          ev.meta?.postTokenBalances || 
          [];

        for (const tb of tokenBalances) {
          if (tb.account === TARGET || tb.owner === TARGET) {
            const amount = tb.uiTokenAmount?.uiAmount || Number(tb.uiTokenAmount?.amount || 0);
            const owner = tb.owner || 'unknown';
            
            console.log(`✅ Token balance change detected for ${TARGET}: ${amount}`);
            await creditL2Account(owner, amount, { sig, token: tb });
          }
        }

        processedSigs.add(sig);
      } catch (err) {
        console.error('Error handling event:', err);
      }
    }

    res.status(200).json({ ok: true });
  } catch (err) {
    console.error('Error processing webhook:', err);
    res.status(500).json({ ok: false, error: 'Internal server error' });
  }
});

app.get('/health', (req, res) => {
  res.status(200).json({ 
    ok: true, 
    target: TARGET,
    processedCount: processedSigs.size 
  });
});

app.listen(PORT, () => {
  console.log(`Helius webhook listener running on ${PORT}`);
  console.log(`Monitoring address: ${TARGET}`);
});
// Usage: set PORT, TARGET_ADDRESS, implement creditL2Account
import express = require('express');
import bodyParser = require('body-parser');


const app = express();
app.use(bodyParser.json({ limit: '2mb' }));


const PORT = process.env.PORT || 3000;
const TARGET = process.env.TARGET_ADDRESS!; // monitored L1 account pubkey


// Replace with your own persistent store
const processedSigs = new Set<string>();


async function creditL2Account(l2Address: string, amount: number, metadata: any) {
    // TODO: call your L2 system to credit `l2Address` with `amount`.
    console.log(`CREDIT L2 ${l2Address} += ${amount}`, metadata);
}


app.post('/helius-webhook', async (req, res) => {
    // Helius can deliver decoded transactions or transfer events depending on webhook config.
    // Example payload shape depends on your Helius webhook options. We'll handle common cases:


    const body = req.body;


    // If Helius sends an array of events
    const events = Array.isArray(body) ? body : [body];


    for (const ev of events) {
        try {
            // Example: ev.signature, ev.type, ev.parsed
            const sig = ev.signature || ev.tx?.signature;
            if (!sig) continue;
            if (processedSigs.has(sig)) continue;


            // Try to extract transfers touching the monitored TARGET pubkey.
            // Helius often returns `transfer` events with fields: source, destination, amount, mint (for SPL)
            const transfers = ev.transfers || ev.parsed?.transfers || ev.logs?.transfers || [];


            for (const t of transfers) {
                const dest = t.destination || t.to || t.account;
                if (dest === TARGET) {
                    const amount = Number(t.amount || t.lamports || 0);
                    const l2Address = ev.metadata?.userL2 || t.memo || null; // your mapping logic
                    await creditL2Account(l2Address ?? 'unknown', amount, { sig, raw: t });
                }
            }


            // Fallback: inspect top-level tokenBalance changes if present
            const tokenBalances = ev.tokenBalanceChanges || ev.token_balances || ev.meta?.postTokenBalances;
            if (tokenBalances) {
                for (const tb of tokenBalances) {
                    if (tb.account === TARGET || tb.owner === TARGET) {
                        // tb.uiTokenAmount?.uiAmount is a decimal amount for SPL tokens
                        const amount = tb.uiTokenAmount?.uiAmount || tb.uiTokenAmount?.amount;
                        await creditL2Account(tb.owner || 'unknown', Number(amount), { sig, token: tb });
                    }
                }
            }


            processedSigs.add(sig);
        } catch (err) {
            console.error('error handling event', err);
        }
    }


    res.status(200).send({ ok: true });
});


app.listen(PORT, () => console.log(`Helius webhook listener running on ${PORT}`));
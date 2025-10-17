import {
    Connection,
    PublicKey,
    AccountInfo,
    Context
} from '@solana/web3.js';

// Configuration
const RPC_ENDPOINT = 'https://api.devnet.solana.com'; // Devnet endpoint
const ACCOUNT_TO_WATCH = 'CSg4fcG4WqaVgTE33gzquXYGKAuZpikNWKQ4P4y71kke'; // Replace with the account you want to monitor

class SolanaAccountListener {
    private connection: Connection;
    private subscriptionId: number | null = null;
    private accountPublicKey: PublicKey;

    constructor(rpcEndpoint: string, accountAddress: string) {
        this.connection = new Connection(rpcEndpoint, {
            commitment: 'confirmed',
            wsEndpoint: rpcEndpoint.replace('https://', 'wss://'),
        });
        this.accountPublicKey = new PublicKey(accountAddress);
    }

    /**
     * Start listening to account changes
     */
    async startListening(): Promise<void> {
        try {
            console.log(`Listening to: ${this.accountPublicKey.toString()}\n`);

            // Subscribe to account changes
            this.subscriptionId = this.connection.onAccountChange(
                this.accountPublicKey,
                async (accountInfo: AccountInfo<Buffer>, context: Context) => {
                    try {
                        const signatures = await this.connection.getSignaturesForAddress(
                            this.accountPublicKey,
                            { limit: 1 },
                        );
                        const sig = signatures[0]?.signature ?? 'N/A';
                        console.log(`RECEIVED: ${sig}`);
                    } catch (e) {
                        console.error('No signature found:', e);
                    }
                },
                {
                    commitment: 'processed', // of 'finalized'
                },
            );

        } catch (error) {
            console.error('Error:', error);
            throw error;
        }
    }

    async stopListening(): Promise<void> {
        if (this.subscriptionId !== null) {
            await this.connection.removeAccountChangeListener(this.subscriptionId);
            this.subscriptionId = null;
        }
    }
}

// Main execution
async function main() {
    const listener = new SolanaAccountListener(RPC_ENDPOINT, ACCOUNT_TO_WATCH);

    // Start listening
    await listener.startListening();

    // Handle graceful shutdown
    process.on('SIGINT', async () => {
        await listener.stopListening();
        process.exit(0);
    });

    // Keep the process running
    await new Promise(() => { });
}

// Run the listener
main().catch((error) => {
    console.error('Fatal error:', error);
    process.exit(1);
});
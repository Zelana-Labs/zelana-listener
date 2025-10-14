# L1 account listener
On-chain account listener

## Benchmark Comparison

| Implementation | Method | Detection Time (ms) | Notes |
|----------------|--------|--------------------:|-------|
| **TypeScript** |
| TS Helius HTTP | HTTP Polling | - | |
| TS Helius WSS | WebSocket | - | |
| TS Native | Native RPC | - | |
| **Rust** |
| Rust Helius HTTP | HTTP Polling | - | |
| Rust Helius WSS | WebSocket | - | |
| Rust Native HTTP | Native RPC (HTTP) | - | |
| Rust Native WSS | Native RPC (WSS) | - | |

## Summary

- **Fastest**: TBD
- **Most Reliable**: TBD
- **Best for Production**: TBD

---

### Test Configuration

- **Network**: Devnet
- **Target Address**: `CSg4fcG4WqaVgTE33gzquXYGKAuZpikNWKQ4P4y71kke`
- **Poll Interval**: 2000ms (HTTP methods)
- **Measurement**: Time from transaction sent to detection logged

### Notes

- All times measured in milliseconds with microsecond precision
- Lower numbers indicate faster detection
- WebSocket methods should theoretically be faster than polling
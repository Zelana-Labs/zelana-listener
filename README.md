# L1 account listener
On-chain account listener

## Benchmark Comparison

| Implementation | Method | Detection Time (ms) | Notes |
|----------------|--------|--------------------:|-------|
| **TypeScript** |
| TS Helius HTTP | HTTP Polling | - 19532| HTTP slow as shit |
| TS Helius WSS | WebSocket | - 5220| |
| TS Native WSS | Native | - 4752| |
| **Rust** |
| Rust Helius | WebSocket | - 5589| |
| Rust Native | Native RPC (WSS) | - 5424| |

## Summary

- **Fastest**: TS Native
- **Most Reliable**: HTTP
- **Best for Production**: Rust Native

---

### Test Configuration

- **Network**: Devnet
- **Target Address**: `CSg4fcG4WqaVgTE33gzquXYGKAuZpikNWKQ4P4y71kke`
- **Measurement**: Time from transaction sent to detection 

### Run test
Run the compiling commands first:
```
cd rust
cargo build --bin helius &&
cargo build --bin native 
```

And then run the benchmark program:
```
cargo run --bin bench
```

### Notes

- All times measured in milliseconds with microsecond precision
- Lower numbers indicate faster detection
- WebSocket methods should theoretically be faster than polling
- The benchmarks are not really accurate, they can always defer by the network and are also based out of stdout
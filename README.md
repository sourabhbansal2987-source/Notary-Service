# Stellar Notary Contract

A Soroban smart contract on the Stellar network that provides on-chain document notarization. It records a document's SHA-256 hash alongside the notary's address and a blockchain timestamp — creating tamper-proof, permanent proof that a specific document existed in a specific state at a specific time.

No document content is ever stored on-chain. Only the hash.

---

## What It Does

When you call `notarize`, the contract:

1. Verifies the notary signed the transaction (no impersonation possible)
2. Checks the hash has not been notarized before (no overwriting)
3. Captures the ledger timestamp at that exact moment
4. Writes the record to persistent storage permanently

When you call `verify`, the contract returns the full notarization record — notary address, timestamp, and metadata — or nothing if the document was never registered.

This gives you a simple, auditable answer to: *"Did this exact document exist, and who certified it, and when?"*

---

## Features

**Immutable records** — Once a document hash is notarized, the record cannot be updated or deleted. The contract will reject any second attempt to notarize the same hash.

**Notary authentication** — The notary address must sign the transaction. The contract calls `require_auth()`, so Stellar's native auth system enforces this. You cannot notarize on someone else's behalf without their key.

**Ledger timestamp** — The timestamp is taken from `env.ledger().timestamp()`, which is the Stellar network's consensus time. It is not user-supplied and cannot be faked.

**Hash validation** — The contract enforces that the submitted hash is exactly 32 bytes (SHA-256). Submitting a truncated or wrong-format hash is rejected before any storage write.

**On-chain verifiability** — Anyone can call `verify` or `is_notarized` without signing anything. Verification is permissionless and free to query.

**No document exposure** — The contract only stores the hash, never the document itself. Sensitive content stays off-chain.

---

## Contract Interface

```rust
// Notarize a document. Notary must sign the transaction.
fn notarize(env, notary: Address, doc_hash: Bytes, metadata: String) -> NotarizedDocument

// Retrieve the full notarization record for a hash.
fn verify(env, doc_hash: Bytes) -> Option<NotarizedDocument>

// Check if a hash has been notarized (returns bool).
fn is_notarized(env, doc_hash: Bytes) -> bool
```

The `NotarizedDocument` struct returned contains:

| Field       | Type    | Description                              |
|-------------|---------|------------------------------------------|
| `doc_hash`  | Bytes   | The 32-byte SHA-256 hash                 |
| `notary`    | Address | Stellar address of the notary            |
| `timestamp` | u64     | Ledger timestamp (Unix seconds)          |
| `metadata`  | String  | Optional label, e.g. "Lease Agreement"   |

---

## Project Structure

```
notary-contract/
├── Cargo.toml
└── src/
    └── lib.rs      # Contract logic + unit tests
```

---

## Build & Test

**Prerequisites:** Rust with `wasm32-unknown-unknown` target and the Soroban CLI.

```bash
# Install target
rustup target add wasm32-unknown-unknown

# Install Soroban CLI
cargo install --locked soroban-cli

# Run tests
cargo test --features testutils

# Build optimized WASM
cargo build --release --target wasm32-unknown-unknown
```

The compiled `.wasm` will be at:
```
target/wasm32-unknown-unknown/release/notary_contract.wasm
```

---

## Deploy & Invoke

```bash
# Deploy to Stellar testnet
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/notary_contract.wasm \
  --network testnet

# Notarize a document (hash must be hex-encoded 32 bytes)
soroban contract invoke \
  --network testnet \
  -- notarize \
  --doc_hash "$(sha256sum document.pdf | awk '{print $1}')" \
  --metadata "My Agreement v1"

# Verify a document
soroban contract invoke \
  --network testnet \
  -- verify \
  --doc_hash "HASH_HERE"
```

---

## Design Decisions Worth Knowing

**Why persistent storage and not temporary?** Temporary storage on Soroban can expire. A notarization that silently disappears after a few months is useless. Persistent storage requires rent to be paid to stay active — consider building a fee mechanism or archival strategy for production use.

**Why no revocation?** The value of notarization is that it cannot be undone. A revocation mechanism would undermine the trust model. If you need revocation, handle it as a separate audit log pattern — not by deleting the original record.

**Why no access control on verification?** Verification is intentionally public. The entire point is that anyone — including courts, counterparties, or auditors — can independently confirm a document's notarization without needing your permission.

---

## License

MIT

wallet address: GACBIUGPGMD7B7ZVFUXE6D5K7YHJ64SITDHB6XTREQE3OGPVCOHBOQCI

contract address: CAVYC7N46ABLHLPDBQ6ZC6JY5HHKIHKPPXPW5GWNQFBWCW7GB6LZ7ZPK

https://stellar.expert/explorer/testnet/contract/CAVYC7N46ABLHLPDBQ6ZC6JY5HHKIHKPPXPW5GWNQFBWCW7GB6LZ7ZPK

<img width="1911" height="1079" alt="image" src="https://github.com/user-attachments/assets/5d2d9de5-a392-456f-9845-157646b3e79e" />

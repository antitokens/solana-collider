# Antitoken Collider

Before beginning, you'll need to install the following core dependencies for a complete runtime environment:

| Tool     | Mac       | Linux     |
|----------|-----------|-----------|
| `rustc`  | `1.83.0`  | `1.75.0`  |
| `solana` | `1.18.26` | `1.18.26` |
| `anchor` | `0.29.0`  | `0.29.0`  |

Please follow these instructions to set up your environment: [`https://solana.com/docs/intro/installation`](https://solana.com/docs/intro/installation)

## Cargo/Rust mode:

This is deep rust mode for unit testing.

> You'll need a local validator running for this. In a separate terminal, simply issue the command `solana-test-validator`

### Build

```
cargo build-bpf
```

### Test

```
cargo test-sbf
```

## Anchor/TS mode:

This is integration mode for interface testing in TypeScript.

> You don't need to run a local validator for this. Anchor does that under the hood.

### Install

```
yarn install
```

### Build

```
anchor build
```

> You may need to change the `version = 4` to `version = 3` in `Cargo.lock` manually for this to work

### Test

```
anchor test
```

## Presets:

```json
"build-anchor-apple": "anchor build && sed -i '' 's/version = 4/version = 3/' Cargo.lock",
"build-anchor-full-apple": "RUST_LOG=trace anchor build && sed -i '' 's/version = 4/version = 3/' Cargo.lock",
"build-cargo-apple": "cargo build-bpf && sed -i '' 's/version = 4/version = 3/' Cargo.lock",
"build-cargo-full-apple": "RUST_LOG=trace cargo build-bpf && sed -i '' 's/version = 4/version = 3/' Cargo.lock",
"build-anchor-linux": "anchor build && sed -i 's/version = 4/version = 3/' Cargo.lock",
"build-anchor-full-linux": "RUST_LOG=trace anchor build && sed -i 's/version = 4/version = 3/' Cargo.lock",
"build-cargo-linux": "cargo build-bpf && sed -i 's/version = 4/version = 3/' Cargo.lock",
"build-cargo-full-linux": "RUST_LOG=trace cargo build-bpf && sed -i :'s/version = 4/version = 3/' Cargo.lock",
"test-anchor": "anchor test",
"test-anchor-full": "RUST_LOG=trace anchor test",
"test-anchor-fast": "RUST_LOG=trace anchor test --skip-build --skip-deploy",
"test-cargo": "RUST_LOG=error cargo test-sbf",
"test-cargo-log": "RUST_LOG=info cargo test-sbf -- --nocapture",
"test-cargo-debug": "RUST_LOG=debug cargo test-sbf",
"test-cargo-full": "RUST_LOG=trace cargo test-sbf",
```
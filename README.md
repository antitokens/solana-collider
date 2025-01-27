# Antitoken Collider

Before beginning, you'll need to install the following core dependencies for a complete runtime environment:

- `rustc`

- `solana`

- `anchor` (preferably with `avm`)

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
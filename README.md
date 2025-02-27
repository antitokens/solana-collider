# Antitoken Collider

Before beginning, you'll need to install the following core dependencies for a complete runtime environment:

| Tool     | Mac       | Linux     |
| -------- | --------- | --------- |
| `rustc`  | `1.83.0`  | `1.83.0`  |
| `solana` | `1.18.26` | `1.18.26` |
| `anchor` | `0.29.0`  | `0.29.0`  |

Try to install the latest versions of the tools using a script I've forked from [`https://github.com/solana-developers/solana-install/blob/main/install.sh`](https://github.com/solana-developers/solana-install/blob/main/install.sh):

```
curl --proto '=https' --tlsv1.2 -sSfL https://raw.githubusercontent.com/antitokens/solana-collider/main/install.sh | bash
```

Alternatively, follow these instructions to set up your environment: [`https://solana.com/docs/intro/installation`](https://solana.com/docs/intro/installation)

## Install

```
yarn install
```

## Cargo/Rust mode:

This is deep rust mode for unit testing.

> You don't need to run a local validator for this. You don't need anything in the `.env` file.

### Build

```
yarn build-cargo
```

If the program builds successfully, you can run the following command for unit testing:

### Test

```
yarn test-cargo
```

All tests should pass assuming the program builds successfully.

## Anchor/TS mode:

This is integration mode for interface testing in TypeScript.

> You don't need to run a local validator for this. Anchor does that under the hood. You don't need anything in the `.env` file.

### Build

```
yarn build-anchor
```

If the program builds successfully, you can run the following command for testing:

### Test

```
yarn test-anchor
```

## `LOCALNET`

We will deploy explicitly to the localnet for testing.

### 1. Start a local validator

You'll need a local validator running for this. In a separate terminal, simply issue the command:

```
solana-test-validator -r solana-test-validator --deactivate-feature EenyoWx9UMXYKpR8mW5Jmfmy2fRjzUtM7NduYMY8bx33
```

> This will disable the feature `EenyoWx9UMXYKpR8mW5Jmfmy2fRjzUtM7NduYMY8bx33` which makes sure that oversized instructions - larger than the `4096` byte stack limit, are not allowed and will lead to an explicit error.

Note down the validator's RPC URL (`SOLANA_API`); this should typically be `http://localhost:8899`.

### 2. Create necessary accounts

```
yarn setup
```

Note down the addresses of the necessary accounts:

- `ANTI_TOKEN_MINT`: The mint address of the anti-token.
- `PRO_TOKEN_MINT`: The mint address of the pro-token.
- `VAULT`: The address of the vault.

In the next step, we will add them to the `.env` file.

### 3. `.env`

Make sure you have the necessary fields set in your `.env` file for this step onwards.

```
SOLANA_API="http://localhost:8899"
ANTI_TOKEN_MINT=EfbqfuxKWTNXtZtDhS47JMqUhsfLCu3y7VcCX7H8QT6V
PRO_TOKEN_MINT=45U2Qhg7M2261SfKrBCMuDFSbScxRwa1QhxW39f71MjF
VAULT=BVkN9PdWJA8YYJCHdkd46Y4HUPhvSUf38qcHYgFUopBh
```
### 4. Deploy the program

```
yarn deploy-anchor
```

### 5. Initialise the program

```
yarn initialise
```

### 6. Call admin function

```
yarn admin
```

### 7. Create a new prediction

```
yarn create-new
```

❌ You should see an access error:

```
'Program 3zKqVU2RiWXPe3bvTjQ869UF6qng2LoGBKEFmUqh8BzA failed: Access violation in stack frame 5 at address 0x200005bd8 of size 8'
```

## `DEVNET`

We will deploy explicitly to the devnet for testing. 

> You don't need to run a local validator.

All steps for devnet are the same as localnet, except the RPC URL. For devnet, you'll need to set the following in your `.env` file:

```
SOLANA_API="https://api.devnet.solana.com"
```

Follow the same steps as localnet, but make sure you're deploying to the devnet.

## `MAINNET`

We will deploy explicitly to the mainnet for production.

> You don't need to run a local validator.

All steps for mainnet are the same as localnet/devnet, except the RPC URL. For mainnet, you'll need to set the following in your `.env` file:

```
SOLANA_API="https://api.mainnet-beta.solana.com"
```

Follow the same steps as localnet/devnet, but make sure you're deploying to the mainnet.


---

# Presets:

> You may need to issue some commands with `sudo` if your `.config` directory is protected.

| Command                       | Script                                                                                                                                                   |
| ----------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `yarn clean-lock`          | `sh -c "if [[ $(uname) == 'Darwin' ]]; then sed -i '' 's/version = 4/version = 3/' Cargo.lock; else sed -i 's/version = 4/version = 3/' Cargo.lock; fi"` |
| `yarn build-anchor`           | `yarn clean-lock && anchor build`                                                                                                                        |
| `yarn build-anchor-full`      | `yarn clean-lock && RUST_LOG=trace anchor build`                                                                                                         |
| `yarn build-cargo`            | `yarn clean-lock && cargo build-bpf`                                                                                                                     |
| `yarn build-cargo-full`       | `yarn clean-lock && RUST_LOG=trace cargo build-bpf`                                                                                                      |
| `yarn build-verify`           | `yarn clean-lock && anchor build --verifiable`                                                                                                           |
| `yarn test-anchor`            | `anchor test`                                                                                                                                            |
| `yarn test-anchor-full`       | `RUST_LOG=trace anchor test`                                                                                                                             |
| `yarn test-anchor-fast`       | `RUST_LOG=trace anchor test --skip-build --skip-deploy`                                                                                                  |
| `yarn test-cargo`             | `RUST_LOG=error cargo test-sbf`                                                                                                                          |
| `yarn test-cargo-log`         | `RUST_LOG=info cargo test-sbf -- --nocapture`                                                                                                            |
| `yarn test-cargo-debug`       | `RUST_LOG=debug cargo test-sbf`                                                                                                                          |
| `yarn test-cargo-full`        | `RUST_LOG=trace cargo test-sbf`                                                                                                                          |
| `yarn prepare-prod`           | `node postbuild/prepare/index.js`                                                                                                                        |
| `yarn query-txt`              | `query-security-txt target/deploy/collider_beta.so`                                                                                                      |
| `yarn deploy`                 | `anchor deploy`                                                                                                                                          |
| `yarn setup`                  | `bash postbuild/initialise/setup.sh`                                                                                                                     |
| `yarn initialise`             | `node --loader ts-node/esm postbuild/initialise/index.ts`                                                                                                |
| `yarn admin`                  | `node --loader ts-node/esm postbuild/admin/index.ts`                                                                                                     |
| `yarn create-new`             | `node --loader ts-node/esm postbuild/create/index.ts`                                                                                                    |
| `yarn deposit`                | `node --loader ts-node/esm postbuild/deposit/index.ts`                                                                                                   |
| `yarn equalise`               | `node --loader ts-node/esm postbuild/equalise/index.ts`                                                                                                  |
| `yarn withdraw-bulk`          | `node --loader ts-node/esm postbuild/withdraw/bulk_withdraw/index.ts`                                                                                    |
| `yarn withdraw-single`        | `node --loader ts-node/esm postbuild/withdraw/user_withdraw/index.ts`                                                                                    |
| `yarn verify-admin`           | `node --loader ts-node/esm postbuild/admin/verifier.ts`                                                                                                  |
| `yarn verify-initialise`      | `node --loader ts-node/esm postbuild/initialise/verifier.ts`                                                                                             |
| `yarn verify-create`          | `node --loader ts-node/esm postbuild/create/verifier.ts`                                                                                                 |
| `yarn verify-deposit`         | `node --loader ts-node/esm postbuild/deposit/verifier.ts`                                                                                                |
| `yarn verify-equalise`        | `node --loader ts-node/esm postbuild/equalise/verifier.ts`                                                                                               |
| `yarn verify-withdraw-bulk`   | `node --loader ts-node/esm postbuild/withdraw/bulk_withdraw/verifier.ts`                                                                                 |
| `yarn verify-withdraw-single` | `node --loader ts-node/esm postbuild/withdraw/user_withdraw/verifier.ts`                                                       
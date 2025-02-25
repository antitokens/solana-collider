#!/bin/bash
chmod -R a+rx .config dAnti dPro

# Config
FEE_PAYER=".config/id.json"
TOKEN_NAMES=("dAnti" "dPro")
MINT_AUTHORITIES=("AN5hFEFeWJR4SrYUaSbR4R521PHkpqZcGvqPqjJGGbNi" "PRFnsn8GWkGsPwxLXqW2EwmpoXmY7AvdKg4jYs1fjSb")
RECIPIENT="Be3KKiyybpHacAYbxZBRVoF1nAdq5kyBL2kqJ7MdivVi"
VAULT="BVkN9PdWJA8YYJCHdkd46Y4HUPhvSUf38qcHYgFUopBh"
AMOUNT=1000000

# Check balances and airdrop if needed
setup_root() {
    local amount=$1
    balance=$(solana balance)
    address=$(solana address)
    if [ "$balance" == "0 SOL" ]; then
        echo "⏳ Initialising $address"
        solana airdrop $amount
    fi
}
check_and_airdrop() {
    local address=$1
    local amount=$2
    balance=$(solana balance $address)
    if [ "$balance" == "0 SOL" ]; then
        echo "⏳ Initialising $address"
        solana airdrop $amount $address
    fi
}

# Perform balance checks
setup_root 1
check_and_airdrop $(solana-keygen pubkey $FEE_PAYER) 10
for auth in "${MINT_AUTHORITIES[@]}"; do
    check_and_airdrop $auth 1
done
check_and_airdrop $VAULT 1
check_and_airdrop $RECIPIENT 1

# Process tokens
for i in "${!TOKEN_NAMES[@]}"; do
    TOKEN_NAME="${TOKEN_NAMES[$i]}"
    MINT_AUTHORITY="${MINT_AUTHORITIES[$i]}"

    # Create token & grab MINT_ADDRESS
    stdout=$(spl-token create-token --mint-authority $MINT_AUTHORITY --fee-payer $FEE_PAYER)
    MINT_ADDRESS=$(echo $stdout | awk '{print $3}')
    echo "✅ Created $TOKEN_NAME token with address: $MINT_ADDRESS"

    # Create token account to receive tokens
    spl-token create-account $MINT_ADDRESS --owner $RECIPIENT --fee-payer $FEE_PAYER

    # Mint tokens to recipient
    spl-token mint $MINT_ADDRESS $AMOUNT --mint-authority $TOKEN_NAME/$MINT_AUTHORITY.json --recipient-owner $RECIPIENT

    # Verify token airdrop
    spl-token accounts --owner $RECIPIENT
done

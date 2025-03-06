#!/bin/bash
chmod -R a+rx .config

# Core Config
SOL_ID=".config/id.json"
USER=".config/dUser/id.json"
CREATOR=".config/dCreator/id.json"
MANAGER=".config/dManager/id.json"
VAULT=".config/dVault/id.json"
TOKEN_NAMES=("dAnti" "dPro")

# Make wallets
if [ ! -f $SOL_ID ]; then
    solana-keygen new --outfile $SOL_ID
fi
if [ ! -f $MANAGER ]; then
    solana-keygen new --outfile $MANAGER
fi
if [ ! -f $CREATOR ]; then
    solana-keygen new --outfile $CREATOR
fi
if [ ! -f $USER ]; then
    solana-keygen new --outfile $USER
fi
if [ ! -f $VAULT ]; then
    solana-keygen new --outfile $VAULT
fi

# Make derived wallets
for TOKEN_NAME in "${TOKEN_NAMES[@]}"; do
    MINT_AUTHORITY=".config/$TOKEN_NAME/id.json"
    TOKEN_FILE=".config/$TOKEN_NAME/token.json"

    if [ ! -f "$MINT_AUTHORITY" ]; then
        solana-keygen new --outfile "$MINT_AUTHORITY"
    fi

    if [ ! -f "$TOKEN_FILE" ]; then
        solana-keygen new --outfile "$TOKEN_FILE"
    fi

    # Store address in array
    MINT_AUTHORITIES+=($(solana address -k "$MINT_AUTHORITY"))
done

# Addresses
RECIPIENT=$(solana address -k $USER)
VAULT=$(solana address -k $VAULT)
AMOUNT=1000000

echo "❗  VAULT=$VAULT"

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
setup_root 10
check_and_airdrop $(solana-keygen pubkey $SOL_ID) 10
check_and_airdrop $(solana-keygen pubkey $MANAGER) 10
check_and_airdrop $(solana-keygen pubkey $CREATOR) 10
check_and_airdrop $(solana-keygen pubkey $VAULT) 10
check_and_airdrop $(solana-keygen pubkey $USER) 10
check_and_airdrop $(solana-keygen pubkey $RECIPIENT) 10
for auth in "${MINT_AUTHORITIES[@]}"; do
    check_and_airdrop $auth 1
done

# Process tokens
for i in "${!TOKEN_NAMES[@]}"; do
    TOKEN_NAME="${TOKEN_NAMES[$i]}"
    MINT_AUTHORITY="${MINT_AUTHORITIES[$i]}"

    TOKEN_FILE=".config/"$TOKEN_NAME"/token.json"
    AUTHORITY_FILE=".config/"$TOKEN_NAME"/id.json"

    if [ ! -f $TOKEN_FILE ]; then
        # Create token & grab MINT_ADDRESS
        stdout=$(spl-token create-token --mint-authority $MINT_AUTHORITY --fee-payer $MANAGER $TOKEN_FILE)
        MINT_ADDRESS=$(echo $stdout | awk '{print $3}')

        # Print the address
        echo "✅  Created $TOKEN_NAME token with address: $MINT_ADDRESS"

        if [ $TOKEN_NAME == "dAnti" ]; then
            echo "❗  ANTI_TOKEN_MINT="$MINT_ADDRESS
        else
            echo "❗   PRO_TOKEN_MINT="$MINT_ADDRESS
        fi

        # Create token account to receive tokens
        spl-token create-account $MINT_ADDRESS --owner $RECIPIENT --fee-payer $MANAGER

        # Mint tokens to recipient
        spl-token mint $MINT_ADDRESS $AMOUNT --mint-authority $AUTHORITY_FILE --recipient-owner $RECIPIENT

        # Verify token airdrop
        spl-token accounts --owner $RECIPIENT
    fi
done

echo "✅  Setup complete"

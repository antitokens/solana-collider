#!/bin/bash
chmod -R a+rx .config

# Core Config
FEE_PAYER=".config/id.json"
USER=".config/user.json"
VAULT=".config/dVault/id.json"

# Make wallets
if [ ! -f $FEE_PAYER ]; then
    solana-keygen new --outfile $FEE_PAYER
fi
if [ ! -f $USER ]; then
    solana-keygen new --outfile $USER
fi
if [ ! -f $VAULT ]; then
    solana-keygen new --outfile $VAULT
fi

# Derived Config
TOKEN_NAMES=("dAnti" "dPro")
ANTI_MINT_AUTHORITY=".config/"${TOKEN_NAMES[0]}"/id.json"
PRO_MINT_AUTHORITY=".config/"${TOKEN_NAMES[1]}"/id.json"

# Make derived wallets
if [ ! -f $ANTI_MINT_AUTHORITY ]; then
    solana-keygen new --outfile $ANTI_MINT_AUTHORITY
fi
if [ ! -f $PRO_MINT_AUTHORITY ]; then
    solana-keygen new --outfile $PRO_MINT_AUTHORITY
fi

# Addresses
MINT_AUTHORITIES=($(solana address -k $ANTI_MINT_AUTHORITY) $(solana address -k $PRO_MINT_AUTHORITY))
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

    TOKEN_FILE=".config/"$TOKEN_NAME"/token.json"
    AUTHORITY_FILE=".config/"$TOKEN_NAME"/id.json"

    if [ ! -f $TOKEN_FILE ]; then
        # Create token & grab MINT_ADDRESS
        stdout=$(spl-token create-token --mint-authority $MINT_AUTHORITY --fee-payer $FEE_PAYER $TOKEN_FILE)
        MINT_ADDRESS=$(echo $stdout | awk '{print $3}')

        # Print the address
        echo "✅  Created $TOKEN_NAME token with address: $MINT_ADDRESS"

        if [ $TOKEN_NAME == "dAnti" ]; then
            echo "❗  ANTI_TOKEN_MINT="$MINT_ADDRESS
        else
            echo "❗  PRO_TOKEN_MINT="$MINT_ADDRESS
        fi

        # Create token account to receive tokens
        spl-token create-account $MINT_ADDRESS --owner $RECIPIENT --fee-payer $FEE_PAYER

        # Mint tokens to recipient
        spl-token mint $MINT_ADDRESS $AMOUNT --mint-authority $AUTHORITY_FILE --recipient-owner $RECIPIENT

        # Verify token airdrop
        spl-token accounts --owner $RECIPIENT
    fi
done

echo "✅  Setup complete"

#!/bin/bash
chmod -R a+rx .config

# Core Config
DEPLOYER=".config/id.json"
USER=".config/dUser/id.json"
CREATOR=".config/dCreator/id.json"
MANAGER=".config/dManager/id.json"
VAULT=".config/dVault/id.json"
TICKERS=("dAnti" "dPro")

# Make wallets
if [ ! -f $DEPLOYER ] || [ ! -s $DEPLOYER ]; then
    solana-keygen new --outfile $DEPLOYER
fi
if [ ! -f $MANAGER ] || [ ! -s $MANAGER ]; then
    solana-keygen new --outfile $MANAGER
fi
if [ ! -f $CREATOR ] || [ ! -s $CREATOR ]; then
    solana-keygen new --outfile $CREATOR
fi
if [ ! -f $USER ] || [ ! -s $USER ]; then
    solana-keygen new --outfile $USER
fi
if [ ! -f $VAULT ] || [ ! -s $VAULT ]; then
    solana-keygen new --outfile $VAULT
fi

# Make derived wallets
for TOKEN_NAME in "${TICKERS[@]}"; do
    MINT_AUTHORITY=".config/$TOKEN_NAME/id.json"
    TOKEN_FILE=".config/$TOKEN_NAME/token.json"

    if [ ! -f "$MINT_AUTHORITY" ] || [ ! -s "$MINT_AUTHORITY" ]; then
        solana-keygen new --outfile "$MINT_AUTHORITY"
    fi

    if [ ! -f "$TOKEN_FILE" ] || [ ! -s "$TOKEN_FILE" ]; then
        solana-keygen new --outfile "$TOKEN_FILE"
    fi

    # Store address in array
    MINT_AUTHORITIES+=($(solana-keygen pubkey "$MINT_AUTHORITY"))
done

echo "❗  VAULT=$(solana-keygen pubkey $VAULT)"
# Check if running on macOS by checking the operating system type
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/^VAULT=.*/VAULT=$(solana-keygen pubkey $VAULT)/" .env
else
    # Linux and others
    sed -i "s/^VAULT=.*/VAULT=$(solana-keygen pubkey $VAULT)/" .env
fi

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
setup_root 10 || check_and_airdrop $(solana-keygen pubkey $DEPLOYER) 10
check_and_airdrop $(solana-keygen pubkey $MANAGER) 1
check_and_airdrop $(solana-keygen pubkey $CREATOR) 1
check_and_airdrop $(solana-keygen pubkey $VAULT) 1
check_and_airdrop $(solana-keygen pubkey $USER) 1
for auth in "${MINT_AUTHORITIES[@]}"; do
    check_and_airdrop $auth 1
done

# Process tokens
for i in "${!TICKERS[@]}"; do
    TOKEN_NAME="${TICKERS[$i]}"
    MINT_AUTHORITY="${MINT_AUTHORITIES[$i]}"

    TOKEN_FILE=".config/"$TOKEN_NAME"/token.json"
    AUTHORITY_FILE=".config/"$TOKEN_NAME"/id.json"

    if [ ! -f "$TOKEN_FILE" ] || [ ! -s "$TOKEN_FILE" ]; then

        # Create token & grab MINT_ADDRESS
        stdout=$(spl-token create-token --mint-authority $MINT_AUTHORITY --fee-payer $MANAGER $TOKEN_FILE)
        MINT_ADDRESS=$(echo $stdout | awk '{print $3}')

        # Print the address
        echo "✅  Created $TOKEN_NAME token with address: $MINT_ADDRESS"

        if [ $TOKEN_NAME == "dAnti" ]; then
            echo "❗  ANTI_TOKEN_MINT="$MINT_ADDRESS
            # Add this to field ANTI_TOKEN_MINT in .env
            if [[ "$OSTYPE" == "darwin"* ]]; then
                sed -i '' "s/^ANTI_TOKEN_MINT=.*/ANTI_TOKEN_MINT=$MINT_ADDRESS/" .env
            else
                sed -i "s/^ANTI_TOKEN_MINT=.*/ANTI_TOKEN_MINT=$MINT_ADDRESS/" .env
            fi
        else
            echo "❗  PRO_TOKEN_MINT="$MINT_ADDRESS
            # Add this to field PRO_TOKEN_MINT in .env
            if [[ "$OSTYPE" == "darwin"* ]]; then
                sed -i '' "s/^PRO_TOKEN_MINT=.*/PRO_TOKEN_MINT=$MINT_ADDRESS/" .env
            else
                sed -i "s/^PRO_TOKEN_MINT=.*/PRO_TOKEN_MINT=$MINT_ADDRESS/" .env
            fi
        fi

        # Amount to mint
        AMOUNT=1000000

        # Create token account to receive tokens
        spl-token create-account $MINT_ADDRESS --owner $(solana-keygen pubkey $USER) --fee-payer $MANAGER

        # Mint tokens to recipient
        spl-token mint $MINT_ADDRESS $AMOUNT --mint-authority $AUTHORITY_FILE --recipient-owner $(solana-keygen pubkey $USER)

        # Verify token airdrop
        spl-token accounts --owner $(solana-keygen pubkey $USER)
    fi
done

echo "✅  Setup complete"

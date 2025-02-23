#!bin/bash
MINT_AUTHORITY = AN5hFEFeWJR4SrYUaSbR4R521PHkpqZcGvqPqjJGGbNi
RECIPIENT = Be3KKiyybpHacAYbxZBRVoF1nAdq5kyBL2kqJ7MdivVi
AMOUNT = 1000000

# Create token & grab MINT_ADDRESS
sudo spl-token create-token --mint-authority $MINT_AUTHORITY

# Create token account to recieve tokens
spl-token create-account $MINT_ADDRESS --owner $RECIPIENT

# Mint tokens to recipient
sudo spl-token mint $MINT_ADDRESS $AMOUNT --mint-authority $MINT_AUTHORITY --recipient-owner $RECIPIENT

# Verify token airdrop
spl-token accounts --owner $RECIPIENT
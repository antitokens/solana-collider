use solana_program::{
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction,
};
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    system_program,
    hash::Hash,
};
use spl_token_2022::{
    instruction as token_instruction,
    state::{Account as TokenAccount, Mint},
};

pub struct TestContext {
    pub program_id: Pubkey,
    pub payer: Keypair,
    pub last_blockhash: Hash,
    pub banks_client: BanksClient,
}

impl TestContext {
    pub async fn new() -> Self {
        let program_id = Pubkey::new_unique();
        let mut program_test = ProgramTest::new(
            "solana_collider",
            program_id,
            processor!(crate::processor::Processor::process),
        );

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        TestContext {
            program_id,
            payer,
            last_blockhash: recent_blockhash,
            banks_client,
        }
    }

    pub async fn create_token_mint(&mut self) -> Result<Pubkey, BanksClientError> {
        let mint_keypair = Keypair::new();
        let mint_rent = self.banks_client
            .get_rent()
            .await?
            .minimum_balance(Mint::LEN);

        // Create account for mint
        let create_mint_account_ix = system_instruction::create_account(
            &self.payer.pubkey(),
            &mint_keypair.pubkey(),
            mint_rent,
            Mint::LEN as u64,
            &spl_token_2022::id(),
        );

        // Initialise mint
        let init_mint_ix = token_instruction::initialise_mint(
            &spl_token_2022::id(),
            &mint_keypair.pubkey(),
            &self.payer.pubkey(),
            Some(&self.payer.pubkey()),
            9,
        )?;

        let transaction = Transaction::new_signed_with_payer(
            &[create_mint_account_ix, init_mint_ix],
            Some(&self.payer.pubkey()),
            &[&self.payer, &mint_keypair],
            self.last_blockhash,
        );

        self.banks_client.process_transaction(transaction).await?;

        Ok(mint_keypair.pubkey())
    }

    pub async fn create_token_account(
        &mut self,
        mint: &Pubkey,
        owner: &Pubkey,
    ) -> Result<Pubkey, BanksClientError> {
        let account_keypair = Keypair::new();
        let account_rent = self.banks_client
            .get_rent()
            .await?
            .minimum_balance(TokenAccount::LEN);

        // Create account
        let create_account_ix = system_instruction::create_account(
            &self.payer.pubkey(),
            &account_keypair.pubkey(),
            account_rent,
            TokenAccount::LEN as u64,
            &spl_token_2022::id(),
        );

        // Initialise token account
        let init_account_ix = token_instruction::initialise_account(
            &spl_token_2022::id(),
            &account_keypair.pubkey(),
            mint,
            owner,
        )?;

        let transaction = Transaction::new_signed_with_payer(
            &[create_account_ix, init_account_ix],
            Some(&self.payer.pubkey()),
            &[&self.payer, &account_keypair],
            self.last_blockhash,
        );

        self.banks_client.process_transaction(transaction).await?;

        Ok(account_keypair.pubkey())
    }

    pub async fn mint_tokens(
        &mut self,
        mint: &Pubkey,
        account: &Pubkey,
        amount: u64,
    ) -> Result<(), BanksClientError> {
        let mint_ix = token_instruction::mint_to(
            &spl_token_2022::id(),
            mint,
            account,
            &self.payer.pubkey(),
            &[],
            amount,
        )?;

        let transaction = Transaction::new_signed_with_payer(
            &[mint_ix],
            Some(&self.payer.pubkey()),
            &[&self.payer],
            self.last_blockhash,
        );

        self.banks_client.process_transaction(transaction).await?;

        Ok(())
    }

    pub async fn get_token_balance(&mut self, account: &Pubkey) -> Result<u64, BanksClientError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        let token_account = TokenAccount::unpack(&account.data)?;
        Ok(token_account.amount)
    }

    pub async fn update_blockhash(&mut self) -> Result<(), BanksClientError> {
        self.last_blockhash = self.banks_client
            .get_latest_blockhash()
            .await?;
        Ok(())
    }
}
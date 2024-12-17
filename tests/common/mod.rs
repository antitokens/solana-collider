//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's standard tests
//! Version: 0.0.1
//! License: MIT
//! Created: 17 Dec 2024
//! Last Modified: 17 Dec 2024
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

use solana_program::{program_pack::Pack, pubkey::Pubkey, system_instruction};
use solana_program_test::*;
use solana_sdk::{
    hash::Hash,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};
use spl_token_2022::{
    instruction as token_instruction,
    state::{Account as TokenAccount, Mint},
};
use crate::state::CollisionState;

pub struct TestContext {
    pub program_id: Pubkey,
    pub payer: Keypair,
    pub last_blockhash: Hash,
    pub banks_client: BanksClient,
    pub authority: Pubkey,
    pub authority_bump: u8,
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
        let (authority, authority_bump) = 
            Pubkey::find_program_address(&[b"authority"], &program_id);

        TestContext {
            program_id,
            payer,
            last_blockhash: recent_blockhash,
            banks_client,
            authority,
            authority_bump,
        }
    }

    pub async fn create_token_mint(&mut self, authority: Option<&Pubkey>) -> Result<Pubkey, BanksClientError> {
        let mint_keypair = Keypair::new();
        let mint_rent = self
            .banks_client
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

        // Initialise mint with specified authority or payer
        let mint_authority = authority.unwrap_or(&self.payer.pubkey());
        let freeze_authority = authority.or(Some(&self.payer.pubkey()));

        let init_mint_ix = token_instruction::initialize_mint(
            &spl_token_2022::id(),
            &mint_keypair.pubkey(),
            mint_authority,
            freeze_authority,
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
        let account_rent = self
            .banks_client
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

        // Initialise token account using initialize_account3
        let init_account_ix = token_instruction::initialize_account3(
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

    pub async fn create_vault_account(
        &mut self,
        mint: &Pubkey,
    ) -> Result<Pubkey, BanksClientError> {
        // Create vault owned by PDA
        self.create_token_account(mint, &self.authority).await
    }

    pub async fn create_collision_state(
        &mut self,
        baryon_mint: &Pubkey,
        photon_mint: &Pubkey,
        vault_anti: &Pubkey,
        vault_pro: &Pubkey,
    ) -> Result<Pubkey, BanksClientError> {
        let state_keypair = Keypair::new();
        let state_rent = self
            .banks_client
            .get_rent()
            .await?
            .minimum_balance(CollisionState::LEN);

        // Create state account
        let create_state_ix = system_instruction::create_account(
            &self.payer.pubkey(),
            &state_keypair.pubkey(),
            state_rent,
            CollisionState::LEN as u64,
            &self.program_id,
        );

        let transaction = Transaction::new_signed_with_payer(
            &[create_state_ix],
            Some(&self.payer.pubkey()),
            &[&self.payer, &state_keypair],
            self.last_blockhash,
        );

        self.banks_client.process_transaction(transaction).await?;

        Ok(state_keypair.pubkey())
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

    pub async fn get_vault_balances(
        &mut self,
        vault_anti: &Pubkey,
        vault_pro: &Pubkey,
    ) -> Result<(u64, u64), BanksClientError> {
        let anti_balance = self.get_token_balance(vault_anti).await?;
        let pro_balance = self.get_token_balance(vault_pro).await?;
        Ok((anti_balance, pro_balance))
    }

    pub async fn update_blockhash(&mut self) -> Result<(), BanksClientError> {
        self.last_blockhash = self.banks_client.get_latest_blockhash().await?;
        Ok(())
    }
}
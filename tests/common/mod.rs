use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
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
        // Implementation here
        Ok(Pubkey::new_unique())
    }

    pub async fn create_token_account(
        &mut self,
        mint: &Pubkey,
        owner: &Pubkey,
    ) -> Result<Pubkey, BanksClientError> {
        // Implementation here
        Ok(Pubkey::new_unique())
    }
}

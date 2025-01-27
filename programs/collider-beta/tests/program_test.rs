use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use anchor_spl::token::{self, Mint, TokenAccount};
use collider_beta::{self, instructions::*};
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    instruction::Instruction,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_token_2022::instruction as token_instruction;

#[tokio::test]
async fn test_full_collider_flow() {
    let program_id = collider_beta::ID;
    let mut program_test = ProgramTest::default();
    program_test.add_program("collider_beta", program_id, None);

    // Create test accounts
    let authority = Keypair::new();
    let user = Keypair::new();

    // Add SOL accounts
    program_test.add_account(
        authority.pubkey(),
        Account {
            lamports: 10_000_000_000, // 10 SOL
            data: vec![],
            owner: solana_program::system_program::id(),
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        user.pubkey(),
        Account {
            lamports: 10_000_000_000,
            data: vec![],
            owner: solana_program::system_program::id(),
            executable: false,
            rent_epoch: 0,
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Create token mints
    let anti_mint = Keypair::new();
    let pro_mint = Keypair::new();

    // Initialise mints
    let rent = banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(Mint::LEN);

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &anti_mint.pubkey(),
                mint_rent,
                Mint::LEN as u64,
                &token::ID,
            ),
            system_instruction::create_account(
                &payer.pubkey(),
                &pro_mint.pubkey(),
                mint_rent,
                Mint::LEN as u64,
                &token::ID,
            ),
            token_instruction::initialize_mint(
                &token::ID,
                &anti_mint.pubkey(),
                &authority.pubkey(),
                None,
                9,
            )
            .unwrap(),
            token_instruction::initialize_mint(
                &token::ID,
                &pro_mint.pubkey(),
                &authority.pubkey(),
                None,
                9,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[&payer, &anti_mint, &pro_mint],
        recent_blockhash,
    );

    banks_client.process_transaction(tx).await.unwrap();

    // Create state PDA
    let (state_pda, _) = Pubkey::find_program_address(&[b"state"], &program_id);

    // Initialise program
    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(state_pda, false),
            AccountMeta::new_readonly(authority.pubkey(), true),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ],
        data: collider_beta::instruction::Initialiser {}.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer, &authority],
        recent_blockhash,
    );

    banks_client.process_transaction(tx).await.unwrap();

    // Create poll
    let poll_index: u32 = 0;
    let (poll_pda, _) =
        Pubkey::find_program_address(&[b"poll", poll_index.to_le_bytes().as_ref()], &program_id);

    let create_poll_data = CreatePollArgs {
        title: "Test Poll".to_string(),
        description: "Test Description".to_string(),
        start_time: "2025-01-21T00:00:00Z".to_string(),
        end_time: "2025-01-22T00:00:00Z".to_string(),
        etc: None,
    };

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(state_pda, false),
            AccountMeta::new(poll_pda, false),
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ],
        data: AnchorSerialize::try_to_vec(&create_poll_data).unwrap(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer, &authority],
        recent_blockhash,
    );

    banks_client.process_transaction(tx).await.unwrap();

    // Create token accounts for the poll
    let poll_anti_token = Keypair::new();
    let poll_pro_token = Keypair::new();

    let token_rent = rent.minimum_balance(TokenAccount::LEN);

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &poll_anti_token.pubkey(),
                token_rent,
                TokenAccount::LEN as u64,
                &token::ID,
            ),
            system_instruction::create_account(
                &payer.pubkey(),
                &poll_pro_token.pubkey(),
                token_rent,
                TokenAccount::LEN as u64,
                &token::ID,
            ),
            token_instruction::initialize_account(
                &token::ID,
                &poll_anti_token.pubkey(),
                &anti_mint.pubkey(),
                &poll_pda,
            )
            .unwrap(),
            token_instruction::initialize_account(
                &token::ID,
                &poll_pro_token.pubkey(),
                &pro_mint.pubkey(),
                &poll_pda,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[&payer, &poll_anti_token, &poll_pro_token],
        recent_blockhash,
    );

    banks_client.process_transaction(tx).await.unwrap();
}

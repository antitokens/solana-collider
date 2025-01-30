use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use anchor_spl::token::spl_token;
use anchor_spl::token::{self, Mint, TokenAccount};
use collider_beta::instruction::Initialiser;
use solana_program_test::*;
use solana_sdk::system_program;
use solana_sdk::{
    account::Account,
    instruction::Instruction,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_token::instruction as token_instruction;

#[tokio::test]
async fn test_full_collider_flow() {
    // Initialise the test environment
    let program_id = collider_beta::ID;
    let mut program_test = ProgramTest::default();
    program_test.add_program("collider_beta", program_id, None);

    // Create test keypairs
    let authority = Keypair::new();
    let user = Keypair::new();

    // Initialise accounts with 10 SOL each
    program_test.add_account(
        authority.pubkey(),
        Account {
            lamports: 10_000_000_000,
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

    // Create ANTI and PRO token mints
    let anti_mint = Keypair::new();
    let pro_mint = Keypair::new();

    // Get rent for various account types
    let rent = banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(Mint::LEN);
    let token_rent = rent.minimum_balance(TokenAccount::LEN);

    // Initialise token mints
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

    // Create user token accounts
    let user_anti_token = Keypair::new();
    let user_pro_token = Keypair::new();

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &user_anti_token.pubkey(),
                token_rent,
                TokenAccount::LEN as u64,
                &token::ID,
            ),
            system_instruction::create_account(
                &payer.pubkey(),
                &user_pro_token.pubkey(),
                token_rent,
                TokenAccount::LEN as u64,
                &token::ID,
            ),
            token_instruction::initialize_account(
                &token::ID,
                &user_anti_token.pubkey(),
                &anti_mint.pubkey(),
                &user.pubkey(),
            )
            .unwrap(),
            token_instruction::initialize_account(
                &token::ID,
                &user_pro_token.pubkey(),
                &pro_mint.pubkey(),
                &user.pubkey(),
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[&payer, &user_anti_token, &user_pro_token],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();

    // Mint tokens to user accounts
    let tx = Transaction::new_signed_with_payer(
        &[
            token_instruction::mint_to(
                &token::ID,
                &anti_mint.pubkey(),
                &user_anti_token.pubkey(),
                &authority.pubkey(),
                &[],
                10_000_000_000,
            )
            .unwrap(),
            token_instruction::mint_to(
                &token::ID,
                &pro_mint.pubkey(),
                &user_pro_token.pubkey(),
                &authority.pubkey(),
                &[],
                10_000_000_000,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[&payer, &authority],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();

    // Initialise program state
    let (state_pda, _) = Pubkey::find_program_address(&[b"state"], &program_id);

    // Create the initialisation instruction
    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(state_pda, false),
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: Initialiser {}.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer, &authority],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();
    println!("✅ Initialisation passing ...");

    // Create poll
    let poll_index = 0u64;
    let (poll_pda, _) =
        Pubkey::find_program_address(&[b"poll", poll_index.to_le_bytes().as_ref()], &program_id);

    let create_poll_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(state_pda, false),
            AccountMeta::new(poll_pda, false),
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ],
        data: collider_beta::instruction::CreatePoll {
            title: "Test Poll".to_string(),
            description: "Test Description".to_string(),
            start_time: "2025-02-01T00:00:00Z".to_string(),
            end_time: "2025-03-01T00:00:00Z".to_string(),
            etc: None,
            unix_timestamp: Some(1736899200), // 2025-01-15T00:00:00Z for testing
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[create_poll_ix],
        Some(&payer.pubkey()),
        &[&payer, &authority],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();
    println!("✅ CreatePoll passing ...");

    // Create poll token accounts
    let poll_anti_token = Keypair::new();
    let poll_pro_token = Keypair::new();

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

    // Deposit tokens
    let deposit_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(poll_pda, false),
            AccountMeta::new(user.pubkey(), true),
            AccountMeta::new(user_anti_token.pubkey(), false),
            AccountMeta::new(user_pro_token.pubkey(), false),
            AccountMeta::new(poll_anti_token.pubkey(), false),
            AccountMeta::new(poll_pro_token.pubkey(), false),
            AccountMeta::new_readonly(token::ID, false),
        ],
        data: collider_beta::instruction::DepositTokens {
            poll_index,
            anti: 7_000_000_000,
            pro: 3_000_000_000,
            unix_timestamp: Some(1739577600), // 2025-02-15T00:00:00Z for testing
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[deposit_ix],
        Some(&payer.pubkey()),
        &[&payer, &user],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();
    println!("✅ Deposits passing ...");

    // Equalise poll
    let equalise_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(poll_pda, false),
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new(user_anti_token.pubkey(), false),
            AccountMeta::new(user_pro_token.pubkey(), false),
            AccountMeta::new(poll_anti_token.pubkey(), false),
            AccountMeta::new(poll_pro_token.pubkey(), false),
            AccountMeta::new_readonly(token::ID, false),
        ],
        data: collider_beta::instruction::EqualiseTokens {
            poll_index,
            truth: vec![6000, 4000],
            unix_timestamp: Some(1741996800), // 2025-03-15T00:00:00Z for testing
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[equalise_ix],
        Some(&payer.pubkey()),
        &[&payer, &authority],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();
    println!("✅ Equalisation passing ...");

    // Withdraw tokens
    let withdraw_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(poll_pda, false),
            AccountMeta::new(user.pubkey(), true),
            AccountMeta::new(user_anti_token.pubkey(), false),
            AccountMeta::new(user_pro_token.pubkey(), false),
            AccountMeta::new(poll_anti_token.pubkey(), false),
            AccountMeta::new(poll_pro_token.pubkey(), false),
            AccountMeta::new_readonly(token::ID, false),
        ],
        data: collider_beta::instruction::WithdrawTokens { poll_index }.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[withdraw_ix],
        Some(&payer.pubkey()),
        &[&payer, &user],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();
    println!("✅ Withdrawals passing ...");
}

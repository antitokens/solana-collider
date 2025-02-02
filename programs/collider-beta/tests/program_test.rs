use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use anchor_spl::token::spl_token;
use anchor_spl::token::{self, Mint, TokenAccount};
use collider_beta::instruction::Initialiser;
use collider_beta::utils::ANTITOKEN_MULTISIG;
use solana_program_test::*;
use solana_sdk::system_program;
use solana_sdk::sysvar;
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
    /* Global setup begins here */
    // Function to generate fixed keypairs for testing
    fn keypair_from_parts(secret: [u8; 32], expected_pubkey_str: &str) -> Keypair {
        // Create empty 64-byte array for full keypair
        let mut keypair_bytes = [0u8; 64];

        // Copy secret to first 32 bytes
        keypair_bytes[..32].copy_from_slice(&secret);

        // Get expected pubkey bytes and copy to last 32 bytes
        let expected_pubkey = Pubkey::from_str(expected_pubkey_str).unwrap();
        keypair_bytes[32..].copy_from_slice(expected_pubkey.as_ref());

        let keypair = Keypair::from_bytes(&keypair_bytes).unwrap();

        // Verify the derived pubkey matches what we expect
        assert_eq!(
            keypair.pubkey(),
            expected_pubkey,
            "Derived pubkey does not match expected pubkey"
        );

        keypair
    }

    // Generate fixed keypairs to mimic ANTI_MINT_ADDRESS, PRO_MINT_ADDRESS and ANTITOKEN_MULTISIG
    let anti_mint = keypair_from_parts(
        [
            239, 190, 227, 14, 160, 236, 240, 48, 197, 85, 133, 154, 162, 0, 104, 165, 142, 246,
            161, 41, 242, 135, 73, 228, 96, 153, 166, 106, 99, 90, 29, 240,
        ],
        "4ZkEvqRny1khv9Cj8SGbV364SSBBHZJRc4mykkLzFjX2",
    );

    let pro_mint = keypair_from_parts(
        [
            207, 45, 179, 220, 32, 150, 106, 44, 115, 149, 90, 137, 157, 116, 16, 225, 151, 252,
            245, 253, 234, 227, 14, 169, 45, 97, 133, 48, 218, 129, 239, 175,
        ],
        "838v9XjmvSMMyFfULwNZZcBcGabdQjEVVPixAFunhX6y",
    );

    let antitoken_multisig = keypair_from_parts(
        [
            254, 97, 231, 93, 237, 143, 203, 126, 1, 178, 146, 247, 85, 208, 86, 187, 223, 80, 74,
            76, 242, 136, 240, 74, 66, 2, 119, 190, 41, 56, 68, 147,
        ],
        "4y85fZmnMmxD4YndrTba794StNLcvzSsNTsHnb97dYJk",
    );

    // Initialise the test environment
    let program_id = collider_beta::ID;
    let mut program_test = ProgramTest::default();
    program_test.add_program("collider_beta", program_id, None);

    // Define program state
    let (state_pda, _) = Pubkey::find_program_address(&[b"state"], &program_id);

    // Create test keypairs
    let manager = Keypair::new();
    let creator = Keypair::new();
    let user = Keypair::new();

    // Derive PDAs for poll token accounts
    let poll_index = 0u64;

    let (poll_pda, _) =
        Pubkey::find_program_address(&[b"poll", poll_index.to_le_bytes().as_ref()], &program_id);

    let (poll_anti_token_pda, _) = Pubkey::find_program_address(
        &[b"anti_token", poll_index.to_le_bytes().as_ref()],
        &program_id,
    );

    let (poll_pro_token_pda, _) = Pubkey::find_program_address(
        &[b"pro_token", poll_index.to_le_bytes().as_ref()],
        &program_id,
    );

    // Initialise accounts with 10 SOL each
    program_test.add_account(
        manager.pubkey(),
        Account {
            lamports: 10_000_000_000,
            data: vec![],
            owner: solana_program::system_program::id(),
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        creator.pubkey(),
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

    program_test.add_account(
        antitoken_multisig.pubkey(),
        Account {
            lamports: 10_000_000_000,
            data: vec![],
            owner: solana_program::system_program::id(),
            executable: false,
            rent_epoch: 0,
        },
    );

    let (mut banks_client, _, recent_blockhash) = program_test.start().await;

    // Get rent for various account types
    let rent = banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(Mint::LEN);
    let token_rent = rent.minimum_balance(TokenAccount::LEN);

    // Initialise token mints
    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &manager.pubkey(),
                &anti_mint.pubkey(),
                mint_rent,
                Mint::LEN as u64,
                &token::ID,
            ),
            system_instruction::create_account(
                &manager.pubkey(),
                &pro_mint.pubkey(),
                mint_rent,
                Mint::LEN as u64,
                &token::ID,
            ),
            token_instruction::initialize_mint(
                &token::ID,
                &anti_mint.pubkey(),
                &manager.pubkey(),
                None,
                9,
            )
            .unwrap(),
            token_instruction::initialize_mint(
                &token::ID,
                &pro_mint.pubkey(),
                &manager.pubkey(),
                None,
                9,
            )
            .unwrap(),
        ],
        Some(&manager.pubkey()),
        &[&manager, &anti_mint, &pro_mint],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();

    // Create user token accounts
    let mut user_token_accounts = vec![];
    let user_anti_token = Keypair::new();
    let user_pro_token = Keypair::new();

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &manager.pubkey(),
                &user_anti_token.pubkey(),
                token_rent,
                TokenAccount::LEN as u64,
                &token::ID,
            ),
            system_instruction::create_account(
                &manager.pubkey(),
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
        Some(&manager.pubkey()),
        &[&manager, &user_anti_token, &user_pro_token],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();
    user_token_accounts.push((user_anti_token.pubkey(), user_pro_token.pubkey()));

    // Mint tokens to user accounts
    let tx = Transaction::new_signed_with_payer(
        &[
            token_instruction::mint_to(
                &token::ID,
                &anti_mint.pubkey(),
                &user_anti_token.pubkey(),
                &manager.pubkey(),
                &[],
                10_000_000_000,
            )
            .unwrap(),
            token_instruction::mint_to(
                &token::ID,
                &pro_mint.pubkey(),
                &user_pro_token.pubkey(),
                &manager.pubkey(),
                &[],
                10_000_000_000,
            )
            .unwrap(),
        ],
        Some(&manager.pubkey()),
        &[&manager],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();
    /* Global setup ends here */

    // Create the initialisation instruction
    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(state_pda, false),
            AccountMeta::new(manager.pubkey(), true),
            AccountMeta::new(system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
        ],
        data: Initialiser {}.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&manager.pubkey()),
        &[&manager],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();
    println!("✅ Initialisation passing ...");

    // Create poll instruction
    let create_poll_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(state_pda, false),
            AccountMeta::new(poll_pda, false),
            AccountMeta::new(creator.pubkey(), true),
            AccountMeta::new(poll_anti_token_pda, false),
            AccountMeta::new(poll_pro_token_pda, false),
            AccountMeta::new(anti_mint.pubkey(), false),
            AccountMeta::new(pro_mint.pubkey(), false),
            AccountMeta::new_readonly(token::ID, false),
            AccountMeta::new_readonly(solana_program::system_program::ID, false),
            AccountMeta::new(ANTITOKEN_MULTISIG, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
        ],
        data: collider_beta::instruction::CreatePoll {
            title: "Test Poll".to_string(),
            description: "Test Description".to_string(),
            start_time: "2025-02-01T00:00:00Z".to_string(),
            end_time: "2025-03-01T00:00:00Z".to_string(),
            etc: None,
            unix_timestamp: Some(1736899200),
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[create_poll_ix],
        Some(&creator.pubkey()),
        &[&creator],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap(); // This fails because create_poll sees state.poll_index = 1284939282, not 0 !
    println!("✅ CreatePoll passing ...");

    // Deposit tokens
    let deposit_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(poll_pda, false),
            AccountMeta::new(user.pubkey(), true),
            AccountMeta::new(user_anti_token.pubkey(), false),
            AccountMeta::new(user_pro_token.pubkey(), false),
            AccountMeta::new(poll_anti_token_pda, false),
            AccountMeta::new(poll_pro_token_pda, false),
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
        Some(&manager.pubkey()),
        &[&manager, &user],
        recent_blockhash,
    );

    banks_client.process_transaction(tx).await.unwrap();
    println!("✅ Deposits passing ...");

    // Equalise poll
    let equalise_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(poll_pda, false),
            AccountMeta::new(manager.pubkey(), true),
            AccountMeta::new(user_anti_token.pubkey(), false),
            AccountMeta::new(user_pro_token.pubkey(), false),
            AccountMeta::new(poll_anti_token_pda, false),
            AccountMeta::new(poll_pro_token_pda, false),
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
        Some(&manager.pubkey()),
        &[&manager, &manager],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();
    println!("✅ Equalisation passing ...");

    // Withdraw tokens
    let mut withdraw_accounts = vec![
        AccountMeta::new(poll_pda, false),
        AccountMeta::new(antitoken_multisig.pubkey(), true), // Only the multisig can withdraw
        AccountMeta::new(poll_anti_token_pda, false),        // Anti Token PDA
        AccountMeta::new(poll_pro_token_pda, false),         // Pro Token PDA
        AccountMeta::new_readonly(token::ID, false),
    ];

    // Include all user token accounts dynamically
    for (anti_account, pro_account) in user_token_accounts.iter() {
        withdraw_accounts.push(AccountMeta::new(*anti_account, false));
        withdraw_accounts.push(AccountMeta::new(*pro_account, false));
    }

    let withdraw_ix = Instruction {
        program_id,
        accounts: withdraw_accounts,
        data: collider_beta::instruction::WithdrawTokens { poll_index }.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[withdraw_ix],
        Some(&antitoken_multisig.pubkey()),
        &[&antitoken_multisig], // Multisig manager should be signing
        recent_blockhash,
    );

    banks_client.process_transaction(tx).await.unwrap();

    println!("✅ Withdrawals passing ...");
}

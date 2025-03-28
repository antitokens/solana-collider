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
            199, 248, 4, 119, 179, 209, 7, 251, 29, 104, 140, 5, 104, 142, 70, 118, 124, 30, 234,
            100, 93, 56, 177, 105, 86, 95, 183, 187, 77, 30, 146, 248,
        ],
        "674rRAKuyAizM6tWKLpo8zDqAtvxYS7ce6DoGBfocmrT",
    );

    let pro_mint = keypair_from_parts(
        [
            154, 211, 254, 243, 5, 250, 22, 77, 89, 239, 46, 250, 57, 45, 194, 24, 18, 196, 39,
            200, 37, 184, 155, 255, 83, 172, 147, 99, 16, 55, 162, 179,
        ],
        "6bDmnBGtGo9pb2vhVkrzQD9uHYcYpBCCSgU61534MyTm",
    );

    let antitoken_multisig = keypair_from_parts(
        [
            12, 63, 179, 210, 90, 185, 236, 243, 1, 37, 19, 188, 76, 159, 88, 72, 82, 172, 171,
            255, 220, 221, 248, 84, 222, 236, 124, 122, 17, 11, 68, 197,
        ],
        "7rFEa4g8UZs7eBBoq66FmLeobtb81dfCPx2Hmt61kJ5t",
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

    // Derive PDAs for prediction token accounts
    let index = 0u64;

    let (admin_pda, _) = Pubkey::find_program_address(&[b"admin"], &program_id);

    let (prediction_pda, _) =
        Pubkey::find_program_address(&[b"prediction", index.to_le_bytes().as_ref()], &program_id);

    let (prediction_anti_token_pda, _) =
        Pubkey::find_program_address(&[b"anti_token", index.to_le_bytes().as_ref()], &program_id);

    let (prediction_pro_token_pda, _) =
        Pubkey::find_program_address(&[b"pro_token", index.to_le_bytes().as_ref()], &program_id);

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

    // Create the admin initialisation instruction
    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(admin_pda, false),
            AccountMeta::new(manager.pubkey(), true),
            AccountMeta::new(system_program::id(), false),
        ],
        data: collider_beta::instruction::InitialiseAdmin {}.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&manager.pubkey()),
        &[&manager],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();
    println!("✅ Admin initialisation passing ...");

    // Test updating prediction creation fee
    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(admin_pda, false),
            AccountMeta::new(antitoken_multisig.pubkey(), true),
        ],
        data: collider_beta::instruction::UpdateCreationFee {
            new_fee: 200_000_000,
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&antitoken_multisig.pubkey()),
        &[&antitoken_multisig],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();
    println!("✅ Prediction creation fee update passing ...");

    // Test updating max title length
    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(admin_pda, false),
            AccountMeta::new(antitoken_multisig.pubkey(), true),
        ],
        data: collider_beta::instruction::UpdateMaxTitleLength { new_length: 512 }.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&antitoken_multisig.pubkey()),
        &[&antitoken_multisig],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();
    println!("✅ Max title length update passing ...");

    // Test updating max description length
    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(admin_pda, false),
            AccountMeta::new(antitoken_multisig.pubkey(), true),
        ],
        data: collider_beta::instruction::UpdateMaxDescriptionLength { new_length: 2048 }.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&antitoken_multisig.pubkey()),
        &[&antitoken_multisig],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();
    println!("✅ Max description length update passing ...");

    // Test updating truth basis
    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(admin_pda, false),
            AccountMeta::new(antitoken_multisig.pubkey(), true),
        ],
        data: collider_beta::instruction::UpdateTruthBasis { new_basis: 200_000 }.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&antitoken_multisig.pubkey()),
        &[&antitoken_multisig],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();
    println!("✅ Truth basis update passing ...");

    // Test updating float basis
    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(admin_pda, false),
            AccountMeta::new(antitoken_multisig.pubkey(), true),
        ],
        data: collider_beta::instruction::UpdateFloatBasis { new_basis: 20_000 }.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&antitoken_multisig.pubkey()),
        &[&antitoken_multisig],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();
    println!("✅ Float basis update passing ...");

    // Test updating min deposit amount
    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(admin_pda, false),
            AccountMeta::new(antitoken_multisig.pubkey(), true),
        ],
        data: collider_beta::instruction::UpdateMinDepositAmount {
            new_min_amount: 20_000,
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&antitoken_multisig.pubkey()),
        &[&antitoken_multisig],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();
    println!("✅ Min deposit amount update passing ...");

    // Test updating $ANTI mint address
    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(admin_pda, false),
            AccountMeta::new(antitoken_multisig.pubkey(), true),
        ],
        data: collider_beta::instruction::UpdateAntiMint {
            new_mint: anti_mint.pubkey(),
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&antitoken_multisig.pubkey()),
        &[&antitoken_multisig],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();
    println!("✅ $ANTI mint update passing ...");

    // Test updating $PRO mint address
    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(admin_pda, false),
            AccountMeta::new(antitoken_multisig.pubkey(), true),
        ],
        data: collider_beta::instruction::UpdateProMint {
            new_mint: pro_mint.pubkey(),
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&antitoken_multisig.pubkey()),
        &[&antitoken_multisig],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();
    println!("✅ $PRO mint update passing ...");

    // Test updating multisig authority
    let new_multisig = Keypair::new();
    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(admin_pda, false),
            AccountMeta::new(antitoken_multisig.pubkey(), true),
        ],
        data: collider_beta::instruction::UpdateMultisig {
            new_multisig: new_multisig.pubkey(),
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&antitoken_multisig.pubkey()),
        &[&antitoken_multisig],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();
    println!("✅ Multisig authority update passing ...");
    println!("✅ Admin actions passing ...");

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

    // Create prediction instruction
    let create_prediction_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(state_pda, false),
            AccountMeta::new(prediction_pda, false),
            AccountMeta::new(creator.pubkey(), true),
            AccountMeta::new(prediction_anti_token_pda, false),
            AccountMeta::new(prediction_pro_token_pda, false),
            AccountMeta::new(anti_mint.pubkey(), false),
            AccountMeta::new(pro_mint.pubkey(), false),
            AccountMeta::new_readonly(token::ID, false),
            AccountMeta::new_readonly(solana_program::system_program::ID, false),
            AccountMeta::new(ANTITOKEN_MULTISIG, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
        ],
        data: collider_beta::instruction::CreatePrediction {
            title: "Test Prediction".to_string(),
            description: "Test Description".to_string(),
            start_time: "2025-02-01T00:00:00Z".to_string(),
            end_time: "2025-03-01T00:00:00Z".to_string(),
            etc: None,
            unix_timestamp: Some(1736899200),
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[create_prediction_ix],
        Some(&creator.pubkey()),
        &[&creator],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();
    println!("✅ CreatePrediction passing ...");

    // Deposit tokens
    let deposit_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(prediction_pda, false),
            AccountMeta::new(user.pubkey(), true),
            AccountMeta::new(user_anti_token.pubkey(), false),
            AccountMeta::new(user_pro_token.pubkey(), false),
            AccountMeta::new(prediction_anti_token_pda, false),
            AccountMeta::new(prediction_pro_token_pda, false),
            AccountMeta::new_readonly(token::ID, false),
        ],
        data: collider_beta::instruction::DepositTokens {
            index,
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

    // Equalise prediction
    let equalise_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(prediction_pda, false),
            AccountMeta::new(manager.pubkey(), true),
            AccountMeta::new(user_anti_token.pubkey(), false),
            AccountMeta::new(user_pro_token.pubkey(), false),
            AccountMeta::new(prediction_anti_token_pda, false),
            AccountMeta::new(prediction_pro_token_pda, false),
            AccountMeta::new_readonly(token::ID, false),
        ],
        data: collider_beta::instruction::EqualiseTokens {
            index,
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
        AccountMeta::new(prediction_pda, false),
        AccountMeta::new(antitoken_multisig.pubkey(), true), // Only the multisig can withdraw
        AccountMeta::new(prediction_anti_token_pda, false),  // Anti Token PDA
        AccountMeta::new(prediction_pro_token_pda, false),   // Pro Token PDA
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
        data: collider_beta::instruction::BulkWithdrawTokens { index }.data(),
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

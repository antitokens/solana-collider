#[cfg(test)]
mod tests {
    use borsh::BorshSerialize;
    use solana_collider::ID;
    use solana_collider::{process_instruction, CollisionInstruction, CollisionState};
    use solana_program::{
        account_info::AccountInfo,
        instruction::{AccountMeta, Instruction},
        program_pack::Pack,
        pubkey::Pubkey,
        system_instruction,
    };
    use solana_program_test::*;
    use solana_sdk::{
        signature::{Keypair, Signer},
        system_program,
        transaction::Transaction,
    };
    use spl_token_2022::state::{Account as TokenAccount, Mint};

    pub const PROGRAM_ID: Pubkey = ID;

    #[tokio::test]
    async fn test_collide() {
        let mut program_test = ProgramTest::new("solana_collider", PROGRAM_ID, None);

        program_test.prefer_bpf(false);
        program_test.add_program(
            "spl_token_2022",
            spl_token_2022::id(),
            processor!(spl_token_2022::processor::Processor::process),
        );

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        // Create all keypairs upfront
        let state_account = Keypair::new();
        let baryon_mint = Keypair::new();
        let photon_mint = Keypair::new();
        let vault_anti = Keypair::new();
        let vault_pro = Keypair::new();
        let anti_mint = Keypair::new();
        let pro_mint = Keypair::new();
        let anti_account = Keypair::new();
        let pro_account = Keypair::new();
        let baryon_account = Keypair::new();
        let photon_account = Keypair::new();

        // Derive PDA for authority
        let (authority_pubkey, _authority_bump) =
            Pubkey::find_program_address(&[b"authority"], &PROGRAM_ID);

        // Get rent amounts
        let rent = banks_client.get_rent().await.unwrap();
        let state_rent = rent.minimum_balance(std::mem::size_of::<CollisionState>());
        let mint_rent = rent.minimum_balance(Mint::LEN);
        let token_rent = rent.minimum_balance(TokenAccount::LEN);

        // Transaction 1: Create state account and output mints
        let create_accounts_tx = Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &payer.pubkey(),
                    &state_account.pubkey(),
                    state_rent,
                    std::mem::size_of::<CollisionState>() as u64,
                    &PROGRAM_ID,
                ),
                system_instruction::create_account(
                    &payer.pubkey(),
                    &baryon_mint.pubkey(),
                    mint_rent,
                    Mint::LEN as u64,
                    &spl_token_2022::id(),
                ),
                system_instruction::create_account(
                    &payer.pubkey(),
                    &photon_mint.pubkey(),
                    mint_rent,
                    Mint::LEN as u64,
                    &spl_token_2022::id(),
                ),
            ],
            Some(&payer.pubkey()),
            &[&payer, &state_account, &baryon_mint, &photon_mint],
            recent_blockhash,
        );
        banks_client
            .process_transaction(create_accounts_tx)
            .await
            .unwrap();

        // Transaction 2: Create vault accounts
        let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
        let create_vaults_tx = Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &payer.pubkey(),
                    &vault_anti.pubkey(),
                    token_rent,
                    TokenAccount::LEN as u64,
                    &spl_token_2022::id(),
                ),
                system_instruction::create_account(
                    &payer.pubkey(),
                    &vault_pro.pubkey(),
                    token_rent,
                    TokenAccount::LEN as u64,
                    &spl_token_2022::id(),
                ),
            ],
            Some(&payer.pubkey()),
            &[&payer, &vault_anti, &vault_pro],
            recent_blockhash,
        );
        banks_client
            .process_transaction(create_vaults_tx)
            .await
            .unwrap();

        // Transaction 3: Initialize output mints
        let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
        let init_mints_tx = Transaction::new_signed_with_payer(
            &[
                spl_token_2022::instruction::initialize_mint(
                    &spl_token_2022::id(),
                    &baryon_mint.pubkey(),
                    &authority_pubkey,
                    Some(&authority_pubkey),
                    9,
                )
                .unwrap(),
                spl_token_2022::instruction::initialize_mint(
                    &spl_token_2022::id(),
                    &photon_mint.pubkey(),
                    &authority_pubkey,
                    Some(&authority_pubkey),
                    9,
                )
                .unwrap(),
            ],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );
        banks_client
            .process_transaction(init_mints_tx)
            .await
            .unwrap();

        // Transaction 4: Initialize vault accounts
        let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
        let init_vaults_tx = Transaction::new_signed_with_payer(
            &[
                spl_token_2022::instruction::initialize_account3(
                    &spl_token_2022::id(),
                    &vault_anti.pubkey(),
                    &baryon_mint.pubkey(),
                    &authority_pubkey,
                )
                .unwrap(),
                spl_token_2022::instruction::initialize_account3(
                    &spl_token_2022::id(),
                    &vault_pro.pubkey(),
                    &photon_mint.pubkey(),
                    &authority_pubkey,
                )
                .unwrap(),
            ],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );
        banks_client
            .process_transaction(init_vaults_tx)
            .await
            .unwrap();

        // Transaction 5: Initialize program state
        let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
        let init_program_tx = Transaction::new_signed_with_payer(
            &[Instruction {
                program_id: PROGRAM_ID,
                accounts: vec![
                    AccountMeta::new(state_account.pubkey(), false),
                    AccountMeta::new(baryon_mint.pubkey(), false),
                    AccountMeta::new(photon_mint.pubkey(), false),
                    AccountMeta::new(vault_anti.pubkey(), false),
                    AccountMeta::new(vault_pro.pubkey(), false),
                    AccountMeta::new(payer.pubkey(), true),
                    AccountMeta::new_readonly(system_program::id(), false),
                    AccountMeta::new_readonly(spl_token_2022::id(), false),
                    AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
                ],
                data: CollisionInstruction::Initialise.try_to_vec().unwrap(),
            }],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );
        banks_client
            .process_transaction(init_program_tx)
            .await
            .unwrap();

        // Transaction 6: Create input mints
        let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
        let create_input_mints_tx = Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &payer.pubkey(),
                    &anti_mint.pubkey(),
                    mint_rent,
                    Mint::LEN as u64,
                    &spl_token_2022::id(),
                ),
                system_instruction::create_account(
                    &payer.pubkey(),
                    &pro_mint.pubkey(),
                    mint_rent,
                    Mint::LEN as u64,
                    &spl_token_2022::id(),
                ),
            ],
            Some(&payer.pubkey()),
            &[&payer, &anti_mint, &pro_mint],
            recent_blockhash,
        );
        banks_client
            .process_transaction(create_input_mints_tx)
            .await
            .unwrap();

        // Transaction 7: Initialize input mints
        let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
        let init_input_mints_tx = Transaction::new_signed_with_payer(
            &[
                spl_token_2022::instruction::initialize_mint(
                    &spl_token_2022::id(),
                    &anti_mint.pubkey(),
                    &payer.pubkey(),
                    Some(&payer.pubkey()),
                    9,
                )
                .unwrap(),
                spl_token_2022::instruction::initialize_mint(
                    &spl_token_2022::id(),
                    &pro_mint.pubkey(),
                    &payer.pubkey(),
                    Some(&payer.pubkey()),
                    9,
                )
                .unwrap(),
            ],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );
        banks_client
            .process_transaction(init_input_mints_tx)
            .await
            .unwrap();

        // Transaction 8a: Create input token accounts
        let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
        let create_input_token_accounts_tx = Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &payer.pubkey(),
                    &anti_account.pubkey(),
                    token_rent,
                    TokenAccount::LEN as u64,
                    &spl_token_2022::id(),
                ),
                system_instruction::create_account(
                    &payer.pubkey(),
                    &pro_account.pubkey(),
                    token_rent,
                    TokenAccount::LEN as u64,
                    &spl_token_2022::id(),
                ),
            ],
            Some(&payer.pubkey()),
            &[&payer, &anti_account, &pro_account],
            recent_blockhash,
        );
        banks_client
            .process_transaction(create_input_token_accounts_tx)
            .await
            .unwrap();

        // Transaction 8b: Create output token accounts
        let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
        let create_output_token_accounts_tx = Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &payer.pubkey(),
                    &baryon_account.pubkey(),
                    token_rent,
                    TokenAccount::LEN as u64,
                    &spl_token_2022::id(),
                ),
                system_instruction::create_account(
                    &payer.pubkey(),
                    &photon_account.pubkey(),
                    token_rent,
                    TokenAccount::LEN as u64,
                    &spl_token_2022::id(),
                ),
            ],
            Some(&payer.pubkey()),
            &[&payer, &baryon_account, &photon_account],
            recent_blockhash,
        );
        banks_client
            .process_transaction(create_output_token_accounts_tx)
            .await
            .unwrap();

        // Transaction 9: Initialize input token accounts
        let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
        let init_input_accounts_tx = Transaction::new_signed_with_payer(
            &[
                spl_token_2022::instruction::initialize_account3(
                    &spl_token_2022::id(),
                    &anti_account.pubkey(),
                    &anti_mint.pubkey(),
                    &payer.pubkey(),
                )
                .unwrap(),
                spl_token_2022::instruction::initialize_account3(
                    &spl_token_2022::id(),
                    &pro_account.pubkey(),
                    &pro_mint.pubkey(),
                    &payer.pubkey(),
                )
                .unwrap(),
            ],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );
        banks_client
            .process_transaction(init_input_accounts_tx)
            .await
            .unwrap();

        // Transaction 10: Initialize output token accounts
        let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
        let init_output_accounts_tx = Transaction::new_signed_with_payer(
            &[
                spl_token_2022::instruction::initialize_account3(
                    &spl_token_2022::id(),
                    &baryon_account.pubkey(),
                    &baryon_mint.pubkey(),
                    &authority_pubkey,
                )
                .unwrap(),
                spl_token_2022::instruction::initialize_account3(
                    &spl_token_2022::id(),
                    &photon_account.pubkey(),
                    &photon_mint.pubkey(),
                    &authority_pubkey,
                )
                .unwrap(),
            ],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );
        banks_client
            .process_transaction(init_output_accounts_tx)
            .await
            .unwrap();

        // Transaction 11: Mint ANTI tokens
        let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
        let mint_anti_tx = Transaction::new_signed_with_payer(
            &[spl_token_2022::instruction::mint_to(
                &spl_token_2022::id(),
                &anti_mint.pubkey(),
                &anti_account.pubkey(),
                &payer.pubkey(),
                &[],
                100,
            )
            .unwrap()],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );
        banks_client
            .process_transaction(mint_anti_tx)
            .await
            .unwrap();

        // Transaction 12: Mint PRO tokens
        let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
        let mint_pro_tx = Transaction::new_signed_with_payer(
            &[spl_token_2022::instruction::mint_to(
                &spl_token_2022::id(),
                &pro_mint.pubkey(),
                &pro_account.pubkey(),
                &payer.pubkey(),
                &[],
                100,
            )
            .unwrap()],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );
        banks_client.process_transaction(mint_pro_tx).await.unwrap();

        // Transaction 13: Perform collision
        let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
        let collide_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(state_account.pubkey(), false),
                AccountMeta::new(anti_account.pubkey(), false),
                AccountMeta::new(pro_account.pubkey(), false),
                AccountMeta::new(baryon_account.pubkey(), false),
                AccountMeta::new(photon_account.pubkey(), false),
                AccountMeta::new(baryon_mint.pubkey(), false),
                AccountMeta::new(photon_mint.pubkey(), false),
                AccountMeta::new(vault_anti.pubkey(), false),
                AccountMeta::new(vault_pro.pubkey(), false),
                AccountMeta::new_readonly(anti_mint.pubkey(), false), // Required for transfer_checked
                AccountMeta::new_readonly(pro_mint.pubkey(), false), // Required for transfer_checked
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(spl_token_2022::id(), false),
                AccountMeta::new_readonly(authority_pubkey, false),
            ],
            data: CollisionInstruction::Collide {
                anti_amount: 100,
                pro_amount: 100,
            }
            .try_to_vec()
            .unwrap(),
        };

        let collide_tx = Transaction::new_signed_with_payer(
            &[collide_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );

        banks_client.process_transaction(collide_tx).await.unwrap();

        // Verify results
        let anti_account_data = banks_client
            .get_account(anti_account.pubkey())
            .await
            .unwrap()
            .unwrap();
        let pro_account_data = banks_client
            .get_account(pro_account.pubkey())
            .await
            .unwrap()
            .unwrap();
        let vault_anti_data = banks_client
            .get_account(vault_anti.pubkey())
            .await
            .unwrap()
            .unwrap();
        let vault_pro_data = banks_client
            .get_account(vault_pro.pubkey())
            .await
            .unwrap()
            .unwrap();
        let baryon_account_data = banks_client
            .get_account(baryon_account.pubkey())
            .await
            .unwrap()
            .unwrap();
        let photon_account_data = banks_client
            .get_account(photon_account.pubkey())
            .await
            .unwrap()
            .unwrap();

        let anti_token_account = TokenAccount::unpack(&anti_account_data.data).unwrap();
        let pro_token_account = TokenAccount::unpack(&pro_account_data.data).unwrap();
        let vault_anti_account = TokenAccount::unpack(&vault_anti_data.data).unwrap();
        let vault_pro_account = TokenAccount::unpack(&vault_pro_data.data).unwrap();
        let baryon_token_account = TokenAccount::unpack(&baryon_account_data.data).unwrap();
        let photon_token_account = TokenAccount::unpack(&photon_account_data.data).unwrap();

        // Verify tokens were transferred to vaults
        assert_eq!(anti_token_account.amount, 0);
        assert_eq!(pro_token_account.amount, 0);
        assert_eq!(vault_anti_account.amount, 100);
        assert_eq!(vault_pro_account.amount, 100);

        // Verify output tokens were minted
        assert!(baryon_token_account.amount > 0);
        assert!(photon_token_account.amount > 0);
    }

    #[test]
    fn test_error_conditions() {
        let mut lamports = 0;
        let mut data = vec![];

        let binding = Pubkey::new_unique();
        let user_account = AccountInfo::new(
            &binding,
            true,
            false,
            &mut lamports,
            &mut data,
            &PROGRAM_ID,
            false,
            0,
        );

        // Test zero amounts
        let accounts = vec![user_account.clone()];
        let instruction_data = CollisionInstruction::Collide {
            anti_amount: 0,
            pro_amount: 0,
        }
        .try_to_vec()
        .unwrap();

        let result = process_instruction(&PROGRAM_ID, &accounts, &instruction_data);
        assert!(result.is_err());

        // Test invalid instruction data
        let result = process_instruction(
            &PROGRAM_ID,
            &accounts,
            &[255], // Invalid instruction
        );
        assert!(result.is_err());
    }
}

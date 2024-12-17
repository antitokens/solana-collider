#[cfg(test)]
mod tests {
    use solana_collider::ID;  // Import the real program ID
    use borsh::BorshSerialize;
    use solana_collider::{
        process_instruction,
        CollisionInstruction
    };
    use solana_program::{
        account_info::AccountInfo,
        instruction::{AccountMeta, Instruction},
        program_pack::Pack,
        pubkey::Pubkey,
        system_instruction,
    };
    use solana_program_test::*;
    use solana_sdk::{
        system_program,
        signature::{Keypair, Signer},
        transaction::Transaction,
    };
    use spl_token::state::{Account as TokenAccount, Mint};

    pub const PROGRAM_ID:Pubkey = ID;

    #[tokio::test]
    async fn test_collide() {
        let mut program_test = ProgramTest::new(
            "solana_collider",
            PROGRAM_ID,  // Use the real program ID
            None,
        );

        // Add SPL Token program
        program_test.prefer_bpf(false);
        program_test.add_program(
            "spl_token",
            spl_token::id(),
            processor!(spl_token::processor::Processor::process),
        );

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        // Create all keypairs first
        let anti_mint = Keypair::new();
        let pro_mint = Keypair::new();
        let baryon_mint = Keypair::new();
        let photon_mint = Keypair::new();
        let anti_account = Keypair::new();
        let pro_account = Keypair::new();
        let baryon_account = Keypair::new();
        let photon_account = Keypair::new();

        // Get rent amounts
        let rent = banks_client.get_rent().await.unwrap();
        let mint_rent = rent.minimum_balance(Mint::LEN);
        let token_account_rent = rent.minimum_balance(TokenAccount::LEN);

        // Create mint accounts - split into two transactions
        let transaction = Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &payer.pubkey(),
                    &anti_mint.pubkey(),
                    mint_rent,
                    Mint::LEN as u64,
                    &spl_token::id(),
                ),
                system_instruction::create_account(
                    &payer.pubkey(),
                    &pro_mint.pubkey(),
                    mint_rent,
                    Mint::LEN as u64,
                    &spl_token::id(),
                ),
            ],
            Some(&payer.pubkey()),
            &[&payer, &anti_mint, &pro_mint],
            recent_blockhash,
        );
        banks_client.process_transaction(transaction).await.unwrap();

        let transaction = Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &payer.pubkey(),
                    &baryon_mint.pubkey(),
                    mint_rent,
                    Mint::LEN as u64,
                    &spl_token::id(),
                ),
                system_instruction::create_account(
                    &payer.pubkey(),
                    &photon_mint.pubkey(),
                    mint_rent,
                    Mint::LEN as u64,
                    &spl_token::id(),
                ),
            ],
            Some(&payer.pubkey()),
            &[&payer, &baryon_mint, &photon_mint],
            recent_blockhash,
        );
        banks_client.process_transaction(transaction).await.unwrap();

        // Create token accounts - split into two transactions
        let transaction = Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &payer.pubkey(),
                    &anti_account.pubkey(),
                    token_account_rent,
                    TokenAccount::LEN as u64,
                    &spl_token::id(),
                ),
                system_instruction::create_account(
                    &payer.pubkey(),
                    &pro_account.pubkey(),
                    token_account_rent,
                    TokenAccount::LEN as u64,
                    &spl_token::id(),
                ),
            ],
            Some(&payer.pubkey()),
            &[&payer, &anti_account, &pro_account],
            recent_blockhash,
        );
        banks_client.process_transaction(transaction).await.unwrap();

        let transaction = Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &payer.pubkey(),
                    &baryon_account.pubkey(),
                    token_account_rent,
                    TokenAccount::LEN as u64,
                    &spl_token::id(),
                ),
                system_instruction::create_account(
                    &payer.pubkey(),
                    &photon_account.pubkey(),
                    token_account_rent,
                    TokenAccount::LEN as u64,
                    &spl_token::id(),
                ),
            ],
            Some(&payer.pubkey()),
            &[&payer, &baryon_account, &photon_account],
            recent_blockhash,
        );
        banks_client.process_transaction(transaction).await.unwrap();

        // Initialise mints with payer as authority
        let transaction = Transaction::new_signed_with_payer(
            &[
                spl_token::instruction::initialize_mint(
                    &spl_token::id(),
                    &anti_mint.pubkey(),
                    &payer.pubkey(),
                    Some(&payer.pubkey()),
                    9,
                ).unwrap(),
                spl_token::instruction::initialize_mint(
                    &spl_token::id(),
                    &pro_mint.pubkey(),
                    &payer.pubkey(),
                    Some(&payer.pubkey()),
                    9,
                ).unwrap(),
            ],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );
        banks_client.process_transaction(transaction).await.unwrap();

        let transaction = Transaction::new_signed_with_payer(
            &[
                spl_token::instruction::initialize_mint(
                    &spl_token::id(),
                    &baryon_mint.pubkey(),
                    &payer.pubkey(),
                    Some(&payer.pubkey()),
                    9,
                ).unwrap(),
                spl_token::instruction::initialize_mint(
                    &spl_token::id(),
                    &photon_mint.pubkey(),
                    &payer.pubkey(),
                    Some(&payer.pubkey()),
                    9,
                ).unwrap(),
            ],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );
        banks_client.process_transaction(transaction).await.unwrap();

        // Initialise token accounts
        let transaction = Transaction::new_signed_with_payer(
            &[
                spl_token::instruction::initialize_account(
                    &spl_token::id(),
                    &anti_account.pubkey(),
                    &anti_mint.pubkey(),
                    &payer.pubkey(),
                ).unwrap(),
                spl_token::instruction::initialize_account(
                    &spl_token::id(),
                    &pro_account.pubkey(),
                    &pro_mint.pubkey(),
                    &payer.pubkey(),
                ).unwrap(),
            ],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );
        banks_client.process_transaction(transaction).await.unwrap();

        let transaction = Transaction::new_signed_with_payer(
            &[
                spl_token::instruction::initialize_account(
                    &spl_token::id(),
                    &baryon_account.pubkey(),
                    &baryon_mint.pubkey(),
                    &payer.pubkey(),
                ).unwrap(),
                spl_token::instruction::initialize_account(
                    &spl_token::id(),
                    &photon_account.pubkey(),
                    &photon_mint.pubkey(),
                    &payer.pubkey(),
                ).unwrap(),
            ],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );
        banks_client.process_transaction(transaction).await.unwrap();

        // Mint initial tokens
        let transaction = Transaction::new_signed_with_payer(
            &[spl_token::instruction::mint_to(
                &spl_token::id(),
                &anti_mint.pubkey(),
                &anti_account.pubkey(),
                &payer.pubkey(),
                &[],
                100,
            ).unwrap()],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );
        banks_client.process_transaction(transaction).await.unwrap();

        let transaction = Transaction::new_signed_with_payer(
            &[spl_token::instruction::mint_to(
                &spl_token::id(),
                &pro_mint.pubkey(),
                &pro_account.pubkey(),
                &payer.pubkey(),
                &[],
                100,
            ).unwrap()],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );
        banks_client.process_transaction(transaction).await.unwrap();

        // Perform collision
        let transaction = Transaction::new_signed_with_payer(
            &[Instruction {
                program_id: PROGRAM_ID,  // Use the real program ID
                accounts: vec![
                    AccountMeta::new(anti_account.pubkey(), false),
                    AccountMeta::new(pro_account.pubkey(), false),
                    AccountMeta::new(baryon_account.pubkey(), false),
                    AccountMeta::new(photon_account.pubkey(), false),
                    AccountMeta::new(baryon_mint.pubkey(), false),
                    AccountMeta::new(photon_mint.pubkey(), false),
                    AccountMeta::new(payer.pubkey(), true),
                    AccountMeta::new_readonly(system_program::id(), false),
                    AccountMeta::new_readonly(spl_token::id(), false),
                ],
                data: CollisionInstruction::Collide {
                    anti_amount: 100,
                    pro_amount: 100,
                }
                .try_to_vec()
                .unwrap(),
            }],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );

        let result = banks_client.process_transaction(transaction).await;
        assert!(result.is_ok());

        // Verify results
        let anti_account_data = banks_client.get_account(anti_account.pubkey()).await.unwrap().unwrap();
        let pro_account_data = banks_client.get_account(pro_account.pubkey()).await.unwrap().unwrap();
        let baryon_account_data = banks_client.get_account(baryon_account.pubkey()).await.unwrap().unwrap();
        let photon_account_data = banks_client.get_account(photon_account.pubkey()).await.unwrap().unwrap();
        
        let anti_token_account = TokenAccount::unpack(&anti_account_data.data).unwrap();
        let pro_token_account = TokenAccount::unpack(&pro_account_data.data).unwrap();
        let baryon_token_account = TokenAccount::unpack(&baryon_account_data.data).unwrap();
        let photon_token_account = TokenAccount::unpack(&photon_account_data.data).unwrap();

        assert_eq!(anti_token_account.amount, 0);
        assert_eq!(pro_token_account.amount, 0);
        assert!(baryon_token_account.amount > 0);
        assert!(photon_token_account.amount > 0);
    }

    #[test]
    fn test_error_conditions() {
        let mut lamports = 0;
        let mut data = vec![];
        
        // Create mock accounts for testing
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
        let accounts = vec![user_account];
        let instruction_data = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        
        let result = process_instruction(
            &PROGRAM_ID,
            &accounts,
            &instruction_data,
        );
        assert!(result.is_err());

        // Test insufficient balance
        let result = process_instruction(
            &PROGRAM_ID,
            &accounts,
            &[1, 100, 0, 0, 0, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0],
        );
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
//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

// instructions/deposit.rs
use crate::state::*;
use crate::utils::*;
use crate::DepositTokens;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

pub fn deposit(
    ctx: Context<DepositTokens>,
    index: u64,
    anti: u64,
    pro: u64,
    unix_timestamp: Option<i64>, // CRITICAL: Remove line in production!
) -> Result<()> {
    let prediction = &mut ctx.accounts.prediction;

    // Get current time, supporting local testing override
    let now = match unix_timestamp {
        Some(ts) => ts,
        None => Clock::get()?.unix_timestamp,
    }; // CRITICAL: Remove block in production!

    // CRITICAL: Add line in production!let now = Clock::get()?.unix_timestamp;

    // Verify prediction is active
    require!(prediction.is_active(now), PredictError::PredictionInactive);

    // Verify minimum deposit
    require!(
        anti >= MIN_DEPOSIT_AMOUNT || pro >= MIN_DEPOSIT_AMOUNT,
        PredictError::InsufficientDeposit
    );

    // Check prediction token account authorities
    require!(
        ctx.accounts.prediction_anti_token.owner == ANTITOKEN_MULTISIG,
        PredictError::InvalidTokenAccount
    );
    require!(
        ctx.accounts.prediction_pro_token.owner == ANTITOKEN_MULTISIG,
        PredictError::InvalidTokenAccount
    );

    // Transfer $ANTI tokens if amount > 0
    if anti > 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_anti_token.to_account_info(),
                    to: ctx.accounts.prediction_anti_token.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            anti,
        )?;
    }

    // Transfer $PRO tokens if amount > 0
    if pro > 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_pro_token.to_account_info(),
                    to: ctx.accounts.prediction_pro_token.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            pro,
        )?;
    }

    // Calculate metrics (mean and stddev values)
    let (mean, stddev) = collide(anti, pro)?;

    // Serialise and update prediction data
    let prediction_info = prediction.to_account_info();
    let mut data_prediction = prediction_info.try_borrow_mut_data()?;

    // Create deposit record
    let deposit = Deposit {
        address: ctx.accounts.authority.key(),
        anti,
        pro,
        mean,
        stddev,
        withdrawn: false,
    };

    // Update prediction state
    prediction.deposits.push(deposit);
    prediction.anti = prediction
        .anti
        .checked_add(anti)
        .ok_or(error!(PredictError::MathError))?;
    prediction.pro = prediction
        .pro
        .checked_add(pro)
        .ok_or(error!(PredictError::MathError))?;

    // Serialise updated prediction state
    let serialised_prediction = prediction.try_to_vec()?;
    data_prediction[8..8 + serialised_prediction.len()].copy_from_slice(&serialised_prediction);

    // Emit deposit event
    emit!(DepositEvent {
        index,
        address: ctx.accounts.authority.key(),
        anti,
        pro,
        mean,
        stddev,
        timestamp: now,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DepositTokensBumps;
    use anchor_lang::prelude::AccountInfo;
    use anchor_lang::solana_program::system_program;
    use anchor_lang::Discriminator;
    use anchor_spl::token::{spl_token, Mint, Token};
    use anchor_spl::token::{spl_token::state::Account as SplTokenAccount, TokenAccount};
    use solana_sdk::program_option::COption;
    use solana_sdk::program_pack::Pack;
    use std::cell::RefCell;
    use std::str::FromStr;

    // Fixed test IDs - these should be consistent across tests
    fn program_id() -> Pubkey {
        Pubkey::from_str(&PROGRAM_ID.to_string()).unwrap()
    }

    struct TestAccountData {
        key: Pubkey,
        lamports: u64,
        data: Vec<u8>,
        owner: Pubkey,
        executable: bool,
        rent_epoch: u64,
    }

    impl TestAccountData {
        fn new_mint(mint: Pubkey) -> Self {
            Self {
                key: mint,
                lamports: 1_000_000,
                data: vec![0; Mint::LEN],
                owner: spl_token::ID,
                executable: false,
                rent_epoch: 0,
            }
        }

        fn new_account_with_key_and_owner<T: AccountSerialize + AccountDeserialize + Clone>(
            key: Pubkey,
            owner: Pubkey,
        ) -> Self {
            Self {
                key,
                lamports: 1_000_000,
                data: vec![0; 8 + PredictionAccount::LEN],
                owner,
                executable: true,
                rent_epoch: 0,
            }
        }

        fn to_account_info<'a>(&'a mut self, is_signer: bool) -> AccountInfo<'a> {
            AccountInfo::new(
                &self.key,
                is_signer,
                true,
                &mut self.lamports,
                &mut self.data,
                &self.owner,
                self.executable,
                self.rent_epoch,
            )
        }

        fn new_token(key: Pubkey) -> Self {
            Self {
                key,
                lamports: 1_000_000,
                data: vec![0; 165],
                owner: spl_token::ID,
                executable: false,
                rent_epoch: 0,
            }
        }

        fn into_token_account<'a>(account_info: &'a AccountInfo<'a>) -> Account<'a, TokenAccount> {
            Account::try_from(account_info).unwrap()
        }

        fn init_state_data(&mut self, state: &StateAccount) -> Result<()> {
            self.data = vec![0; 8 + StateAccount::LEN];
            let data = self.data.as_mut_slice();

            let disc = StateAccount::discriminator();
            data[..8].copy_from_slice(&disc);

            let account_data = state.try_to_vec()?;
            data[8..8 + account_data.len()].copy_from_slice(&account_data);

            Ok(())
        }

        fn init_prediction_data(&mut self, prediction: &PredictionAccount) -> Result<()> {
            self.data = vec![0; 8 + PredictionAccount::LEN];
            let data = self.data.as_mut_slice();

            let disc = PredictionAccount::discriminator();
            data[..8].copy_from_slice(&disc);

            let account_data = prediction.try_to_vec()?;
            data[8..8 + account_data.len()].copy_from_slice(&account_data);

            Ok(())
        }

        fn init_token_account(&mut self, owner: Pubkey, mint: Pubkey) -> Result<()> {
            self.data = vec![0; TokenAccount::LEN]; // Ensure correct buffer size
            let data = self.data.as_mut_slice();

            let close_authority: COption<Pubkey> = COption::None;

            // Initialise a new SPL Token Account manually
            let token_account = SplTokenAccount {
                mint,
                owner,
                amount: 0,
                delegate: None.into(),
                state: spl_token::state::AccountState::Initialized,
                is_native: None.into(),
                delegated_amount: 0,
                close_authority,
            };

            token_account.pack_into_slice(data);

            Ok(())
        }
    }

    // Struct to hold all test accounts
    struct TestAccounts {
        pub prediction_data: TestAccountData,
        pub authority: TestAccountData,
        pub user_anti_token: TestAccountData,
        pub user_pro_token: TestAccountData,
        pub prediction_anti_token: TestAccountData,
        pub prediction_pro_token: TestAccountData,
        pub token_program: TestAccountData,
    }

    fn create_test_accounts(
        prediction_pda: Pubkey,
        anti_token_pda: Pubkey,
        pro_token_pda: Pubkey,
        program_id: Pubkey,
    ) -> TestAccounts {
        TestAccounts {
            prediction_data: TestAccountData::new_account_with_key_and_owner::<StateAccount>(
                prediction_pda,
                program_id,
            ),
            authority: TestAccountData::new_account_with_key_and_owner::<StateAccount>(
                ANTITOKEN_MULTISIG,
                program_id,
            ),
            user_anti_token: TestAccountData::new_token(Pubkey::new_unique()),
            user_pro_token: TestAccountData::new_token(Pubkey::new_unique()),
            prediction_anti_token: TestAccountData::new_token(anti_token_pda),
            prediction_pro_token: TestAccountData::new_token(pro_token_pda),

            // Correct SPL Token Program ID
            token_program: TestAccountData {
                key: spl_token::ID,       // Correct SPL Token Program ID
                lamports: 1_000_000,      // Dummy balance
                data: vec![],             // Programs don't have on-chain data in tests
                owner: Pubkey::default(), // Not owned by another program
                executable: true,         // Mark as an executable program
                rent_epoch: 0,
            },
        }
    }

    // Reusable method to create a test prediction
    fn create_test_prediction(start_time: &str, end_time: &str) -> PredictionAccount {
        PredictionAccount {
            index: 0,
            title: "Test Prediction".to_string(),
            description: "Test Description".to_string(),
            start_time: start_time.to_string(),
            end_time: end_time.to_string(),
            etc: None,
            anti: 0,
            pro: 0,
            deposits: vec![],
            equalised: false,
            equalisation: None,
        }
    }

    #[test]
    fn test_deposit() {
        /* Common Setup Begins Here */
        let program_id = program_id();

        // Create mints
        let anti_mint = TestAccountData::new_mint(ANTI_MINT_ADDRESS);
        let pro_mint = TestAccountData::new_mint(PRO_MINT_ADDRESS);

        // Test double for Clock
        thread_local! {
            static MOCK_UNIX_TIMESTAMP: RefCell<i64> = RefCell::new(1736899200); // 2025-01-15T00:00:00Z
        }

        // Initialise state account
        let root: Pubkey = Pubkey::new_unique();
        let mut state =
            TestAccountData::new_account_with_key_and_owner::<StateAccount>(root, program_id);
        state
            .init_state_data(&StateAccount {
                index: 0,
                authority: root,
            })
            .unwrap();

        // Derive PDAs and bumps
        let (prediction_pda, prediction_bump) = Pubkey::find_program_address(
            &[b"prediction", state.data[8..16].try_into().unwrap()],
            &program_id,
        );

        let (anti_token_pda, anti_token_bump) = Pubkey::find_program_address(
            &[b"anti_token", state.data[8..16].try_into().unwrap()],
            &program_id,
        );

        let (pro_token_pda, pro_token_bump) = Pubkey::find_program_address(
            &[b"pro_token", state.data[8..16].try_into().unwrap()],
            &program_id,
        );

        let mut accounts =
            create_test_accounts(prediction_pda, anti_token_pda, pro_token_pda, program_id);

        let authority_key = Pubkey::new_unique();

        // Initialise user token accounts
        accounts
            .user_anti_token
            .init_token_account(authority_key, anti_mint.key)
            .unwrap();
        accounts
            .user_pro_token
            .init_token_account(authority_key, pro_mint.key)
            .unwrap();
        // Initialise prediction token accounts
        accounts
            .prediction_anti_token
            .init_token_account(ANTITOKEN_MULTISIG, anti_mint.key)
            .unwrap();
        accounts
            .prediction_pro_token
            .init_token_account(ANTITOKEN_MULTISIG, pro_mint.key)
            .unwrap();

        assert_eq!(
            accounts.user_anti_token.data.len(),
            165,
            "Token account buffer size mismatch!"
        );
        assert!(
            !accounts.user_anti_token.data.iter().all(|&x| x == 0),
            "Token account is still uninitialised!"
        );

        // Safely convert to TokenAccount
        let binding_user_anti = accounts.user_anti_token.to_account_info(false);
        let user_anti_token = TestAccountData::into_token_account(&binding_user_anti);
        let binding_user_pro = accounts.user_pro_token.to_account_info(false);
        let user_pro_token = TestAccountData::into_token_account(&binding_user_pro);
        let binding_prediction_anti = accounts.prediction_anti_token.to_account_info(false);
        let prediction_anti_token = TestAccountData::into_token_account(&binding_prediction_anti);
        let binding_prediction_pro = accounts.prediction_pro_token.to_account_info(false);
        let prediction_pro_token = TestAccountData::into_token_account(&binding_prediction_pro);

        // Verify initialisation
        assert_eq!(user_anti_token.amount, 0);
        assert_eq!(user_pro_token.amount, 0);
        assert_eq!(prediction_anti_token.amount, 0);
        assert_eq!(prediction_pro_token.amount, 0);

        // Get account infos
        let authority_info = accounts.authority.to_account_info(true);
        let user_anti_info = accounts.user_anti_token.to_account_info(false);
        let user_pro_info = accounts.user_pro_token.to_account_info(false);
        let prediction_anti_info = accounts.prediction_anti_token.to_account_info(false);
        let prediction_pro_info = accounts.prediction_pro_token.to_account_info(false);
        let token_program_info = accounts.token_program.to_account_info(false);

        // Create and initialise the prediction account
        let prediction = create_test_prediction("2025-01-01T00:00:00Z", "2025-02-01T00:00:00Z");
        accounts
            .prediction_data
            .init_prediction_data(&prediction)
            .unwrap();

        let prediction_account_info = accounts.prediction_data.to_account_info(false);

        // Check that the buffer is correctly allocated
        assert!(
            prediction_account_info.try_borrow_data().unwrap().len() >= 8 + PredictionAccount::LEN
        );

        // Create deposit accounts
        let mut accounts = DepositTokens {
            prediction: Account::try_from(&prediction_account_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            user_anti_token: TestAccountData::into_token_account(&user_anti_info),
            user_pro_token: TestAccountData::into_token_account(&user_pro_info),
            prediction_anti_token: TestAccountData::into_token_account(&prediction_anti_info),
            prediction_pro_token: TestAccountData::into_token_account(&prediction_pro_info),
            token_program: Program::<Token>::try_from(&token_program_info).unwrap(),
        };

        // Create bumps
        let bumps = DepositTokensBumps {
            prediction: prediction_bump,
            prediction_anti_token: anti_token_bump,
            prediction_pro_token: pro_token_bump,
        };

        // Create context with bump for prediction PDA
        let ctx = Context::new(&program_id, &mut accounts, &[], bumps);
        /* Common Setup Ends Here */

        // Test deposit
        let result = deposit(ctx, 0, 50_000, 50_000, Some(1736899200));

        // If the test fails, print the error
        if result.is_err() {
            println!("Error: {:?}", result.unwrap_err());
        } else {
            assert!(result.is_ok());
        }

        // Verify prediction state updates
        let prediction_info_borrowed = prediction_account_info.try_borrow_data().unwrap();
        let updated_prediction =
            PredictionAccount::try_deserialize(&mut &prediction_info_borrowed[..]).unwrap();

        assert_eq!(updated_prediction.anti, 50_000);
        assert_eq!(updated_prediction.pro, 50_000);
        assert_eq!(updated_prediction.deposits.len(), 1);

        let deposit_record = &updated_prediction.deposits[0];
        assert_eq!(deposit_record.address, authority_info.key());
        assert_eq!(deposit_record.anti, 50_000);
        assert_eq!(deposit_record.pro, 50_000);
        assert_eq!(deposit_record.withdrawn, false);
    }

    #[test]
    fn test_deposit_validation() {
        /* Common Setup Begins Here */
        let program_id = program_id();

        // Create mints
        let anti_mint = TestAccountData::new_mint(ANTI_MINT_ADDRESS);
        let pro_mint = TestAccountData::new_mint(PRO_MINT_ADDRESS);

        // Test double for Clock
        thread_local! {
            static MOCK_UNIX_TIMESTAMP: RefCell<i64> = RefCell::new(1736899200); // 2025-01-15T00:00:00Z
        }

        // Initialise state account
        let root: Pubkey = Pubkey::new_unique();
        let mut state =
            TestAccountData::new_account_with_key_and_owner::<StateAccount>(root, program_id);
        state
            .init_state_data(&StateAccount {
                index: 0,
                authority: root,
            })
            .unwrap();

        // Derive PDAs and bumps
        let (prediction_pda, prediction_bump) = Pubkey::find_program_address(
            &[b"prediction", state.data[8..16].try_into().unwrap()],
            &program_id,
        );

        let (anti_token_pda, anti_token_bump) = Pubkey::find_program_address(
            &[b"anti_token", state.data[8..16].try_into().unwrap()],
            &program_id,
        );

        let (pro_token_pda, pro_token_bump) = Pubkey::find_program_address(
            &[b"pro_token", state.data[8..16].try_into().unwrap()],
            &program_id,
        );

        let mut accounts =
            create_test_accounts(prediction_pda, anti_token_pda, pro_token_pda, program_id);

        let authority_key = Pubkey::new_unique();

        // Initialise user token accounts
        accounts
            .user_anti_token
            .init_token_account(authority_key, anti_mint.key)
            .unwrap();
        accounts
            .user_pro_token
            .init_token_account(authority_key, pro_mint.key)
            .unwrap();
        // Initialise prediction token accounts
        accounts
            .prediction_anti_token
            .init_token_account(ANTITOKEN_MULTISIG, anti_mint.key)
            .unwrap();
        accounts
            .prediction_pro_token
            .init_token_account(ANTITOKEN_MULTISIG, pro_mint.key)
            .unwrap();

        assert_eq!(
            accounts.user_anti_token.data.len(),
            165,
            "Token account buffer size mismatch!"
        );
        assert!(
            !accounts.user_anti_token.data.iter().all(|&x| x == 0),
            "Token account is still uninitialised!"
        );

        // Safely convert to TokenAccount
        let binding_user_anti = accounts.user_anti_token.to_account_info(false);
        let user_anti_token = TestAccountData::into_token_account(&binding_user_anti);
        let binding_user_pro = accounts.user_pro_token.to_account_info(false);
        let user_pro_token = TestAccountData::into_token_account(&binding_user_pro);
        let binding_prediction_anti = accounts.prediction_anti_token.to_account_info(false);
        let prediction_anti_token = TestAccountData::into_token_account(&binding_prediction_anti);
        let binding_prediction_pro = accounts.prediction_pro_token.to_account_info(false);
        let prediction_pro_token = TestAccountData::into_token_account(&binding_prediction_pro);

        // Verify initialisation
        assert_eq!(user_anti_token.amount, 0);
        assert_eq!(user_pro_token.amount, 0);
        assert_eq!(prediction_anti_token.amount, 0);
        assert_eq!(prediction_pro_token.amount, 0);

        // Get account infos
        let authority_info = accounts.authority.to_account_info(true);
        let user_anti_info = accounts.user_anti_token.to_account_info(false);
        let user_pro_info = accounts.user_pro_token.to_account_info(false);
        let prediction_anti_info = accounts.prediction_anti_token.to_account_info(false);
        let prediction_pro_info = accounts.prediction_pro_token.to_account_info(false);
        let token_program_info = accounts.token_program.to_account_info(false);

        // Create and initialise the prediction account
        let prediction = create_test_prediction("2025-01-01T00:00:00Z", "2025-02-01T00:00:00Z");
        accounts
            .prediction_data
            .init_prediction_data(&prediction)
            .unwrap();

        let prediction_account_info = accounts.prediction_data.to_account_info(false);

        // Check that the buffer is correctly allocated
        assert!(
            prediction_account_info.try_borrow_data().unwrap().len() >= 8 + PredictionAccount::LEN
        );
        /* Common Setup Begins Here */

        // Test minimum deposit validation
        {
            // Create deposit accounts
            let mut accounts = DepositTokens {
                prediction: Account::try_from(&prediction_account_info).unwrap(),
                authority: Signer::try_from(&authority_info).unwrap(),
                user_anti_token: TestAccountData::into_token_account(&user_anti_info),
                user_pro_token: TestAccountData::into_token_account(&user_pro_info),
                prediction_anti_token: TestAccountData::into_token_account(&prediction_anti_info),
                prediction_pro_token: TestAccountData::into_token_account(&prediction_pro_info),
                token_program: Program::<Token>::try_from(&token_program_info).unwrap(),
            };

            // Create bumps
            let bumps = DepositTokensBumps {
                prediction: prediction_bump,
                prediction_anti_token: anti_token_bump,
                prediction_pro_token: pro_token_bump,
            };

            // Create context with bump for prediction PDA
            let ctx = Context::new(&program_id, &mut accounts, &[], bumps);

            let result = deposit(ctx, 0, 100, 100, Some(1736899200)); // Below MIN_DEPOSIT
            match result {
                Err(err) => assert_eq!(err, PredictError::InsufficientDeposit.into()),
                _ => panic!("Expected insufficient deposit error"),
            }
        }

        // Test invalid token account ownership
        {
            // Create an invalid token account with wrong owner
            let mut invalid_anti_token = TestAccountData::new_token(Pubkey::new_unique());
            invalid_anti_token.owner = system_program::ID; // Wrong owner
            invalid_anti_token.key = Pubkey::new_unique();

            let invalid_anti_info = invalid_anti_token.to_account_info(false);

            // Try to convert the invalid account - this should return an error
            let token_account_result = Account::<TokenAccount>::try_from(&invalid_anti_info);
            assert!(token_account_result.is_err());

            if let Err(error) = token_account_result {
                match error {
                    anchor_lang::error::Error::AnchorError(e) => {
                        let error_code: u32 = ErrorCode::AccountOwnedByWrongProgram.into();
                        assert_eq!(e.error_code_number, error_code);
                        assert!(e.compared_values.is_some());
                    }
                    _ => panic!("Expected AccountOwnedByWrongProgram error"),
                }
            }
        }
    }

    #[test]
    fn test_deposit_calculation() {
        /* Common Setup Begins Here */
        let program_id = program_id();

        // Create mints
        let anti_mint = TestAccountData::new_mint(ANTI_MINT_ADDRESS);
        let pro_mint = TestAccountData::new_mint(PRO_MINT_ADDRESS);

        // Test double for Clock
        thread_local! {
            static MOCK_UNIX_TIMESTAMP: RefCell<i64> = RefCell::new(1736899200); // 2025-01-15T00:00:00Z
        }

        // Initialise state account
        let root: Pubkey = Pubkey::new_unique();
        let mut state =
            TestAccountData::new_account_with_key_and_owner::<StateAccount>(root, program_id);
        state
            .init_state_data(&StateAccount {
                index: 0,
                authority: root,
            })
            .unwrap();

        // Derive PDAs and bumps
        let (prediction_pda, prediction_bump) = Pubkey::find_program_address(
            &[b"prediction", state.data[8..16].try_into().unwrap()],
            &program_id,
        );

        let (anti_token_pda, anti_token_bump) = Pubkey::find_program_address(
            &[b"anti_token", state.data[8..16].try_into().unwrap()],
            &program_id,
        );

        let (pro_token_pda, pro_token_bump) = Pubkey::find_program_address(
            &[b"pro_token", state.data[8..16].try_into().unwrap()],
            &program_id,
        );

        let mut accounts =
            create_test_accounts(prediction_pda, anti_token_pda, pro_token_pda, program_id);

        let authority_key = Pubkey::new_unique();

        // Initialise user token accounts
        accounts
            .user_anti_token
            .init_token_account(authority_key, anti_mint.key)
            .unwrap();
        accounts
            .user_pro_token
            .init_token_account(authority_key, pro_mint.key)
            .unwrap();
        // Initialise prediction token accounts
        accounts
            .prediction_anti_token
            .init_token_account(ANTITOKEN_MULTISIG, anti_mint.key)
            .unwrap();
        accounts
            .prediction_pro_token
            .init_token_account(ANTITOKEN_MULTISIG, pro_mint.key)
            .unwrap();

        assert_eq!(
            accounts.user_anti_token.data.len(),
            165,
            "Token account buffer size mismatch!"
        );
        assert!(
            !accounts.user_anti_token.data.iter().all(|&x| x == 0),
            "Token account is still uninitialised!"
        );

        // Safely convert to TokenAccount
        let binding_user_anti = accounts.user_anti_token.to_account_info(false);
        let user_anti_token = TestAccountData::into_token_account(&binding_user_anti);
        let binding_user_pro = accounts.user_pro_token.to_account_info(false);
        let user_pro_token = TestAccountData::into_token_account(&binding_user_pro);
        let binding_prediction_anti = accounts.prediction_anti_token.to_account_info(false);
        let prediction_anti_token = TestAccountData::into_token_account(&binding_prediction_anti);
        let binding_prediction_pro = accounts.prediction_pro_token.to_account_info(false);
        let prediction_pro_token = TestAccountData::into_token_account(&binding_prediction_pro);

        // Verify initialisation
        assert_eq!(user_anti_token.amount, 0);
        assert_eq!(user_pro_token.amount, 0);
        assert_eq!(prediction_anti_token.amount, 0);
        assert_eq!(prediction_pro_token.amount, 0);

        // Get account infos
        let authority_info = accounts.authority.to_account_info(true);
        let user_anti_info = accounts.user_anti_token.to_account_info(false);
        let user_pro_info = accounts.user_pro_token.to_account_info(false);
        let prediction_anti_info = accounts.prediction_anti_token.to_account_info(false);
        let prediction_pro_info = accounts.prediction_pro_token.to_account_info(false);
        let token_program_info = accounts.token_program.to_account_info(false);

        // Create and initialise the prediction account
        let prediction = create_test_prediction("2025-01-01T00:00:00Z", "2025-02-01T00:00:00Z");
        accounts
            .prediction_data
            .init_prediction_data(&prediction)
            .unwrap();

        let prediction_account_info = accounts.prediction_data.to_account_info(false);

        // Check that the buffer is correctly allocated
        assert!(
            prediction_account_info.try_borrow_data().unwrap().len() >= 8 + PredictionAccount::LEN
        );

        // Create deposit accounts
        let mut accounts = DepositTokens {
            prediction: Account::try_from(&prediction_account_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            user_anti_token: TestAccountData::into_token_account(&user_anti_info),
            user_pro_token: TestAccountData::into_token_account(&user_pro_info),
            prediction_anti_token: TestAccountData::into_token_account(&prediction_anti_info),
            prediction_pro_token: TestAccountData::into_token_account(&prediction_pro_info),
            token_program: Program::<Token>::try_from(&token_program_info).unwrap(),
        };

        // Create bumps
        let bumps = DepositTokensBumps {
            prediction: prediction_bump,
            prediction_anti_token: anti_token_bump,
            prediction_pro_token: pro_token_bump,
        };

        // Create context with bump for prediction PDA
        let ctx = Context::new(&program_id, &mut accounts, &[], bumps);
        /* Common Setup Ends Here */

        let anti = 70_000;
        let pro = 30_000;

        let result = deposit(ctx, 0, anti, pro, Some(1736899200));
        assert!(result.is_ok());

        let prediction_info_borrowed = prediction_account_info.try_borrow_data().unwrap();
        let updated_prediction =
            PredictionAccount::try_deserialize(&mut &prediction_info_borrowed[..]).unwrap();

        assert_eq!(updated_prediction.anti, anti);
        assert_eq!(updated_prediction.pro, pro);

        let deposit = &updated_prediction.deposits[0];
        let (expected_u, expected_s) = collide(anti, pro).unwrap();

        assert_eq!(deposit.mean, expected_u);
        assert_eq!(deposit.stddev, expected_s);
        assert_eq!(deposit.anti, anti);
        assert_eq!(deposit.pro, pro);
        assert!(!deposit.withdrawn);
        assert_eq!(deposit.address, authority_info.key());
    }
}

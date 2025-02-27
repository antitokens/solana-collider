//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

use crate::state::*;
use crate::utils::*;
use crate::EqualiseTokens;
use anchor_lang::prelude::*;

pub fn equalise(
    ctx: Context<EqualiseTokens>,
    index: u64,
    truth: Vec<u64>,
    unix_timestamp: Option<i64>, // CRITICAL: Remove line in production!
) -> Result<()> {
    let prediction = &mut ctx.accounts.prediction;

    // Verify prediction has ended
    // Get current time, supporting local testing override
    let now = match unix_timestamp {
        Some(ts) => ts,
        None => Clock::get()?.unix_timestamp,
    }; // CRITICAL: Remove block in production!

    // CRITICAL: Add line in production!let now = Clock::get()?.unix_timestamp;

    let end_time = parse_iso_timestamp(&prediction.end_time)?;
    require!(now >= end_time, PredictError::PredictionActive);

    // Validate truth values
    require!(
        truth.len() == 2 && truth.iter().all(|v| *v <= TRUTH_BASIS),
        PredictError::InvalidTruthValues
    );

    // Check if prediction not already equalised
    require!(!prediction.equalised, PredictError::AlreadyEqualised);

    // Calculate distributions and returns
    let (anti, pro) = equalise_with_truth(
        &prediction.deposits,
        prediction.anti,
        prediction.pro,
        &truth,
    )?;

    // Update prediction state with equalisation results
    prediction.equalised = true;
    prediction.equalisation = Some(Equalisation {
        anti,
        pro,
        truth: truth.clone(),
        timestamp: now,
    });

    // Get account info and serialise
    let prediction_info = prediction.to_account_info();
    let mut data = prediction_info.try_borrow_mut_data()?;
    let serialised_prediction = prediction.try_to_vec()?;
    data[8..8 + serialised_prediction.len()].copy_from_slice(&serialised_prediction);

    // Emit equalisation event
    emit!(EqualisationEvent {
        index,
        truth,
        anti: prediction.anti,
        pro: prediction.pro,
        timestamp: now,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::PROGRAM_ID;
    use crate::EqualiseTokensBumps;
    use anchor_lang::{system_program, Discriminator};
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

        fn new_authority_account(pubkey: Pubkey) -> Self {
            Self {
                key: pubkey,
                lamports: 1_000_000,
                data: vec![],
                owner: system_program::ID,
                executable: true,
                rent_epoch: 0,
            }
        }

        fn new_token() -> Self {
            Self {
                key: Pubkey::new_unique(),
                lamports: 1_000_000,
                data: vec![0; 165],
                owner: spl_token::ID,
                executable: false,
                rent_epoch: 0,
            }
        }

        fn new_token_program() -> Self {
            Self {
                key: spl_token::ID,
                lamports: 1_000_000,
                data: vec![],
                owner: Pubkey::default(),
                executable: true,
                rent_epoch: 0,
            }
        }

        fn into_token_account<'a>(account_info: &'a AccountInfo<'a>) -> Account<'a, TokenAccount> {
            Account::try_from(account_info).unwrap()
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

        // Reusable method to create a test prediction
        fn create_test_prediction(authority: Pubkey) -> PredictionAccount {
            PredictionAccount {
                index: 0,
                title: "Test Prediction".to_string(),
                description: "Test Description".to_string(),
                start_time: "2025-01-01T00:00:00Z".to_string(),
                end_time: "2025-01-02T00:00:00Z".to_string(), // Already ended
                etc: None,
                anti: 70000,
                pro: 30000,
                deposits: vec![Deposit {
                    address: authority,
                    anti: 70000,
                    pro: 30000,
                    mean: 40000,
                    stddev: 100000,
                    withdrawn: false,
                }],
                equalised: false,
                equalisation: None,
            }
        }

        // Reusable method to create an active test prediction
        fn create_active_test_prediction(authority: Pubkey) -> PredictionAccount {
            PredictionAccount {
                index: 0,
                title: "Test Prediction".to_string(),
                description: "Test Description".to_string(),
                start_time: "2025-02-01T00:00:00Z".to_string(),
                end_time: "2025-03-01T00:00:00Z".to_string(), // Still active
                etc: None,
                anti: 70000,
                pro: 30000,
                deposits: vec![Deposit {
                    address: authority,
                    anti: 70000,
                    pro: 30000,
                    mean: 40000,
                    stddev: 100000,
                    withdrawn: false,
                }],
                equalised: false,
                equalisation: None,
            }
        }

        // Reusable method to create an equalised test prediction
        fn create_equalised_test_prediction(authority: Pubkey) -> PredictionAccount {
            PredictionAccount {
                index: 0,
                title: "Test Prediction".to_string(),
                description: "Test Description".to_string(),
                start_time: "2025-01-01T00:00:00Z".to_string(),
                end_time: "2025-01-02T00:00:00Z".to_string(), // Already ended
                etc: None,
                anti: 70000,
                pro: 30000,
                deposits: vec![Deposit {
                    address: authority,
                    anti: 70000,
                    pro: 30000,
                    mean: 40000,
                    stddev: 100000,
                    withdrawn: false,
                }],
                equalised: true,
                equalisation: Some(Equalisation {
                    truth: vec![60000, 40000],
                    anti: vec![70000],
                    pro: vec![30000],
                    timestamp: 0,
                }),
            }
        }
    }

    #[test]
    fn test_equalise_success() {
        let program_id = program_id();

        // Test double for Clock
        thread_local! {
            static MOCK_UNIX_TIMESTAMP: RefCell<i64> = RefCell::new(1736899200); // 2025-01-15T00:00:00Z
        }

        // Create test accounts
        let mut prediction = TestAccountData::new_account_with_key_and_owner::<PredictionAccount>(
            Pubkey::new_unique(),
            program_id,
        );
        let mut authority = TestAccountData::new_authority_account(Pubkey::new_unique());

        // Initialise token accounts
        let mint_key = Pubkey::new_unique();
        let user_authority_key = Pubkey::new_unique();

        let mut user_anti = TestAccountData::new_token();
        let mut user_pro = TestAccountData::new_token();
        let mut prediction_anti = TestAccountData::new_token();
        let mut prediction_pro = TestAccountData::new_token();

        user_anti
            .init_token_account(user_authority_key, mint_key)
            .unwrap();
        user_pro
            .init_token_account(user_authority_key, mint_key)
            .unwrap();
        prediction_anti
            .init_token_account(Pubkey::new_unique(), mint_key)
            .unwrap();
        prediction_pro
            .init_token_account(Pubkey::new_unique(), mint_key)
            .unwrap();

        let mut token_program = TestAccountData::new_token_program();

        // Create prediction with deposits
        let prediction_data = TestAccountData::create_test_prediction(authority.key);

        // Write discriminator
        prediction.data[..8].copy_from_slice(&PredictionAccount::discriminator());

        // Serialise initial prediction data
        let serialised_prediction = prediction_data.try_to_vec().unwrap();
        prediction.data[8..8 + serialised_prediction.len()].copy_from_slice(&serialised_prediction);

        // Get account infos
        let prediction_info = prediction.to_account_info(false);
        let authority_info = authority.to_account_info(true);
        let user_anti_info = user_anti.to_account_info(false);
        let user_pro_info = user_pro.to_account_info(false);
        let prediction_anti_info = prediction_anti.to_account_info(false);
        let prediction_pro_info = prediction_pro.to_account_info(false);
        let token_program_info = token_program.to_account_info(false);

        let mut accounts = EqualiseTokens {
            prediction: Account::try_from(&prediction_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            user_anti_token: TestAccountData::into_token_account(&user_anti_info),
            user_pro_token: TestAccountData::into_token_account(&user_pro_info),
            prediction_anti_token: TestAccountData::into_token_account(&prediction_anti_info),
            prediction_pro_token: TestAccountData::into_token_account(&prediction_pro_info),
            token_program: Program::<Token>::try_from(&token_program_info).unwrap(),
        };

        let context = Context::new(&program_id, &mut accounts, &[], EqualiseTokensBumps {});

        // Test equalisation
        let truth = vec![6000, 4000]; // 60-40 split
        let result = equalise(context, 0, truth.clone(), Some(1736899200));

        // If the test fails, print the error
        if result.is_err() {
            println!("Error: {:?}", result.unwrap_err());
        } else {
            assert!(result.is_ok());
        }

        // Verify prediction state after equalisation
        let prediction_account: PredictionAccount = PredictionAccount::try_deserialize(
            &mut prediction_info.try_borrow_data().unwrap().as_ref(),
        )
        .unwrap();

        assert!(prediction_account.equalised);
        assert!(prediction_account.equalisation.is_some());

        let results = prediction_account.equalisation.unwrap();
        assert_eq!(results.truth, truth);
        assert!(!results.anti.is_empty());
        assert!(!results.pro.is_empty());
    }

    #[test]
    fn test_equalise_validation_failures() {
        let program_id = program_id();

        // Create mints
        let anti_mint = TestAccountData::new_mint(ANTI_MINT_ADDRESS);
        let pro_mint = TestAccountData::new_mint(PRO_MINT_ADDRESS);

        // Test double for Clock
        thread_local! {
            static MOCK_UNIX_TIMESTAMP: RefCell<i64> = RefCell::new(1736899200); // 2025-01-15T00:00:00Z
        }

        // Create test accounts
        let mut prediction = TestAccountData::new_account_with_key_and_owner::<PredictionAccount>(
            Pubkey::new_unique(),
            program_id,
        );
        let mut authority = TestAccountData::new_authority_account(Pubkey::new_unique());

        // Initialise token accounts
        let user = Pubkey::new_unique();
        let mut user_anti = TestAccountData::new_token();
        let mut user_pro = TestAccountData::new_token();
        let mut prediction_anti = TestAccountData::new_token();
        let mut prediction_pro = TestAccountData::new_token();

        user_anti.init_token_account(user, anti_mint.key).unwrap();
        user_pro.init_token_account(user, pro_mint.key).unwrap();
        prediction_anti
            .init_token_account(Pubkey::new_unique(), anti_mint.key)
            .unwrap();
        prediction_pro
            .init_token_account(Pubkey::new_unique(), pro_mint.key)
            .unwrap();

        let mut token_program = TestAccountData::new_token_program();

        // Test active prediction (should fail)
        {
            // Create prediction with deposits
            let prediction_data = TestAccountData::create_active_test_prediction(authority.key);

            // Write discriminator
            prediction.data[..8].copy_from_slice(&PredictionAccount::discriminator());

            // Serialise initial prediction data
            let serialised_prediction = prediction_data.try_to_vec().unwrap();
            prediction.data[8..8 + serialised_prediction.len()]
                .copy_from_slice(&serialised_prediction);

            // Get account infos
            let prediction_info = prediction.to_account_info(false);
            let authority_info = authority.to_account_info(true);
            let user_anti_info = user_anti.to_account_info(false);
            let user_pro_info = user_pro.to_account_info(false);
            let prediction_anti_info = prediction_anti.to_account_info(false);
            let prediction_pro_info = prediction_pro.to_account_info(false);
            let token_program_info = token_program.to_account_info(false);

            let mut accounts = EqualiseTokens {
                prediction: Account::try_from(&prediction_info).unwrap(),
                authority: Signer::try_from(&authority_info).unwrap(),
                user_anti_token: TestAccountData::into_token_account(&user_anti_info),
                user_pro_token: TestAccountData::into_token_account(&user_pro_info),
                prediction_anti_token: TestAccountData::into_token_account(&prediction_anti_info),
                prediction_pro_token: TestAccountData::into_token_account(&prediction_pro_info),
                token_program: Program::<Token>::try_from(&token_program_info).unwrap(),
            };

            let ctx = Context::new(&program_id, &mut accounts, &[], EqualiseTokensBumps {});
            let truth = vec![6000, 4000];
            let result = equalise(ctx, 0, truth, Some(1736899200));
            match result {
                Err(err) => assert_eq!(err, PredictError::PredictionActive.into()),
                _ => panic!("Expected prediction active error"),
            }
        }

        // Test invalid truth values
        {
            // Create prediction with deposits
            let prediction_data = TestAccountData::create_test_prediction(authority.key);

            // Write discriminator
            prediction.data[..8].copy_from_slice(&PredictionAccount::discriminator());

            // Serialise initial prediction data
            let serialised_prediction = prediction_data.try_to_vec().unwrap();
            prediction.data[8..8 + serialised_prediction.len()]
                .copy_from_slice(&serialised_prediction);

            // Get account infos
            let prediction_info = prediction.to_account_info(false);
            let authority_info = authority.to_account_info(true);
            let user_anti_info = user_anti.to_account_info(false);
            let user_pro_info = user_pro.to_account_info(false);
            let prediction_anti_info = prediction_anti.to_account_info(false);
            let prediction_pro_info = prediction_pro.to_account_info(false);
            let token_program_info = token_program.to_account_info(false);

            let mut accounts = EqualiseTokens {
                prediction: Account::try_from(&prediction_info).unwrap(),
                authority: Signer::try_from(&authority_info).unwrap(),
                user_anti_token: TestAccountData::into_token_account(&user_anti_info),
                user_pro_token: TestAccountData::into_token_account(&user_pro_info),
                prediction_anti_token: TestAccountData::into_token_account(&prediction_anti_info),
                prediction_pro_token: TestAccountData::into_token_account(&prediction_pro_info),
                token_program: Program::<Token>::try_from(&token_program_info).unwrap(),
            };

            let ctx = Context::new(&program_id, &mut accounts, &[], EqualiseTokensBumps {});
            let invalid_truth = vec![50_000_000, 5_000_000_000];
            let result = equalise(ctx, 0, invalid_truth, Some(1736899200));
            match result {
                Err(err) => assert_eq!(err, PredictError::InvalidTruthValues.into()),
                _ => panic!("Expected invalid truth values error"),
            }
        }

        // Test already equalised prediction
        {
            // Create prediction with deposits
            let prediction_data = TestAccountData::create_equalised_test_prediction(authority.key);

            // Write discriminator
            prediction.data[..8].copy_from_slice(&PredictionAccount::discriminator());

            // Serialise initial prediction data
            let serialised_prediction = prediction_data.try_to_vec().unwrap();
            prediction.data[8..8 + serialised_prediction.len()]
                .copy_from_slice(&serialised_prediction);

            // Get account infos
            let prediction_info = prediction.to_account_info(false);
            let authority_info = authority.to_account_info(true);
            let user_anti_info = user_anti.to_account_info(false);
            let user_pro_info = user_pro.to_account_info(false);
            let prediction_anti_info = prediction_anti.to_account_info(false);
            let prediction_pro_info = prediction_pro.to_account_info(false);
            let token_program_info = token_program.to_account_info(false);

            let mut accounts = EqualiseTokens {
                prediction: Account::try_from(&prediction_info).unwrap(),
                authority: Signer::try_from(&authority_info).unwrap(),
                user_anti_token: TestAccountData::into_token_account(&user_anti_info),
                user_pro_token: TestAccountData::into_token_account(&user_pro_info),
                prediction_anti_token: TestAccountData::into_token_account(&prediction_anti_info),
                prediction_pro_token: TestAccountData::into_token_account(&prediction_pro_info),
                token_program: Program::<Token>::try_from(&token_program_info).unwrap(),
            };

            let ctx = Context::new(&program_id, &mut accounts, &[], EqualiseTokensBumps {});
            let truth = vec![6000, 4000];
            let result = equalise(ctx, 0, truth, Some(1736899200));
            match result {
                Err(err) => assert_eq!(err, PredictError::AlreadyEqualised.into()),
                _ => panic!("Expected already equalised error"),
            }
        }
    }
}

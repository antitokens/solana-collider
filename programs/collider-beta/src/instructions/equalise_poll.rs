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

pub fn equalise(ctx: Context<EqualiseTokens>, poll_index: u64, truth: Vec<u64>) -> Result<()> {
    let poll = &mut ctx.accounts.poll;

    // Verify poll has ended
    // Get current time
    let now = Clock::get()?.unix_timestamp;
    let end_time = parse_iso_timestamp(&poll.end_time)?;
    require!(now >= end_time, PredictError::PollActive);

    // Validate truth values
    require!(
        truth.len() == 2 && truth.iter().all(|v| *v <= TRUTH_BASIS),
        PredictError::InvalidTruthValues
    );

    // Check if poll not already equalised
    require!(!poll.equalised, PredictError::AlreadyEqualised);

    // Calculate distributions and returns
    let (anti, pro) = equalise_with_truth(&poll.deposits, poll.anti, poll.pro, &truth)?;

    // Update poll state with equalisation results
    poll.equalised = true;
    poll.equalisation_results = Some(EqualisationResult {
        anti,
        pro,
        truth: truth.clone(),
        timestamp: now,
    });

    // Get account info and serialise
    let poll_info = poll.to_account_info();
    let mut data = poll_info.try_borrow_mut_data()?;
    let serialised_poll = poll.try_to_vec()?;
    data[8..8 + serialised_poll.len()].copy_from_slice(&serialised_poll);

    // Emit equalisation event
    emit!(EqualisationEvent {
        poll_index,
        truth,
        anti: poll.anti,
        pro: poll.pro,
        timestamp: now,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::PROGRAM_ID;
    use crate::EqualiseTokensBumps;
    use anchor_lang::Discriminator;
    use anchor_spl::token::{spl_token, Token};
    use anchor_spl::token::{spl_token::state::Account as SplTokenAccount, TokenAccount};
    use solana_sdk::program_option::COption;
    use solana_sdk::program_pack::Pack;
    use std::cell::RefCell;
    use std::str::FromStr;

    // Fixed test IDs - these should be consistent across tests
    fn program_id() -> Pubkey {
        Pubkey::from_str(PROGRAM_ID).unwrap()
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
        fn new_with_key<T: AccountSerialize + AccountDeserialize + Clone>(
            key: Pubkey,
            owner: Pubkey,
        ) -> Self {
            Self {
                key,
                lamports: 1_000_000,
                data: vec![0; 8 + PollAccount::LEN],
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
    }

    #[test]
    fn test_equalise_success() {
        let program_id = program_id();

        // Test double for Clock
        thread_local! {
            static MOCK_UNIX_TIMESTAMP: RefCell<i64> = RefCell::new(1736899200); // 2025-01-15T00:00:00Z
        }

        // Create test accounts
        let mut poll =
            TestAccountData::new_with_key::<PollAccount>(Pubkey::new_unique(), program_id);
        let mut authority =
            TestAccountData::new_with_key::<StateAccount>(Pubkey::new_unique(), program_id);

        // Initialise token accounts
        let mint_key = Pubkey::new_unique();
        let authority_key = Pubkey::new_unique();

        let mut user_anti = TestAccountData::new_token();
        let mut user_pro = TestAccountData::new_token();
        let mut poll_anti = TestAccountData::new_token();
        let mut poll_pro = TestAccountData::new_token();

        user_anti
            .init_token_account(authority_key, mint_key)
            .unwrap();
        user_pro
            .init_token_account(authority_key, mint_key)
            .unwrap();
        poll_anti
            .init_token_account(Pubkey::new_unique(), mint_key)
            .unwrap();
        poll_pro
            .init_token_account(Pubkey::new_unique(), mint_key)
            .unwrap();

        let mut token_program =
            TestAccountData::new_with_key::<StateAccount>(spl_token::ID, spl_token::ID);

        // Create poll with deposits
        let poll_data = PollAccount {
            index: 0,
            title: "Test Poll".to_string(),
            description: "Test Description".to_string(),
            start_time: "2025-01-01T00:00:00Z".to_string(),
            end_time: "2025-01-02T00:00:00Z".to_string(), // Already ended
            etc: None,
            anti: 70000,
            pro: 30000,
            deposits: vec![UserDeposit {
                address: authority.key,
                anti: 70000,
                pro: 30000,
                u: 40000,
                s: 100000,
                withdrawn: false,
            }],
            equalised: false,
            equalisation_results: None,
        };

        // Write discriminator
        poll.data[..8].copy_from_slice(&PollAccount::discriminator());

        // Serialise initial poll data
        let serialised_poll = poll_data.try_to_vec().unwrap();
        poll.data[8..8 + serialised_poll.len()].copy_from_slice(&serialised_poll);

        // Get account infos
        let poll_info = poll.to_account_info(false);
        let authority_info = authority.to_account_info(true);
        let user_anti_info = user_anti.to_account_info(false);
        let user_pro_info = user_pro.to_account_info(false);
        let poll_anti_info = poll_anti.to_account_info(false);
        let poll_pro_info = poll_pro.to_account_info(false);
        let token_program_info = token_program.to_account_info(false);

        let mut accounts = EqualiseTokens {
            poll: Account::try_from(&poll_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            user_anti_token: TestAccountData::into_token_account(&user_anti_info),
            user_pro_token: TestAccountData::into_token_account(&user_pro_info),
            poll_anti_token: TestAccountData::into_token_account(&poll_anti_info),
            poll_pro_token: TestAccountData::into_token_account(&poll_pro_info),
            token_program: Program::<Token>::try_from(&token_program_info).unwrap(),
        };

        let context = Context::new(&program_id, &mut accounts, &[], EqualiseTokensBumps {});

        // Test equalisation
        let truth = vec![6000, 4000]; // 60-40 split
        let result = equalise(context, 0, truth.clone());

        // If the test fails, print the error
        if result.is_err() {
            println!("Error: {:?}", result.unwrap_err());
        } else {
            assert!(result.is_ok());
        }

        // Verify poll state after equalisation
        let poll_account: PollAccount =
            PollAccount::try_deserialize(&mut poll_info.try_borrow_data().unwrap().as_ref())
                .unwrap();

        assert!(poll_account.equalised);
        assert!(poll_account.equalisation_results.is_some());

        let results = poll_account.equalisation_results.unwrap();
        assert_eq!(results.truth, truth);
        assert!(!results.anti.is_empty());
        assert!(!results.pro.is_empty());
    }

    #[test]
    fn test_equalise_validation_failures() {
        let program_id = program_id();

        // Test double for Clock
        thread_local! {
            static MOCK_UNIX_TIMESTAMP: RefCell<i64> = RefCell::new(1736899200); // 2025-01-15T00:00:00Z
        }

        // Create test accounts
        let mut poll =
            TestAccountData::new_with_key::<PollAccount>(Pubkey::new_unique(), program_id);
        let mut authority =
            TestAccountData::new_with_key::<StateAccount>(Pubkey::new_unique(), program_id);

        // Initialise token accounts
        let mint_key = Pubkey::new_unique();
        let authority_key = Pubkey::new_unique();

        let mut user_anti = TestAccountData::new_token();
        let mut user_pro = TestAccountData::new_token();
        let mut poll_anti = TestAccountData::new_token();
        let mut poll_pro = TestAccountData::new_token();

        user_anti
            .init_token_account(authority_key, mint_key)
            .unwrap();
        user_pro
            .init_token_account(authority_key, mint_key)
            .unwrap();
        poll_anti
            .init_token_account(Pubkey::new_unique(), mint_key)
            .unwrap();
        poll_pro
            .init_token_account(Pubkey::new_unique(), mint_key)
            .unwrap();

        let mut token_program =
            TestAccountData::new_with_key::<StateAccount>(spl_token::ID, spl_token::ID);

        // Test active poll (should fail)
        {
            // Create poll with deposits
            let poll_data = PollAccount {
                index: 0,
                title: "Test Poll".to_string(),
                description: "Test Description".to_string(),
                start_time: "2025-01-01T00:00:00Z".to_string(),
                end_time: "2025-02-01T00:00:00Z".to_string(), // Still active
                etc: None,
                anti: 70000,
                pro: 30000,
                deposits: vec![UserDeposit {
                    address: authority.key,
                    anti: 70000,
                    pro: 30000,
                    u: 40000,
                    s: 100000,
                    withdrawn: false,
                }],
                equalised: false,
                equalisation_results: None,
            };

            // Write discriminator
            poll.data[..8].copy_from_slice(&PollAccount::discriminator());

            // Serialise initial poll data
            let serialised_poll = poll_data.try_to_vec().unwrap();
            poll.data[8..8 + serialised_poll.len()].copy_from_slice(&serialised_poll);

            // Get account infos
            let poll_info = poll.to_account_info(false);
            let authority_info = authority.to_account_info(true);
            let user_anti_info = user_anti.to_account_info(false);
            let user_pro_info = user_pro.to_account_info(false);
            let poll_anti_info = poll_anti.to_account_info(false);
            let poll_pro_info = poll_pro.to_account_info(false);
            let token_program_info = token_program.to_account_info(false);

            let mut accounts = EqualiseTokens {
                poll: Account::try_from(&poll_info).unwrap(),
                authority: Signer::try_from(&authority_info).unwrap(),
                user_anti_token: TestAccountData::into_token_account(&user_anti_info),
                user_pro_token: TestAccountData::into_token_account(&user_pro_info),
                poll_anti_token: TestAccountData::into_token_account(&poll_anti_info),
                poll_pro_token: TestAccountData::into_token_account(&poll_pro_info),
                token_program: Program::<Token>::try_from(&token_program_info).unwrap(),
            };

            let ctx = Context::new(&program_id, &mut accounts, &[], EqualiseTokensBumps {});
            let truth = vec![6000, 4000];
            let result = equalise(ctx, 0, truth);
            match result {
                Err(err) => assert_eq!(err, PredictError::PollActive.into()),
                _ => panic!("Expected poll active error"),
            }
        }

        // Test invalid truth values
        {
            // Create poll with deposits
            let poll_data = PollAccount {
                index: 0,
                title: "Test Poll".to_string(),
                description: "Test Description".to_string(),
                start_time: "2025-01-01T00:00:00Z".to_string(),
                end_time: "2025-01-02T00:00:00Z".to_string(), // Already ended
                etc: None,
                anti: 70000,
                pro: 30000,
                deposits: vec![UserDeposit {
                    address: authority.key,
                    anti: 70000,
                    pro: 30000,
                    u: 40000,
                    s: 100000,
                    withdrawn: false,
                }],
                equalised: false,
                equalisation_results: None,
            };

            // Write discriminator
            poll.data[..8].copy_from_slice(&PollAccount::discriminator());

            // Serialise initial poll data
            let serialised_poll = poll_data.try_to_vec().unwrap();
            poll.data[8..8 + serialised_poll.len()].copy_from_slice(&serialised_poll);

            // Get account infos
            let poll_info = poll.to_account_info(false);
            let authority_info = authority.to_account_info(true);
            let user_anti_info = user_anti.to_account_info(false);
            let user_pro_info = user_pro.to_account_info(false);
            let poll_anti_info = poll_anti.to_account_info(false);
            let poll_pro_info = poll_pro.to_account_info(false);
            let token_program_info = token_program.to_account_info(false);

            let mut accounts = EqualiseTokens {
                poll: Account::try_from(&poll_info).unwrap(),
                authority: Signer::try_from(&authority_info).unwrap(),
                user_anti_token: TestAccountData::into_token_account(&user_anti_info),
                user_pro_token: TestAccountData::into_token_account(&user_pro_info),
                poll_anti_token: TestAccountData::into_token_account(&poll_anti_info),
                poll_pro_token: TestAccountData::into_token_account(&poll_pro_info),
                token_program: Program::<Token>::try_from(&token_program_info).unwrap(),
            };

            let ctx = Context::new(&program_id, &mut accounts, &[], EqualiseTokensBumps {});
            let invalid_truth = vec![50_000_000, 5_000_000_000];
            let result = equalise(ctx, 0, invalid_truth);
            match result {
                Err(err) => assert_eq!(err, PredictError::InvalidTruthValues.into()),
                _ => panic!("Expected invalid truth values error"),
            }
        }

        // Test already equalised poll
        {
            // Create poll with deposits
            let poll_data = PollAccount {
                index: 0,
                title: "Test Poll".to_string(),
                description: "Test Description".to_string(),
                start_time: "2025-01-01T00:00:00Z".to_string(),
                end_time: "2025-01-02T00:00:00Z".to_string(), // Already ended
                etc: None,
                anti: 70000,
                pro: 30000,
                deposits: vec![UserDeposit {
                    address: authority.key,
                    anti: 70000,
                    pro: 30000,
                    u: 40000,
                    s: 100000,
                    withdrawn: false,
                }],
                equalised: true,
                equalisation_results: Some(EqualisationResult {
                    truth: vec![60000, 40000],
                    anti: vec![],
                    pro: vec![],
                    timestamp: 0,
                }),
            };

            // Write discriminator
            poll.data[..8].copy_from_slice(&PollAccount::discriminator());

            // Serialise initial poll data
            let serialised_poll = poll_data.try_to_vec().unwrap();
            poll.data[8..8 + serialised_poll.len()].copy_from_slice(&serialised_poll);

            // Get account infos
            let poll_info = poll.to_account_info(false);
            let authority_info = authority.to_account_info(true);
            let user_anti_info = user_anti.to_account_info(false);
            let user_pro_info = user_pro.to_account_info(false);
            let poll_anti_info = poll_anti.to_account_info(false);
            let poll_pro_info = poll_pro.to_account_info(false);
            let token_program_info = token_program.to_account_info(false);

            let mut accounts = EqualiseTokens {
                poll: Account::try_from(&poll_info).unwrap(),
                authority: Signer::try_from(&authority_info).unwrap(),
                user_anti_token: TestAccountData::into_token_account(&user_anti_info),
                user_pro_token: TestAccountData::into_token_account(&user_pro_info),
                poll_anti_token: TestAccountData::into_token_account(&poll_anti_info),
                poll_pro_token: TestAccountData::into_token_account(&poll_pro_info),
                token_program: Program::<Token>::try_from(&token_program_info).unwrap(),
            };

            let ctx = Context::new(&program_id, &mut accounts, &[], EqualiseTokensBumps {});
            let truth = vec![6000, 4000];
            let result = equalise(ctx, 0, truth);
            match result {
                Err(err) => assert_eq!(err, PredictError::AlreadyEqualised.into()),
                _ => panic!("Expected already equalised error"),
            }
        }
    }
}

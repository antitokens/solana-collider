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
    let clock = Clock::get()?;
    let end_time = parse_iso_timestamp(&poll.end_time)?;
    require!(clock.unix_timestamp >= end_time, PredictError::PollEnded);

    // Validate truth values
    require!(
        truth.len() == 2 && truth.iter().all(|v| *v <= BASIS_POINTS),
        PredictError::InvalidTruthValues
    );

    // Calculate distributions and returns
    let (anti, pro) = equalise_with_truth(&poll.deposits, poll.anti, poll.pro, &truth)?;

    // Update poll state with equalisation results
    poll.equalised = true;
    poll.equalisation_results = Some(EqualisationResult {
        anti,
        pro,
        truth: truth.clone(),
        timestamp: clock.unix_timestamp,
    });

    // Emit equalisation event
    emit!(EqualisationEvent {
        poll_index,
        truth,
        anti: poll.anti,
        pro: poll.pro,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EqualiseTokensBumps;
    use anchor_lang::solana_program::system_program;
    use anchor_spl::token::spl_token;
    use anchor_spl::token::{Token, TokenAccount};
    use std::str::FromStr;

    struct TestAccountData {
        key: Pubkey,
        lamports: u64,
        data: Vec<u8>,
        owner: Pubkey,
        executable: bool,
        rent_epoch: u64,
    }

    impl TestAccountData {
        fn new_owned(owner: Pubkey) -> Self {
            Self {
                key: Pubkey::new_unique(),
                lamports: 1_000_000,
                data: vec![0; 1000],
                owner,
                executable: false,
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

        fn into_token_account<'a>(account_info: &'a AccountInfo<'a>) -> Account<'a, TokenAccount> {
            Account::try_from(account_info).unwrap()
        }
    }

    #[test]
    fn test_equalise_success() {
        let program_id = Pubkey::from_str("5eR98MdgS8jYpKB2iD9oz3MtBdLJ6s7gAVWJZFMvnL9G").unwrap();

        // Create test accounts
        let mut poll = TestAccountData::new_owned(program_id);
        let mut authority = TestAccountData::new_owned(system_program::ID);
        let mut user_anti = TestAccountData::new_owned(program_id);
        let mut user_pro = TestAccountData::new_owned(program_id);
        let mut poll_anti = TestAccountData::new_owned(program_id);
        let mut poll_pro = TestAccountData::new_owned(program_id);
        let mut token_program = TestAccountData::new_owned(spl_token::ID);

        // Create poll with deposits
        let poll_data = PollAccount {
            index: 0,
            title: "Test Poll".to_string(),
            description: "Test Description".to_string(),
            start_time: "2025-01-01T00:00:00Z".to_string(),
            end_time: "2025-01-02T00:00:00Z".to_string(), // Already ended
            etc: None,
            anti: 7000,
            pro: 3000,
            deposits: vec![UserDeposit {
                address: authority.key,
                anti: 7000,
                pro: 3000,
                u: 4000,
                s: 10000,
                withdrawn: false,
            }],
            equalised: false,
            equalisation_results: None,
        };

        // Get account infos
        let poll_info = poll.to_account_info(false);
        let authority_info = authority.to_account_info(true);
        let user_anti_info = user_anti.to_account_info(false);
        let user_pro_info = user_pro.to_account_info(false);
        let poll_anti_info = poll_anti.to_account_info(false);
        let poll_pro_info = poll_pro.to_account_info(false);
        let token_program_info = token_program.to_account_info(false);

        // Serialize poll data
        poll_info
            .try_borrow_mut_data()
            .unwrap()
            .copy_from_slice(&poll_data.try_to_vec().unwrap());

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

        assert!(result.is_ok());

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
        let program_id = Pubkey::from_str("5eR98MdgS8jYpKB2iD9oz3MtBdLJ6s7gAVWJZFMvnL9G").unwrap();

        // Create test accounts
        let mut poll = TestAccountData::new_owned(program_id);
        let mut authority = TestAccountData::new_owned(system_program::ID);
        let mut user_anti = TestAccountData::new_owned(program_id);
        let mut user_pro = TestAccountData::new_owned(program_id);
        let mut poll_anti = TestAccountData::new_owned(program_id);
        let mut poll_pro = TestAccountData::new_owned(program_id);
        let mut token_program = TestAccountData::new_owned(spl_token::ID);

        // Create active poll
        let poll_data = PollAccount {
            index: 0,
            title: "Test Poll".to_string(),
            description: "Test Description".to_string(),
            start_time: "2025-01-01T00:00:00Z".to_string(),
            end_time: "2025-12-31T00:00:00Z".to_string(), // Not ended yet
            etc: None,
            anti: 7000,
            pro: 3000,
            deposits: vec![],
            equalised: false,
            equalisation_results: None,
        };

        // Get account infos
        let poll_info = poll.to_account_info(false);
        let authority_info = authority.to_account_info(true);
        let user_anti_info = user_anti.to_account_info(false);
        let user_pro_info = user_pro.to_account_info(false);
        let poll_anti_info = poll_anti.to_account_info(false);
        let poll_pro_info = poll_pro.to_account_info(false);
        let token_program_info = token_program.to_account_info(false);

        // Serialize poll data
        poll_info
            .try_borrow_mut_data()
            .unwrap()
            .copy_from_slice(&poll_data.try_to_vec().unwrap());

        let mut accounts = EqualiseTokens {
            poll: Account::try_from(&poll_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            user_anti_token: TestAccountData::into_token_account(&user_anti_info),
            user_pro_token: TestAccountData::into_token_account(&user_pro_info),
            poll_anti_token: TestAccountData::into_token_account(&poll_anti_info),
            poll_pro_token: TestAccountData::into_token_account(&poll_pro_info),
            token_program: Program::<Token>::try_from(&token_program_info).unwrap(),
        };

        // Test active poll (should fail)
        {
            let ctx = Context::new(&program_id, &mut accounts, &[], EqualiseTokensBumps {});

            let truth = vec![6000, 4000];
            let result = equalise(ctx, 0, truth);
            match result {
                Err(err) => assert_eq!(err, Error::from(PredictError::PollInactive)),
                _ => panic!("Expected poll active error"),
            }
        }

        // Test empty deposits
        {
            let ctx = Context::new(&program_id, &mut accounts, &[], EqualiseTokensBumps {});

            let truth = vec![6000, 4000];
            let result = equalise(ctx, 0, truth);
            match result {
                Err(err) => assert_eq!(err, Error::from(PredictError::NoDeposits)),
                _ => panic!("Expected no deposits error"),
            }
        }

        // Test invalid truth values
        {
            let ctx = Context::new(&program_id, &mut accounts, &[], EqualiseTokensBumps {});

            let invalid_truth = vec![5000, 5000];
            let result = equalise(ctx, 0, invalid_truth);
            match result {
                Err(err) => assert_eq!(err, Error::from(PredictError::InvalidTruthValues)),
                _ => panic!("Expected invalid truth values error"),
            }
        }

        // Test already equalised poll
        {
            // Update poll data to be equalised
            let mut equalised_poll_data = poll_data.clone();
            equalised_poll_data.equalised = true;
            equalised_poll_data.equalisation_results = Some(EqualisationResult {
                truth: vec![6000, 4000],
                anti: vec![],
                pro: vec![],
                timestamp: 0,
            });

            poll_info
                .try_borrow_mut_data()
                .unwrap()
                .copy_from_slice(&equalised_poll_data.try_to_vec().unwrap());

            let ctx = Context::new(&program_id, &mut accounts, &[], EqualiseTokensBumps {});

            let truth = vec![6000, 4000];
            let result = equalise(ctx, 0, truth);
            match result {
                Err(err) => assert_eq!(err, Error::from(PredictError::AlreadyEqualised)),
                _ => panic!("Expected already equalised error"),
            }
        }
    }
}

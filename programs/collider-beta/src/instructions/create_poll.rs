//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 0.0.1
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

// instructions/create_poll.rs
use crate::utils::*;
use crate::CreatePoll;
use anchor_lang::prelude::*;

pub fn create(
    ctx: Context<CreatePoll>,
    title: String,
    description: String,
    start_time: String,
    end_time: String,
    etc: Option<Vec<u8>>,
) -> Result<()> {
    // Verify payment
    require!(
        ctx.accounts.payment.lamports() >= 100000000, // 0.1 SOL in lamports
        PredictError::InsufficientPayment
    );

    // Validate string lengths
    require!(title.len() <= MAX_TITLE_LENGTH, PredictError::TitleTooLong);
    require!(
        description.len() <= MAX_DESC_LENGTH,
        PredictError::DescriptionTooLong
    );

    // Validate title uniqueness
    require!(
        !state_has_title(&ctx.accounts.state, &title),
        PredictError::TitleExists
    );

    // Validate timestamps
    let start = parse_iso_timestamp(&start_time)?;
    let end = parse_iso_timestamp(&end_time)?;
    let now = Clock::get()?.unix_timestamp;

    require!(end > start, PredictError::InvalidTimeRange);
    require!(start > now, PredictError::StartTimeInPast);

    let poll = &mut ctx.accounts.poll;
    let state = &mut ctx.accounts.state;

    // Initialize poll data
    poll.index = state.poll_index;
    poll.title = title.clone();
    poll.description = description;
    poll.start_time = start_time.clone();
    poll.end_time = end_time.clone();
    poll.etc = etc;
    poll.anti = 0;
    poll.pro = 0;
    poll.deposits = vec![];
    poll.equalised = false;
    poll.equalisation_results = None;

    state.poll_index += 1;

    // Emit event
    emit!(PollCreatedEvent {
        poll_index: poll.index,
        address: ctx.accounts.authority.key(),
        title,
        start_time,
        end_time,
        timestamp: now,
    });

    Ok(())
}

// Add instruction data structs
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreatePollArgs {
    pub title: String,
    pub description: String,
    pub start_time: String,
    pub end_time: String,
    pub etc: Option<Vec<u8>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CreatePollBumps;
    use crate::PollAccount;
    use crate::StateAccount;
    use anchor_lang::solana_program::system_program;
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
        fn new_owned(owner: Pubkey, lamports: u64) -> Self {
            Self {
                key: Pubkey::new_unique(),
                lamports,
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
    }

    #[test]
    fn test_create_poll_success() {
        let program_id = Pubkey::from_str("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS").unwrap();

        // Create test accounts
        let mut state = TestAccountData::new_owned(program_id, 1_000_000);
        let mut poll = TestAccountData::new_owned(program_id, 1_000_000);
        let mut payment = TestAccountData::new_owned(system_program::ID, 200_000_000); // 0.2 SOL
        let mut authority = TestAccountData::new_owned(system_program::ID, 1_000_000);
        let mut system = TestAccountData::new_owned(system_program::ID, 1_000_000);

        // Initialize state account
        let state_data = StateAccount {
            poll_index: 0,
            authority: authority.key,
        };
        state.data[0..state_data.try_to_vec().unwrap().len()]
            .copy_from_slice(&state_data.try_to_vec().unwrap());

        // Get account infos
        let state_info = state.to_account_info(false);
        let poll_info = poll.to_account_info(false);
        let payment_info = payment.to_account_info(false);
        let authority_info = authority.to_account_info(true);
        let system_info = system.to_account_info(false);

        let mut accounts = CreatePoll {
            state: Account::try_from(&state_info).unwrap(),
            poll: Account::try_from(&poll_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            payment: payment_info,
            system_program: Program::try_from(&system_info).unwrap(),
        };

        let result = create(
            Context::new(
                &program_id,
                &mut accounts,
                &[],
                CreatePollBumps { poll: 255 },
            ),
            "Test Poll".to_string(),
            "Test Description".to_string(),
            "2025-02-01T00:00:00Z".to_string(),
            "2025-02-02T00:00:00Z".to_string(),
            None,
        );

        assert!(result.is_ok());

        // Verify poll state
        let poll_account: PollAccount =
            PollAccount::try_deserialize(&mut poll_info.try_borrow_data().unwrap().as_ref())
                .unwrap();

        assert_eq!(poll_account.index, 0);
        assert_eq!(poll_account.title, "Test Poll");
        assert_eq!(poll_account.description, "Test Description");
        assert_eq!(poll_account.start_time, "2025-02-01T00:00:00Z");
        assert_eq!(poll_account.end_time, "2025-02-02T00:00:00Z");
        assert_eq!(poll_account.anti, 0);
        assert_eq!(poll_account.pro, 0);
        assert!(poll_account.deposits.is_empty());
        assert!(!poll_account.equalised);
        assert!(poll_account.equalisation_results.is_none());

        // Verify state update
        let state_account: StateAccount =
            StateAccount::try_deserialize(&mut state_info.try_borrow_data().unwrap().as_ref())
                .unwrap();
        assert_eq!(state_account.poll_index, 1);
    }

    #[test]
    fn test_create_poll_validation_failures() {
        let program_id = Pubkey::from_str("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS").unwrap();

        // Create test accounts
        let mut state = TestAccountData::new_owned(program_id, 1_000_000);
        let mut poll = TestAccountData::new_owned(program_id, 1_000_000);
        let mut payment = TestAccountData::new_owned(system_program::ID, 50_000_000); // 0.05 SOL (insufficient)
        let mut authority = TestAccountData::new_owned(system_program::ID, 1_000_000);
        let mut system = TestAccountData::new_owned(system_program::ID, 1_000_000);

        // Get account infos
        let state_info = state.to_account_info(false);
        let poll_info = poll.to_account_info(false);
        let payment_info = payment.to_account_info(false);
        let authority_info = authority.to_account_info(true);
        let system_info = system.to_account_info(false);

        let mut accounts = CreatePoll {
            state: Account::try_from(&state_info).unwrap(),
            poll: Account::try_from(&poll_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            payment: payment_info,
            system_program: Program::try_from(&system_info).unwrap(),
        };

        // Test insufficient payment
        {
            let result = create(
                Context::new(
                    &program_id,
                    &mut accounts,
                    &[],
                    CreatePollBumps { poll: 255 },
                ),
                "Test Poll".to_string(),
                "Test Description".to_string(),
                "2025-02-01T00:00:00Z".to_string(),
                "2025-02-02T00:00:00Z".to_string(),
                None,
            );
            match result {
                Err(err) => assert_eq!(err, Error::from(PredictError::InsufficientPayment)),
                _ => panic!("Expected insufficient payment error"),
            }
        }

        // Test title too long
        {
            let long_title = "a".repeat(MAX_TITLE_LENGTH + 1);
            let result = create(
                Context::new(
                    &program_id,
                    &mut accounts,
                    &[],
                    CreatePollBumps { poll: 255 },
                ),
                long_title,
                "Test Description".to_string(),
                "2025-02-01T00:00:00Z".to_string(),
                "2025-02-02T00:00:00Z".to_string(),
                None,
            );
            match result {
                Err(err) => assert_eq!(err, Error::from(PredictError::TitleTooLong)),
                _ => panic!("Expected title too long error"),
            }
        }

        // Test description too long
        {
            let long_desc = "a".repeat(MAX_DESC_LENGTH + 1);
            let result = create(
                Context::new(
                    &program_id,
                    &mut accounts,
                    &[],
                    CreatePollBumps { poll: 255 },
                ),
                "Test Poll".to_string(),
                long_desc,
                "2025-02-01T00:00:00Z".to_string(),
                "2025-02-02T00:00:00Z".to_string(),
                None,
            );
            match result {
                Err(err) => assert_eq!(err, Error::from(PredictError::DescriptionTooLong)),
                _ => panic!("Expected description too long error"),
            }
        }

        // Test invalid time range
        {
            let result = create(
                Context::new(
                    &program_id,
                    &mut accounts,
                    &[],
                    CreatePollBumps { poll: 255 },
                ),
                "Test Poll".to_string(),
                "Test Description".to_string(),
                "2025-02-02T00:00:00Z".to_string(), // End before start
                "2025-02-01T00:00:00Z".to_string(),
                None,
            );
            match result {
                Err(err) => assert_eq!(err, Error::from(PredictError::InvalidTimeRange)),
                _ => panic!("Expected invalid time range error"),
            }
        }

        // Test start time in past
        {
            let result = create(
                Context::new(
                    &program_id,
                    &mut accounts,
                    &[],
                    CreatePollBumps { poll: 255 },
                ),
                "Test Poll".to_string(),
                "Test Description".to_string(),
                "2024-01-01T00:00:00Z".to_string(), // Past date
                "2025-02-01T00:00:00Z".to_string(),
                None,
            );
            match result {
                Err(err) => assert_eq!(err, Error::from(PredictError::StartTimeInPast)),
                _ => panic!("Expected start time in past error"),
            }
        }
    }
}

//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 1.0.0-beta
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
    unix_timestamp: Option<i64>, // CRITICAL: Remove in production
) -> Result<()> {
    // Ensure payment is sufficient
    require!(
        ctx.accounts.payment.lamports() >= 100_000_000,
        PredictError::InsufficientPayment
    );

    // Validate title and description lengths
    require!(title.len() <= MAX_TITLE_LENGTH, PredictError::TitleTooLong);
    require!(
        description.len() <= MAX_DESCRIPTION_LENGTH,
        PredictError::DescriptionTooLong
    );

    // Ensure the title is unique
    require!(
        !state_has_title(&ctx.accounts.state, &title),
        PredictError::TitleExists
    );

    // Parse and validate time ranges
    let start = parse_iso_timestamp(&start_time)?;
    let end = parse_iso_timestamp(&end_time)?;
    let now = match unix_timestamp {
        Some(ts) => ts,
        None => Clock::get()?.unix_timestamp,
    }; // CRITICAL: Remove in production
    //let now = Clock::get()?.unix_timestamp;

    require!(end > start, PredictError::InvalidTimeRange);
    require!(start > now, PredictError::StartTimeInPast);

    // Initialise the poll account
    let poll = &mut ctx.accounts.poll;
    let state = &mut ctx.accounts.state;

    let poll_info = poll.to_account_info();
    let state_info = state.to_account_info();
    let mut data_poll = poll_info.try_borrow_mut_data()?;
    let mut data_state = state_info.try_borrow_mut_data()?;

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

    let serialised_poll = poll.try_to_vec()?;
    data_poll[8..8 + serialised_poll.len()].copy_from_slice(&serialised_poll);

    // Increment poll index
    state.poll_index += 1;

    let serialised_state = state.try_to_vec()?;
    data_state[8..8 + serialised_state.len()].copy_from_slice(&serialised_state);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CreatePollBumps;
    use crate::{PollAccount, StateAccount};
    use anchor_lang::system_program;
    use anchor_lang::Discriminator;
    use std::cell::RefCell;
    use std::str::FromStr;

    // Fixed test IDs - these should be consistent across tests
    fn program_id() -> Pubkey {
        Pubkey::from_str("5eR98MdgS8jYpKB2iD9oz3MtBdLJ6s7gAVWJZFMvnL9G").unwrap()
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

        fn init_state_data(&mut self, state: &StateAccount) -> Result<()> {
            self.data = vec![0; 8 + StateAccount::LEN];
            let data = self.data.as_mut_slice();

            let disc = StateAccount::discriminator();
            data[..8].copy_from_slice(&disc);

            let account_data = state.try_to_vec()?;
            data[8..8 + account_data.len()].copy_from_slice(&account_data);

            Ok(())
        }

        fn init_poll_data(&mut self, poll: &PollAccount) -> Result<()> {
            self.data = vec![0; 8 + PollAccount::LEN];
            let data = self.data.as_mut_slice();

            let disc = PollAccount::discriminator();
            data[..8].copy_from_slice(&disc);

            let account_data = poll.try_to_vec()?;
            data[8..8 + account_data.len()].copy_from_slice(&account_data);

            Ok(())
        }
    }

    #[test]
    fn test_create_poll_success() -> Result<()> {
        let program_id = program_id();

        // Test double for Clock
        thread_local! {
            static MOCK_UNIX_TIMESTAMP: RefCell<i64> = RefCell::new(1736899200); // 2025-01-15T00:00:00Z
        }

        // Initialise state account
        let mut state =
            TestAccountData::new_with_key::<StateAccount>(Pubkey::new_unique(), program_id);
        state
            .init_state_data(&StateAccount {
                poll_index: 0,
                authority: Pubkey::new_unique(),
            })
            .unwrap();

        // Derive PDA and bump for poll account
        let (poll_pda, poll_bump) = Pubkey::find_program_address(
            &[b"poll", state.data[8..16].try_into().unwrap()],
            &program_id,
        );

        let mut poll = TestAccountData::new_with_key::<PollAccount>(poll_pda, program_id);
        poll.init_poll_data(&PollAccount::default()).unwrap();

        // Initialise payment account
        let mut payment = TestAccountData {
            key: Pubkey::new_unique(),
            lamports: 200_000_000,
            data: vec![],
            owner: system_program::ID,
            executable: false,
            rent_epoch: 0,
        };

        // Initialise other accounts
        let mut authority =
            TestAccountData::new_with_key::<StateAccount>(Pubkey::new_unique(), program_id);
        let mut system_program =
            TestAccountData::new_with_key::<StateAccount>(system_program::ID, system_program::ID);

        // Prepare account infos
        let state_info = state.to_account_info(true);
        let poll_info = poll.to_account_info(true);
        let payment_info = payment.to_account_info(false);
        let authority_info = authority.to_account_info(true);
        let system_info = system_program.to_account_info(false);

        // Set up CreatePoll context
        let mut accounts = CreatePoll {
            state: Account::try_from(&state_info).unwrap(),
            poll: Account::try_from(&poll_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            payment: payment_info.clone(),
            system_program: Program::try_from(&system_info).unwrap(),
        };

        // Include the CreatePollBumps with the bump for the poll account
        let bumps = CreatePollBumps { poll: poll_bump };

        // Call the create function
        let result = create(
            Context::new(&program_id, &mut accounts, &[], bumps),
            "Test Poll".to_string(),
            "Test Description".to_string(),
            "2025-02-01T00:00:00Z".to_string(),
            "2025-02-02T00:00:00Z".to_string(),
            None,
            Some(1736899200),
        );

        // If the test fails, print the error
        if result.is_err() {
            println!("Error: {:?}", result.unwrap_err());
        } else {
            assert!(result.is_ok());
        }

        // Verify poll data
        let poll_info_borrowed = poll_info.try_borrow_data()?;
        let poll_account = PollAccount::try_deserialize(&mut &poll_info_borrowed[..])?;

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
        Ok(())
    }

    #[test]
    fn test_create_poll_with_insufficient_payment() -> Result<()> {
        let program_id = program_id();

        // Test double for Clock
        thread_local! {
            static MOCK_UNIX_TIMESTAMP: RefCell<i64> = RefCell::new(1736899200); // 2025-01-15T00:00:00Z
        }

        // Initialise state account
        let mut state =
            TestAccountData::new_with_key::<StateAccount>(Pubkey::new_unique(), program_id);
        state
            .init_state_data(&StateAccount {
                poll_index: 0,
                authority: Pubkey::new_unique(),
            })
            .unwrap();

        // Derive PDA and bump for poll account
        let (poll_pda, poll_bump) = Pubkey::find_program_address(
            &[b"poll", state.data[8..16].try_into().unwrap()],
            &program_id,
        );

        let mut poll = TestAccountData::new_with_key::<PollAccount>(poll_pda, program_id);
        poll.init_poll_data(&PollAccount::default()).unwrap();

        // Initialise payment account
        let mut payment = TestAccountData {
            key: Pubkey::new_unique(),
            lamports: 50_000_000,
            data: vec![],
            owner: system_program::ID,
            executable: false,
            rent_epoch: 0,
        };

        // Initialise other accounts
        let mut authority =
            TestAccountData::new_with_key::<StateAccount>(Pubkey::new_unique(), program_id);
        let mut system_program =
            TestAccountData::new_with_key::<StateAccount>(system_program::ID, system_program::ID);

        // Prepare account infos
        let state_info = state.to_account_info(true);
        let poll_info = poll.to_account_info(true);
        let payment_info = payment.to_account_info(false);
        let authority_info = authority.to_account_info(true);
        let system_info = system_program.to_account_info(false);

        // Set up CreatePoll context
        let mut accounts = CreatePoll {
            state: Account::try_from(&state_info).unwrap(),
            poll: Account::try_from(&poll_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            payment: payment_info.clone(),
            system_program: Program::try_from(&system_info).unwrap(),
        };

        // Test insufficient payment
        {
            let bumps = CreatePollBumps { poll: poll_bump };
            let result = create(
                Context::new(&program_id, &mut accounts, &[], bumps),
                "Test Poll".to_string(),
                "Test Description".to_string(),
                "2025-02-01T00:00:00Z".to_string(),
                "2025-02-02T00:00:00Z".to_string(),
                None,
                Some(1736899200),
            );
            assert_eq!(
                result.unwrap_err(),
                Error::from(PredictError::InsufficientPayment)
            );
        }
        Ok(())
    }

    #[test]
    fn test_create_poll_with_title_and_description_too_long() -> Result<()> {
        let program_id = program_id();

        // Test double for Clock
        thread_local! {
            static MOCK_UNIX_TIMESTAMP: RefCell<i64> = RefCell::new(1736899200); // 2025-01-15T00:00:00Z
        }

        // Initialise state account
        let mut state =
            TestAccountData::new_with_key::<StateAccount>(Pubkey::new_unique(), program_id);
        state
            .init_state_data(&StateAccount {
                poll_index: 0,
                authority: Pubkey::new_unique(),
            })
            .unwrap();

        // Derive PDA and bump for poll account
        let (poll_pda, poll_bump) = Pubkey::find_program_address(
            &[b"poll", state.data[8..16].try_into().unwrap()],
            &program_id,
        );

        let mut poll = TestAccountData::new_with_key::<PollAccount>(poll_pda, program_id);
        poll.init_poll_data(&PollAccount::default()).unwrap();

        // Initialise payment account
        let mut payment = TestAccountData {
            key: Pubkey::new_unique(),
            lamports: 200_000_000,
            data: vec![],
            owner: system_program::ID,
            executable: false,
            rent_epoch: 0,
        };

        // Initialise other accounts
        let mut authority =
            TestAccountData::new_with_key::<StateAccount>(Pubkey::new_unique(), program_id);
        let mut system_program =
            TestAccountData::new_with_key::<StateAccount>(system_program::ID, system_program::ID);

        // Prepare account infos
        let state_info = state.to_account_info(true);
        let poll_info = poll.to_account_info(true);
        let payment_info = payment.to_account_info(false);
        let authority_info = authority.to_account_info(true);
        let system_info = system_program.to_account_info(false);

        // Set up CreatePoll context
        let mut accounts = CreatePoll {
            state: Account::try_from(&state_info).unwrap(),
            poll: Account::try_from(&poll_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            payment: payment_info.clone(),
            system_program: Program::try_from(&system_info).unwrap(),
        };

        // Test title too long
        {
            let bumps = CreatePollBumps { poll: poll_bump };
            let long_title = "a".repeat(MAX_TITLE_LENGTH + 1);
            let result = create(
                Context::new(&program_id, &mut accounts, &[], bumps),
                long_title,
                "Test Description".to_string(),
                "2025-02-01T00:00:00Z".to_string(),
                "2025-02-02T00:00:00Z".to_string(),
                None,
                Some(1736899200),
            );
            assert_eq!(result.unwrap_err(), Error::from(PredictError::TitleTooLong));
        }

        // Test description too long
        {
            let bumps = CreatePollBumps { poll: poll_bump };
            let long_description = "a".repeat(MAX_DESCRIPTION_LENGTH + 1);
            let result = create(
                Context::new(&program_id, &mut accounts, &[], bumps),
                "Test Poll".to_string(),
                long_description,
                "2025-02-01T00:00:00Z".to_string(),
                "2025-02-02T00:00:00Z".to_string(),
                None,
                Some(1736899200),
            );
            assert_eq!(
                result.unwrap_err(),
                Error::from(PredictError::DescriptionTooLong)
            );
        }

        Ok(())
    }

    #[test]
    fn test_create_poll_with_bad_schedule() -> Result<()> {
        let program_id = program_id();

        // Test double for Clock
        thread_local! {
            static MOCK_UNIX_TIMESTAMP: RefCell<i64> = RefCell::new(1736899200); // 2025-01-15T00:00:00Z
        }

        // Initialise state account
        let mut state =
            TestAccountData::new_with_key::<StateAccount>(Pubkey::new_unique(), program_id);
        state
            .init_state_data(&StateAccount {
                poll_index: 0,
                authority: Pubkey::new_unique(),
            })
            .unwrap();

        // Derive PDA and bump for poll account
        let (poll_pda, poll_bump) = Pubkey::find_program_address(
            &[b"poll", state.data[8..16].try_into().unwrap()],
            &program_id,
        );

        let mut poll = TestAccountData::new_with_key::<PollAccount>(poll_pda, program_id);
        poll.init_poll_data(&PollAccount::default()).unwrap();

        // Initialise payment account
        let mut payment = TestAccountData {
            key: Pubkey::new_unique(),
            lamports: 200_000_000,
            data: vec![],
            owner: system_program::ID,
            executable: false,
            rent_epoch: 0,
        };

        // Initialise other accounts
        let mut authority =
            TestAccountData::new_with_key::<StateAccount>(Pubkey::new_unique(), program_id);
        let mut system_program =
            TestAccountData::new_with_key::<StateAccount>(system_program::ID, system_program::ID);

        // Prepare account infos
        let state_info = state.to_account_info(true);
        let poll_info = poll.to_account_info(true);
        let payment_info = payment.to_account_info(false);
        let authority_info = authority.to_account_info(true);
        let system_info = system_program.to_account_info(false);

        // Set up CreatePoll context
        let mut accounts = CreatePoll {
            state: Account::try_from(&state_info).unwrap(),
            poll: Account::try_from(&poll_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            payment: payment_info.clone(),
            system_program: Program::try_from(&system_info).unwrap(),
        };

        // Test invalid time range
        {
            let bumps = CreatePollBumps { poll: poll_bump };
            let result = create(
                Context::new(&program_id, &mut accounts, &[], bumps),
                "Test Poll".to_string(),
                "Test Description".to_string(),
                "2025-02-02T00:00:00Z".to_string(), // End before start
                "2025-02-01T00:00:00Z".to_string(),
                None,
                Some(1736899200),
            );
            assert_eq!(
                result.unwrap_err(),
                Error::from(PredictError::InvalidTimeRange)
            );
        }

        // Test start time in past
        {
            let bumps = CreatePollBumps { poll: poll_bump };
            let result = create(
                Context::new(&program_id, &mut accounts, &[], bumps),
                "Test Poll".to_string(),
                "Test Description".to_string(),
                "2024-01-01T00:00:00Z".to_string(), // Past date
                "2025-02-01T00:00:00Z".to_string(),
                None,
                Some(1736899200),
            );
            assert_eq!(
                result.unwrap_err(),
                Error::from(PredictError::StartTimeInPast)
            );
        }

        Ok(())
    }
}

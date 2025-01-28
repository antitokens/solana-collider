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
    unix_timestamp: Option<i64>,
) -> Result<()> {
    require!(
        ctx.accounts.payment.lamports() >= 100000000,
        PredictError::InsufficientPayment
    );

    require!(title.len() <= MAX_TITLE_LENGTH, PredictError::TitleTooLong);
    require!(
        description.len() <= MAX_DESC_LENGTH,
        PredictError::DescriptionTooLong
    );

    require!(
        !state_has_title(&ctx.accounts.state, &title),
        PredictError::TitleExists
    );

    let start = parse_iso_timestamp(&start_time)?;
    let end = parse_iso_timestamp(&end_time)?;
    let now = match unix_timestamp {
        Some(ts) => ts,
        None => Clock::get()?.unix_timestamp,
    };

    require!(end > start, PredictError::InvalidTimeRange);
    require!(start > now, PredictError::StartTimeInPast);

    let poll = &mut ctx.accounts.poll;
    let state = &mut ctx.accounts.state;

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
    use anchor_lang::system_program;
    use anchor_lang::Discriminator;
    use std::cell::RefCell;
    use std::str::FromStr;

    // Test double for Clock
    thread_local! {
        static MOCK_UNIX_TIMESTAMP: RefCell<i64> = RefCell::new(1705276800); // 2025-01-15T00:00:00Z
    }

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
        fn new_owned<T: AccountSerialize + AccountDeserialize + Clone>(owner: Pubkey) -> Self {
            Self {
                key: system_program::ID,
                lamports: 1_000_000,
                data: vec![0; 8 + std::mem::size_of::<T>()], // Account for the 8-byte discriminator that Anchor adds
                owner,
                executable: true,
                rent_epoch: 0,
            }
        }

        fn new_system_account() -> Self {
            Self {
                key: system_program::ID,
                lamports: 1_000_000,
                data: vec![],
                owner: program_id(),
                executable: true,
                rent_epoch: 0,
            }
        }

        fn new_payment(lamports: u64) -> Self {
            Self {
                key: system_program::ID,
                lamports,
                data: vec![],
                owner: system_program::ID,
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

        fn init_state_data(&mut self, state: &StateAccount) -> Result<()> {
            let data = self.data.as_mut_slice();

            // Write discriminator
            let disc = StateAccount::discriminator();
            data[..8].copy_from_slice(&disc);

            // Write account data
            let account_data = state.try_to_vec()?;
            data[8..8 + account_data.len()].copy_from_slice(&account_data);

            Ok(())
        }

        fn init_poll_data(&mut self, poll: &PollAccount) -> Result<()> {
            let data = self.data.as_mut_slice();

            // Write discriminator
            let disc = PollAccount::discriminator();
            data[..8].copy_from_slice(&disc);

            // Write account data
            let account_data = poll.try_to_vec()?;
            data[8..8 + account_data.len()].copy_from_slice(&account_data);

            Ok(())
        }
    }

    #[test]
    fn test_create_poll_success() {
        let program_id = program_id();

        // Create test accounts
        let mut state = TestAccountData::new_owned::<StateAccount>(program_id);
        let mut poll = TestAccountData::new_owned::<PollAccount>(program_id);
        let mut payment = TestAccountData::new_payment(200_000_000);
        let mut authority = TestAccountData::new_system_account();
        let mut system = TestAccountData::new_system_account();

        // Initialise state account
        let state_data = StateAccount {
            poll_index: 0,
            authority: authority.key,
        };
        state.init_state_data(&state_data).unwrap();

        // Initialise poll account
        let poll_data = PollAccount {
            index: 0,
            title: String::new(),
            description: String::new(),
            start_time: String::new(),
            end_time: String::new(),
            etc: None,
            anti: 0,
            pro: 0,
            deposits: vec![],
            equalised: false,
            equalisation_results: None,
        };
        poll.init_poll_data(&poll_data).unwrap();

        // Get account infos
        let state_info = state.to_account_info(true);
        let poll_info = poll.to_account_info(true);
        let payment_info = payment.to_account_info(false);
        let authority_info = authority.to_account_info(true);
        let system_info = system.to_account_info(false);

        let mut accounts = CreatePoll {
            state: Account::try_from(&state_info).unwrap(),
            poll: Account::try_from(&poll_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            payment: payment_info.clone(),
            system_program: Program::try_from(&system_info).unwrap(),
        };

        let result = create(
            Context::new(&program_id, &mut accounts, &[], CreatePollBumps { poll: 0 }),
            "Test Poll".to_string(),
            "Test Description".to_string(),
            "2025-02-01T00:00:00Z".to_string(),
            "2025-02-02T00:00:00Z".to_string(),
            None,
            Some(1705276800),
        );

        // If the test fails, print the error
        if result.is_err() {
            println!("Error: {:?}", result.unwrap_err());
        } else {
            assert!(result.is_ok());
        }

        // Get updated poll data
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
        let program_id = program_id();

        // Create test accounts
        let mut state = TestAccountData::new_owned::<StateAccount>(program_id);
        let mut poll = TestAccountData::new_owned::<PollAccount>(program_id);
        let mut payment = TestAccountData::new_payment(200_000_000);
        let mut authority = TestAccountData::new_system_account();
        let mut system = TestAccountData::new_system_account();

        // Initialise state account
        let state_data = StateAccount {
            poll_index: 0,
            authority: authority.key,
        };
        state.init_state_data(&state_data).unwrap();

        // Initialise poll account
        let poll_data = PollAccount {
            index: 0,
            title: "".to_string(),
            description: "".to_string(),
            start_time: "".to_string(),
            end_time: "".to_string(),
            etc: None,
            anti: 0,
            pro: 0,
            deposits: vec![],
            equalised: false,
            equalisation_results: None,
        };
        poll.init_poll_data(&poll_data).unwrap();

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
            payment: payment_info.clone(),
            system_program: Program::try_from(&system_info).unwrap(),
        };

        // Test insufficient payment
        {
            let result = create(
                Context::new(&program_id, &mut accounts, &[], CreatePollBumps { poll: 0 }),
                "Test Poll".to_string(),
                "Test Description".to_string(),
                "2025-02-01T00:00:00Z".to_string(),
                "2025-02-02T00:00:00Z".to_string(),
                None,
                Some(1705276800),
            );
            assert_eq!(
                result.unwrap_err(),
                Error::from(PredictError::InsufficientPayment)
            );
        }

        // Test title too long
        {
            let long_title = "a".repeat(MAX_TITLE_LENGTH + 1);
            let result = create(
                Context::new(&program_id, &mut accounts, &[], CreatePollBumps { poll: 0 }),
                long_title,
                "Test Description".to_string(),
                "2025-02-01T00:00:00Z".to_string(),
                "2025-02-02T00:00:00Z".to_string(),
                None,
                Some(1705276800),
            );
            assert_eq!(result.unwrap_err(), Error::from(PredictError::TitleTooLong));
        }

        // Test description too long
        {
            let long_desc = "a".repeat(MAX_DESC_LENGTH + 1);
            let result = create(
                Context::new(&program_id, &mut accounts, &[], CreatePollBumps { poll: 0 }),
                "Test Poll".to_string(),
                long_desc,
                "2025-02-01T00:00:00Z".to_string(),
                "2025-02-02T00:00:00Z".to_string(),
                None,
                Some(1705276800),
            );
            assert_eq!(
                result.unwrap_err(),
                Error::from(PredictError::DescriptionTooLong)
            );
        }

        // Test invalid time range
        {
            let result = create(
                Context::new(&program_id, &mut accounts, &[], CreatePollBumps { poll: 0 }),
                "Test Poll".to_string(),
                "Test Description".to_string(),
                "2025-02-02T00:00:00Z".to_string(), // End before start
                "2025-02-01T00:00:00Z".to_string(),
                None,
                Some(1705276800),
            );
            assert_eq!(
                result.unwrap_err(),
                Error::from(PredictError::InvalidTimeRange)
            );
        }

        // Test start time in past
        {
            let result = create(
                Context::new(&program_id, &mut accounts, &[], CreatePollBumps { poll: 0 }),
                "Test Poll".to_string(),
                "Test Description".to_string(),
                "2024-01-01T00:00:00Z".to_string(), // Past date
                "2025-02-01T00:00:00Z".to_string(),
                None,
                Some(1705276800),
            );
            assert_eq!(
                result.unwrap_err(),
                Error::from(PredictError::StartTimeInPast)
            );
        }
    }
}

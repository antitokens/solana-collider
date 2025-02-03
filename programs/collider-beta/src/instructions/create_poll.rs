//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 02 Feb 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

// instructions/create_poll.rs
use crate::utils::*;
use crate::CreatePoll;
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token;
use anchor_spl::token::spl_token::instruction::AuthorityType;
use anchor_spl::token::SetAuthority;

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
        ctx.accounts.authority.lamports() >= POLL_CREATION_FEE,
        PredictError::InsufficientPayment
    );

    // Validate title and description lengths
    require!(
        title.len() <= MAX_TITLE_LENGTH as usize,
        PredictError::TitleTooLong
    );
    require!(
        description.len() <= MAX_DESCRIPTION_LENGTH as usize,
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
    }; // CRITICAL: Remove in production!
       // CRITICAL: Add in production!let now = Clock::get()?.unix_timestamp;

    require!(end > start, PredictError::InvalidTimeRange);
    require!(start > now, PredictError::StartTimeInPast);

    // Transfer payment to state account
    let payment_amount = POLL_CREATION_FEE;
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.authority.to_account_info(),
                to: ctx.accounts.state.to_account_info(),
            },
        ),
        payment_amount,
    )?;

    // Set the token account authority to ANTITOKEN_MULTISIG using token instruction
    let cpi_accounts = SetAuthority {
        account_or_mint: ctx.accounts.poll_anti_token.to_account_info(),
        current_authority: ctx.accounts.authority.to_account_info(),
    };

    token::set_authority(
        CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
        AuthorityType::AccountOwner,
        Some(ANTITOKEN_MULTISIG),
    )?;

    let cpi_accounts = SetAuthority {
        account_or_mint: ctx.accounts.poll_pro_token.to_account_info(),
        current_authority: ctx.accounts.authority.to_account_info(),
    };

    token::set_authority(
        CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
        AuthorityType::AccountOwner,
        Some(ANTITOKEN_MULTISIG),
    )?;

    // Get account infos for manual serialisation
    let state_info = &ctx.accounts.state.to_account_info();
    let poll_info = &ctx.accounts.poll.to_account_info();
    let mut data_state = state_info.try_borrow_mut_data()?;
    let mut data_poll = poll_info.try_borrow_mut_data()?;

    // Set poll data
    ctx.accounts.poll.index = ctx.accounts.state.poll_index;
    ctx.accounts.poll.title = title.clone();
    ctx.accounts.poll.description = description;
    ctx.accounts.poll.start_time = start_time.clone();
    ctx.accounts.poll.end_time = end_time.clone();
    ctx.accounts.poll.etc = etc;
    ctx.accounts.poll.anti = 0;
    ctx.accounts.poll.pro = 0;
    ctx.accounts.poll.deposits = vec![];
    ctx.accounts.poll.equalised = false;
    ctx.accounts.poll.equalisation_results = None;

    // Manual serialisation
    let serialised_poll = ctx.accounts.poll.try_to_vec()?;
    data_poll[8..8 + serialised_poll.len()].copy_from_slice(&serialised_poll);

    // Increment poll index
    ctx.accounts.state.poll_index += 1;

    // Manual serialisation for state
    let serialised_state = ctx.accounts.state.try_to_vec()?;
    data_state[8..8 + serialised_state.len()].copy_from_slice(&serialised_state);

    // Emit event
    emit!(PollCreatedEvent {
        poll_index: ctx.accounts.poll.index,
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
    use crate::utils::PROGRAM_ID;
    use crate::CreatePollBumps;
    use crate::{PollAccount, StateAccount};
    use anchor_lang::system_program;
    use anchor_lang::Discriminator;
    use anchor_spl::token::{
        spl_token, spl_token::state::Account as SplTokenAccount, Mint, TokenAccount,
    };
    use solana_program::program_pack::Pack;
    use solana_sdk::program_option::COption;
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
        fn new_account_with_key_and_owner<T: AccountSerialize + AccountDeserialize + Clone>(
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

        fn new_mint_address(mint: Pubkey) -> Self {
            Self {
                key: mint,
                lamports: 1_000_000,
                data: vec![0; Mint::LEN],
                owner: spl_token::ID,
                executable: false,
                rent_epoch: 0,
            }
        }

        fn new_system_account() -> Self {
            Self {
                key: system_program::ID,
                lamports: 1_000_000,
                data: vec![],
                owner: system_program::ID,
                executable: true,
                rent_epoch: 0,
            }
        }

        fn new_token_account(key: Pubkey) -> Self {
            Self {
                key,
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

        fn new_vault_with_key() -> Self {
            Self {
                key: ANTITOKEN_MULTISIG,
                lamports: 10_000_000,
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

        fn init_rent_account() -> Self {
            Self {
                key: anchor_lang::solana_program::sysvar::rent::ID,
                lamports: 1_000_000,
                data: vec![0; 32], // Minimal rent sysvar data
                owner: system_program::ID,
                executable: false,
                rent_epoch: 0,
            }
        }
    }

    #[test]
    fn test_create_poll_success() -> Result<()> {
        /* Common Setup Begins Here */
        let program_id = program_id();

        // Create token program account
        let mut token_program = TestAccountData::new_token_program();

        // Create mints
        let mut anti_mint = TestAccountData::new_mint_address(ANTI_MINT_ADDRESS);
        let mut pro_mint = TestAccountData::new_mint_address(PRO_MINT_ADDRESS);

        // Test double for Clock
        thread_local! {
            static MOCK_UNIX_TIMESTAMP: RefCell<i64> = RefCell::new(1736899200); // 2025-01-15T00:00:00Z
        }

        // Initialise state account
        let manager: Pubkey = Pubkey::new_unique();
        let mut state =
            TestAccountData::new_account_with_key_and_owner::<StateAccount>(manager, program_id);
        state
            .init_state_data(&StateAccount {
                poll_index: 0,
                authority: manager,
            })
            .unwrap();

        // Derive PDAs and bumps
        let (_state_pda, state_bump) = Pubkey::find_program_address(&[b"state"], &program_id);

        let (poll_pda, poll_bump) = Pubkey::find_program_address(
            &[b"poll", state.data[8..16].try_into().unwrap()],
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

        let mut poll =
            TestAccountData::new_account_with_key_and_owner::<PollAccount>(poll_pda, program_id);
        poll.init_poll_data(&PollAccount::default()).unwrap();

        // Initialise creator account
        let mut creator = TestAccountData {
            key: Pubkey::new_unique(),
            lamports: 200_000_000,
            data: vec![],
            owner: system_program::ID,
            executable: false,
            rent_epoch: 0,
        };

        // Create token accounts
        let mut poll_anti_token = TestAccountData::new_token_account(anti_token_pda);
        let mut poll_pro_token = TestAccountData::new_token_account(pro_token_pda);

        // Rent for accounts
        let mut rent_account = TestAccountData::init_rent_account();

        // Initialise token accounts
        poll_anti_token
            .init_token_account(ANTITOKEN_MULTISIG, anti_mint.key)
            .unwrap();
        poll_pro_token
            .init_token_account(ANTITOKEN_MULTISIG, pro_mint.key)
            .unwrap();

        // Initialise other accounts
        let mut system_program = TestAccountData::new_system_account();
        let mut vault = TestAccountData::new_vault_with_key();

        // Prepare account infos
        let state_info = state.to_account_info(false);
        let poll_info = poll.to_account_info(false);
        let authority_info = creator.to_account_info(true);
        let system_info = system_program.to_account_info(false);
        let anti_mint_info = anti_mint.to_account_info(false);
        let pro_mint_info = pro_mint.to_account_info(false);
        let poll_anti_token_info = poll_anti_token.to_account_info(false);
        let poll_pro_token_info = poll_pro_token.to_account_info(false);
        let token_program_info = token_program.to_account_info(false);
        let rent_account_info = rent_account.to_account_info(false);
        let vault_info = vault.to_account_info(false);

        // Set up CreatePoll context
        let mut accounts = CreatePoll {
            state: Account::try_from(&state_info).unwrap(),
            poll: Account::try_from(&poll_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            poll_anti_token: Account::try_from(&poll_anti_token_info).unwrap(),
            poll_pro_token: Account::try_from(&poll_pro_token_info).unwrap(),
            anti_mint: anti_mint_info.clone(),
            pro_mint: pro_mint_info.clone(),
            vault: vault_info,
            token_program: Program::try_from(&token_program_info).unwrap(),
            system_program: Program::try_from(&system_info).unwrap(),
            rent: Sysvar::<Rent>::from_account_info(&rent_account_info)?,
        };

        // Include the CreatePollBumps with the bump for the poll account
        let bumps = CreatePollBumps {
            state: state_bump,
            poll: poll_bump,
            poll_anti_token: anti_token_bump,
            poll_pro_token: pro_token_bump,
        };
        /* Common Setup Ends Here */

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
        /* Common Setup Begins Here */
        let program_id = program_id();

        // Create token program account
        let mut token_program = TestAccountData::new_token_program();

        // Create mints
        let mut anti_mint = TestAccountData::new_mint_address(ANTI_MINT_ADDRESS);
        let mut pro_mint = TestAccountData::new_mint_address(PRO_MINT_ADDRESS);

        // Test double for Clock
        thread_local! {
            static MOCK_UNIX_TIMESTAMP: RefCell<i64> = RefCell::new(1736899200); // 2025-01-15T00:00:00Z
        }

        // Initialise state account
        let manager: Pubkey = Pubkey::new_unique();
        let mut state =
            TestAccountData::new_account_with_key_and_owner::<StateAccount>(manager, program_id);
        state
            .init_state_data(&StateAccount {
                poll_index: 0,
                authority: manager,
            })
            .unwrap();

        // Derive PDAs and bumps
        let (_state_pda, state_bump) = Pubkey::find_program_address(&[b"state"], &program_id);

        let (poll_pda, poll_bump) = Pubkey::find_program_address(
            &[b"poll", state.data[8..16].try_into().unwrap()],
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

        let mut poll =
            TestAccountData::new_account_with_key_and_owner::<PollAccount>(poll_pda, program_id);
        poll.init_poll_data(&PollAccount::default()).unwrap();

        // Initialise creator account
        let mut creator = TestAccountData {
            key: Pubkey::new_unique(),
            lamports: 50_000_000, // Insufficient payment
            data: vec![],
            owner: system_program::ID,
            executable: false,
            rent_epoch: 0,
        };

        // Create token accounts
        let mut poll_anti_token = TestAccountData::new_token_account(anti_token_pda);
        let mut poll_pro_token = TestAccountData::new_token_account(pro_token_pda);

        // Rent for accounts
        let mut rent_account = TestAccountData::init_rent_account();

        // Initialise token accounts
        poll_anti_token
            .init_token_account(ANTITOKEN_MULTISIG, anti_mint.key)
            .unwrap();
        poll_pro_token
            .init_token_account(ANTITOKEN_MULTISIG, pro_mint.key)
            .unwrap();

        // Initialise other accounts
        let mut system_program = TestAccountData::new_system_account();
        let mut vault = TestAccountData::new_vault_with_key();

        // Prepare account infos
        let state_info = state.to_account_info(false);
        let poll_info = poll.to_account_info(false);
        let authority_info = creator.to_account_info(true);
        let system_info = system_program.to_account_info(false);
        let anti_mint_info = anti_mint.to_account_info(false);
        let pro_mint_info = pro_mint.to_account_info(false);
        let poll_anti_token_info = poll_anti_token.to_account_info(false);
        let poll_pro_token_info = poll_pro_token.to_account_info(false);
        let token_program_info = token_program.to_account_info(false);
        let rent_account_info = rent_account.to_account_info(false);
        let vault_info = vault.to_account_info(false);

        // Set up CreatePoll context
        let mut accounts = CreatePoll {
            state: Account::try_from(&state_info).unwrap(),
            poll: Account::try_from(&poll_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            poll_anti_token: Account::try_from(&poll_anti_token_info).unwrap(),
            poll_pro_token: Account::try_from(&poll_pro_token_info).unwrap(),
            anti_mint: anti_mint_info.clone(),
            pro_mint: pro_mint_info.clone(),
            vault: vault_info,
            token_program: Program::try_from(&token_program_info).unwrap(),
            system_program: Program::try_from(&system_info).unwrap(),
            rent: Sysvar::<Rent>::from_account_info(&rent_account_info)?,
        };

        // Include the CreatePollBumps with the bump for the poll account
        let bumps = CreatePollBumps {
            state: state_bump,
            poll: poll_bump,
            poll_anti_token: anti_token_bump,
            poll_pro_token: pro_token_bump,
        };
        /* Common Setup Ends Here */

        // Test insufficient payment
        {
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
        /* Common Setup Begins Here */
        let program_id = program_id();

        // Create token program account
        let mut token_program = TestAccountData::new_token_program();

        // Create mints
        let mut anti_mint = TestAccountData::new_mint_address(ANTI_MINT_ADDRESS);
        let mut pro_mint = TestAccountData::new_mint_address(PRO_MINT_ADDRESS);

        // Test double for Clock
        thread_local! {
            static MOCK_UNIX_TIMESTAMP: RefCell<i64> = RefCell::new(1736899200); // 2025-01-15T00:00:00Z
        }

        // Initialise state account
        let manager: Pubkey = Pubkey::new_unique();
        let mut state =
            TestAccountData::new_account_with_key_and_owner::<StateAccount>(manager, program_id);
        state
            .init_state_data(&StateAccount {
                poll_index: 0,
                authority: manager,
            })
            .unwrap();

        // Derive PDAs and bumps
        let (_state_pda, state_bump) = Pubkey::find_program_address(&[b"state"], &program_id);

        let (poll_pda, poll_bump) = Pubkey::find_program_address(
            &[b"poll", state.data[8..16].try_into().unwrap()],
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

        let mut poll =
            TestAccountData::new_account_with_key_and_owner::<PollAccount>(poll_pda, program_id);
        poll.init_poll_data(&PollAccount::default()).unwrap();

        // Initialise creator account
        let mut creator = TestAccountData {
            key: Pubkey::new_unique(),
            lamports: 200_000_000,
            data: vec![],
            owner: system_program::ID,
            executable: false,
            rent_epoch: 0,
        };

        // Create token accounts
        let mut poll_anti_token = TestAccountData::new_token_account(anti_token_pda);
        let mut poll_pro_token = TestAccountData::new_token_account(pro_token_pda);

        // Rent for accounts
        let mut rent_account = TestAccountData::init_rent_account();

        // Initialise token accounts
        poll_anti_token
            .init_token_account(ANTITOKEN_MULTISIG, anti_mint.key)
            .unwrap();
        poll_pro_token
            .init_token_account(ANTITOKEN_MULTISIG, pro_mint.key)
            .unwrap();

        // Initialise other accounts
        let mut system_program = TestAccountData::new_system_account();
        let mut vault = TestAccountData::new_vault_with_key();

        // Prepare account infos
        let state_info = state.to_account_info(false);
        let poll_info = poll.to_account_info(false);
        let authority_info = creator.to_account_info(true);
        let system_info = system_program.to_account_info(false);
        let anti_mint_info = anti_mint.to_account_info(false);
        let pro_mint_info = pro_mint.to_account_info(false);
        let poll_anti_token_info = poll_anti_token.to_account_info(false);
        let poll_pro_token_info = poll_pro_token.to_account_info(false);
        let token_program_info = token_program.to_account_info(false);
        let rent_account_info = rent_account.to_account_info(false);
        let vault_info = vault.to_account_info(false);

        // Set up CreatePoll context
        let mut accounts = CreatePoll {
            state: Account::try_from(&state_info).unwrap(),
            poll: Account::try_from(&poll_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            poll_anti_token: Account::try_from(&poll_anti_token_info).unwrap(),
            poll_pro_token: Account::try_from(&poll_pro_token_info).unwrap(),
            anti_mint: anti_mint_info.clone(),
            pro_mint: pro_mint_info.clone(),
            vault: vault_info,
            token_program: Program::try_from(&token_program_info).unwrap(),
            system_program: Program::try_from(&system_info).unwrap(),
            rent: Sysvar::<Rent>::from_account_info(&rent_account_info)?,
        };
        /* Common Setup Ends Here */

        // Test title too long
        {
            // Include the CreatePollBumps with the bump for the poll account
            let bumps = CreatePollBumps {
                state: state_bump,
                poll: poll_bump,
                poll_anti_token: anti_token_bump,
                poll_pro_token: pro_token_bump,
            };
            let long_title = "a".repeat((MAX_TITLE_LENGTH + 1) as usize);
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
            // Include the CreatePollBumps with the bump for the poll account
            let bumps = CreatePollBumps {
                state: state_bump,
                poll: poll_bump,
                poll_anti_token: anti_token_bump,
                poll_pro_token: pro_token_bump,
            };
            let long_description = "a".repeat((MAX_DESCRIPTION_LENGTH + 1) as usize);
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
        /* Common Setup Begins Here */
        let program_id = program_id();

        // Create token program account
        let mut token_program = TestAccountData::new_token_program();

        // Create mints
        let mut anti_mint = TestAccountData::new_mint_address(ANTI_MINT_ADDRESS);
        let mut pro_mint = TestAccountData::new_mint_address(PRO_MINT_ADDRESS);

        // Test double for Clock
        thread_local! {
            static MOCK_UNIX_TIMESTAMP: RefCell<i64> = RefCell::new(1736899200); // 2025-01-15T00:00:00Z
        }

        // Initialise state account
        let manager: Pubkey = Pubkey::new_unique();
        let mut state =
            TestAccountData::new_account_with_key_and_owner::<StateAccount>(manager, program_id);
        state
            .init_state_data(&StateAccount {
                poll_index: 0,
                authority: manager,
            })
            .unwrap();

        // Derive PDAs and bumps
        let (_state_pda, state_bump) = Pubkey::find_program_address(&[b"state"], &program_id);

        let (poll_pda, poll_bump) = Pubkey::find_program_address(
            &[b"poll", state.data[8..16].try_into().unwrap()],
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

        let mut poll =
            TestAccountData::new_account_with_key_and_owner::<PollAccount>(poll_pda, program_id);
        poll.init_poll_data(&PollAccount::default()).unwrap();

        // Initialise creator account
        let mut creator = TestAccountData {
            key: Pubkey::new_unique(),
            lamports: 200_000_000,
            data: vec![],
            owner: system_program::ID,
            executable: false,
            rent_epoch: 0,
        };

        // Create token accounts
        let mut poll_anti_token = TestAccountData::new_token_account(anti_token_pda);
        let mut poll_pro_token = TestAccountData::new_token_account(pro_token_pda);

        // Rent for accounts
        let mut rent_account = TestAccountData::init_rent_account();

        // Initialise token accounts
        poll_anti_token
            .init_token_account(ANTITOKEN_MULTISIG, anti_mint.key)
            .unwrap();
        poll_pro_token
            .init_token_account(ANTITOKEN_MULTISIG, pro_mint.key)
            .unwrap();

        // Initialise other accounts
        let mut system_program = TestAccountData::new_system_account();
        let mut vault = TestAccountData::new_vault_with_key();

        // Prepare account infos
        let state_info = state.to_account_info(true);
        let poll_info = poll.to_account_info(true);
        let authority_info = creator.to_account_info(true);
        let system_info = system_program.to_account_info(false);
        let anti_mint_info = anti_mint.to_account_info(false);
        let pro_mint_info = pro_mint.to_account_info(false);
        let poll_anti_token_info = poll_anti_token.to_account_info(false);
        let poll_pro_token_info = poll_pro_token.to_account_info(false);
        let token_program_info = token_program.to_account_info(false);
        let rent_account_info = rent_account.to_account_info(false);
        let vault_info = vault.to_account_info(false);

        // Set up CreatePoll context
        let mut accounts = CreatePoll {
            state: Account::try_from(&state_info).unwrap(),
            poll: Account::try_from(&poll_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            poll_anti_token: Account::try_from(&poll_anti_token_info).unwrap(),
            poll_pro_token: Account::try_from(&poll_pro_token_info).unwrap(),
            anti_mint: anti_mint_info.clone(),
            pro_mint: pro_mint_info.clone(),
            vault: vault_info,
            token_program: Program::try_from(&token_program_info).unwrap(),
            system_program: Program::try_from(&system_info).unwrap(),
            rent: Sysvar::<Rent>::from_account_info(&rent_account_info)?,
        };
        /* Common Setup Ends Here */

        // Test invalid time range
        {
            // Include the CreatePollBumps with the bump for the poll account
            let bumps = CreatePollBumps {
                state: state_bump,
                poll: poll_bump,
                poll_anti_token: anti_token_bump,
                poll_pro_token: pro_token_bump,
            };
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
            // Include the CreatePollBumps with the bump for the poll account
            let bumps = CreatePollBumps {
                state: state_bump,
                poll: poll_bump,
                poll_anti_token: anti_token_bump,
                poll_pro_token: pro_token_bump,
            };
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

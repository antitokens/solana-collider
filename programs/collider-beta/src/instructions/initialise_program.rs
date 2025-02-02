//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's initialisation
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

use crate::Initialise;
use crate::PredictError;
use anchor_lang::prelude::*;

pub fn initialise(ctx: Context<Initialise>) -> Result<()> {
    let state = &mut ctx.accounts.state;

    // Prevent unnecessary state writes if already initialised
    require!(state.poll_index == 0, PredictError::AlreadyInitialised);

    // Directly set values without redundant references
    state.poll_index = 0;
    state.authority = ctx.accounts.authority.key();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::PROGRAM_ID;
    use crate::InitialiseBumps;
    use crate::PollAccount;
    use crate::PredictError;
    use crate::StateAccount;
    use anchor_lang::prelude::AccountInfo;
    use anchor_lang::solana_program::system_program;
    use anchor_lang::Discriminator;
    use solana_sdk::signature::{Keypair, Signer as _};
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
        fn new_owned_state<T: AccountSerialize + AccountDeserialize + Clone>(
            owner: Pubkey,
            key: Pubkey,
        ) -> Self {
            Self {
                key,
                lamports: 1_000_000,
                data: vec![0; 8 + PollAccount::LEN],
                owner,
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
    }

    #[test]
    fn test_successful_initialisation() {
        let program_id = program_id();
        let authority = Keypair::new();

        // Create test accounts
        let (state_pda, state_bump) = Pubkey::find_program_address(&[b"state"], &program_id);
        let mut state = TestAccountData::new_owned_state::<StateAccount>(program_id, state_pda);
        let mut authority = TestAccountData::new_authority_account(authority.pubkey());
        let mut system = TestAccountData::new_system_account();

        // Initialise state account
        let state_data = StateAccount {
            poll_index: 0,
            authority: authority.key,
        };
        state.init_state_data(&state_data).unwrap();

        // Get account infos
        let state_info = state.to_account_info(false);
        let authority_info = authority.to_account_info(true);
        let system_info = system.to_account_info(false);

        let mut accounts = Initialise {
            state: Account::try_from(&state_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            system_program: Program::try_from(&system_info).unwrap(),
        };

        let result = initialise(Context::new(
            &program_id,
            &mut accounts,
            &[],
            InitialiseBumps { state: state_bump },
        ));

        assert!(result.is_ok());
    }

    #[test]
    fn test_double_initialisation() {
        let program_id = program_id();
        let authority = Keypair::new();

        let (state_pda, state_bump) = Pubkey::find_program_address(&[b"state"], &program_id);
        let mut state = TestAccountData::new_owned_state::<StateAccount>(program_id, state_pda);
        let mut authority = TestAccountData::new_authority_account(authority.pubkey());
        let mut system = TestAccountData::new_system_account();

        let authority_key = authority.key;

        // Initialise state account
        let state_data = StateAccount {
            poll_index: 0,
            authority: authority_key,
        };
        state.init_state_data(&state_data).unwrap();

        // First initialisation
        {
            let state_info = state.to_account_info(false);
            let authority_info = authority.to_account_info(true);
            let system_info = system.to_account_info(false);

            let mut accounts = Initialise {
                state: Account::try_from(&state_info).unwrap(),
                authority: Signer::try_from(&authority_info).unwrap(),
                system_program: Program::try_from(&system_info).unwrap(),
            };

            let result1 = initialise(Context::new(
                &program_id,
                &mut accounts,
                &[],
                InitialiseBumps { state: state_bump },
            ));
            assert!(result1.is_ok());
        }

        // Update state
        let updated_state = StateAccount {
            poll_index: 1,
            authority: authority_key,
        };
        state.init_state_data(&updated_state).unwrap();

        // Second initialisation attempt
        {
            let state_info = state.to_account_info(false);
            let authority_info = authority.to_account_info(true);
            let system_info = system.to_account_info(false);

            let mut accounts = Initialise {
                state: Account::try_from(&state_info).unwrap(),
                authority: Signer::try_from(&authority_info).unwrap(),
                system_program: Program::try_from(&system_info).unwrap(),
            };

            let result2 = initialise(Context::new(
                &program_id,
                &mut accounts,
                &[],
                InitialiseBumps { state: state_bump },
            ));

            assert_eq!(
                result2.unwrap_err(),
                Error::from(PredictError::AlreadyInitialised)
            );
        }
    }

    #[test]
    fn test_initialisation_with_different_authority() {
        let program_id = program_id();
        let authority = Keypair::new();

        let (state_pda, state_bump) = Pubkey::find_program_address(&[b"state"], &program_id);
        let mut state = TestAccountData::new_owned_state::<StateAccount>(program_id, state_pda);
        let mut authority = TestAccountData::new_authority_account(authority.pubkey());
        let mut system = TestAccountData::new_system_account();

        // Switch to new authority
        let different_authority = Pubkey::new_unique();

        let state_data = StateAccount {
            poll_index: 0,
            authority: different_authority,
        };
        state.init_state_data(&state_data).unwrap();

        {
            let state_info = state.to_account_info(false);
            let authority_info = authority.to_account_info(true);
            let system_info = system.to_account_info(false);

            let mut accounts = Initialise {
                state: Account::try_from(&state_info).unwrap(),
                authority: Signer::try_from(&authority_info).unwrap(),
                system_program: Program::try_from(&system_info).unwrap(),
            };

            let result = initialise(Context::new(
                &program_id,
                &mut accounts,
                &[],
                InitialiseBumps { state: state_bump },
            ));

            assert!(result.is_ok());

            let state_account: StateAccount =
                StateAccount::try_deserialize(&mut state_info.try_borrow_data().unwrap().as_ref())
                    .unwrap();

            assert_eq!(state_account.authority, different_authority);
            assert_ne!(state_account.authority, program_id);
        }
    }

    #[test]
    fn test_initialisation_with_state_validation() {
        let program_id = program_id();
        let authority = Keypair::new();

        let (state_pda, state_bump) = Pubkey::find_program_address(&[b"state"], &program_id);
        let mut state = TestAccountData::new_owned_state::<StateAccount>(program_id, state_pda);
        let mut authority = TestAccountData::new_authority_account(authority.pubkey());
        let mut system = TestAccountData::new_system_account();

        let state_data = StateAccount {
            poll_index: 0,
            authority: authority.key,
        };

        // Initialise state account data before running the test
        state.init_state_data(&state_data).unwrap();

        // Get account infos
        let state_info = state.to_account_info(false);
        let authority_info = authority.to_account_info(true);
        let system_program_info = system.to_account_info(false);

        let mut accounts = Initialise {
            state: Account::try_from(&state_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            system_program: Program::try_from(&system_program_info).unwrap(),
        };

        let result = initialise(Context::new(
            &program_id,
            &mut accounts,
            &[],
            InitialiseBumps { state: state_bump },
        ));

        assert!(result.is_ok());

        // Verify state initialisation
        let state: StateAccount =
            StateAccount::try_deserialize(&mut state_info.try_borrow_data().unwrap().as_ref())
                .unwrap();

        assert_eq!(state.poll_index, 0);
        assert_eq!(state.authority, authority_info.key());
    }
}

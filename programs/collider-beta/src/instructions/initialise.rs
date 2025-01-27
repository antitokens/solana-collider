//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's initialisation
//! Version: 0.0.1
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

use crate::Initialise;
use anchor_lang::prelude::*;

pub fn initialise(ctx: Context<Initialise>) -> Result<()> {
    let state = &mut ctx.accounts.state;
    state.poll_index = 0;
    state.authority = ctx.accounts.authority.key();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InitialiseBumps;
    use crate::PredictError;
    use crate::StateAccount;
    use anchor_lang::prelude::AccountInfo;
    use anchor_lang::solana_program::system_program;

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
                data: vec![0; StateAccount::LEN],
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

    struct TestContext {
        program_id: Pubkey,
        state_data: TestAccountData,
        authority_data: TestAccountData,
        system_program_data: TestAccountData,
    }

    impl TestContext {
        fn new() -> Self {
            let program_id = Pubkey::new_unique();

            TestContext {
                program_id: program_id.clone(),
                state_data: TestAccountData::new_owned(program_id),
                authority_data: TestAccountData::new_owned(program_id),
                system_program_data: TestAccountData::new_owned(system_program::ID),
            }
        }
    }

    #[test]
    fn test_successful_initialization() {
        let mut ctx = TestContext::new();

        // Get account infos
        let state_info = ctx.state_data.to_account_info(false);
        let authority_info = ctx.authority_data.to_account_info(true);
        let system_program_info = ctx.system_program_data.to_account_info(false);

        let mut accounts = Initialise {
            state: Account::try_from(&state_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            system_program: Program::try_from(&system_program_info).unwrap(),
        };

        let result = initialise(Context::new(
            &ctx.program_id,
            &mut accounts,
            &[],
            InitialiseBumps {},
        ));

        assert!(result.is_ok());

        // Verify state initialization by deserializing the account data
        let state: StateAccount =
            StateAccount::try_deserialize(&mut state_info.try_borrow_data().unwrap().as_ref())
                .unwrap();

        assert_eq!(state.poll_index, 0);
        assert_eq!(state.authority, authority_info.key());
    }

    #[test]
    fn test_double_initialization() {
        let mut ctx = TestContext::new();

        // Get account infos
        let state_info = ctx.state_data.to_account_info(false);
        let authority_info = ctx.authority_data.to_account_info(true);
        let system_program_info = ctx.system_program_data.to_account_info(false);

        let mut accounts = Initialise {
            state: Account::try_from(&state_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            system_program: Program::try_from(&system_program_info).unwrap(),
        };

        // First initialization
        let result1 = initialise(Context::new(
            &ctx.program_id,
            &mut accounts,
            &[],
            InitialiseBumps {},
        ));
        assert!(result1.is_ok());

        // Try to initialize again
        let result2 = initialise(Context::new(
            &ctx.program_id,
            &mut accounts,
            &[],
            InitialiseBumps {},
        ));

        match result2 {
            Err(err) => assert_eq!(err, PredictError::AlreadyInitialised.into()),
            _ => panic!("Expected already initialised error"),
        }
    }

    #[test]
    fn test_initialization_with_different_authority() {
        let mut ctx = TestContext::new();
        ctx.authority_data.key = Pubkey::new_unique(); // Set different authority

        // Get account infos
        let state_info = ctx.state_data.to_account_info(false);
        let authority_info = ctx.authority_data.to_account_info(true);
        let system_program_info = ctx.system_program_data.to_account_info(false);

        let mut accounts = Initialise {
            state: Account::try_from(&state_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            system_program: Program::try_from(&system_program_info).unwrap(),
        };

        let result = initialise(Context::new(
            &ctx.program_id,
            &mut accounts,
            &[],
            InitialiseBumps {},
        ));

        assert!(result.is_ok());

        // Verify state initialization
        let state: StateAccount =
            StateAccount::try_deserialize(&mut state_info.try_borrow_data().unwrap().as_ref())
                .unwrap();

        assert_eq!(state.authority, authority_info.key());
        assert_ne!(state.authority, ctx.program_id);
    }

    #[test]
    fn test_initialization_state_validation() {
        let mut ctx = TestContext::new();

        // Get account infos
        let state_info = ctx.state_data.to_account_info(false);
        let authority_info = ctx.authority_data.to_account_info(true);
        let system_program_info = ctx.system_program_data.to_account_info(false);

        let mut accounts = Initialise {
            state: Account::try_from(&state_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            system_program: Program::try_from(&system_program_info).unwrap(),
        };

        let result = initialise(Context::new(
            &ctx.program_id,
            &mut accounts,
            &[],
            InitialiseBumps {},
        ));

        assert!(result.is_ok());

        // Verify state initialization
        let state: StateAccount =
            StateAccount::try_deserialize(&mut state_info.try_borrow_data().unwrap().as_ref())
                .unwrap();

        assert_eq!(state.poll_index, 0);
        assert_eq!(state.authority, authority_info.key());
    }
}

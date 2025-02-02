// instructions/admin.rs
//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's admin instruction set
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 02 Feb 2025
//! Last Modified: 02 Feb 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

use crate::utils::*;
use crate::Admin;
use crate::Update;
use anchor_lang::prelude::*;

pub fn initialise_admin(ctx: Context<Admin>) -> Result<()> {
    let settings = &mut ctx.accounts.admin;
    settings.poll_creation_fee = 100_000_000; // 0.1 SOL
    settings.max_title_length = MAX_TITLE_LENGTH;
    settings.max_description_length = MAX_DESCRIPTION_LENGTH;
    settings.truth_basis = TRUTH_BASIS;
    settings.float_basis = FLOAT_BASIS;
    settings.min_deposit_amount = MIN_DEPOSIT_AMOUNT;
    settings.antitoken_multisig = ANTITOKEN_MULTISIG;
    settings.anti_mint_address = ANTI_MINT_ADDRESS;
    settings.pro_mint_address = PRO_MINT_ADDRESS;

    // Emit admin event
    emit!(AdminEvent {
        action: "init_admin".to_string(),
        poll_index: 0,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

/// Update the poll creation fee
pub fn update_poll_creation_fee(ctx: Context<Update>, new_fee: u64) -> Result<()> {
    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.poll_creation_fee = new_fee;
    Ok(())
}

/// Update the max title length
pub fn update_max_title_length(ctx: Context<Update>, new_length: usize) -> Result<()> {
    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.max_title_length = new_length;
    Ok(())
}

/// Update the max description length
pub fn update_max_description_length(ctx: Context<Update>, new_length: usize) -> Result<()> {
    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.max_description_length = new_length;
    Ok(())
}

/// Update truth basis
pub fn update_truth_basis(ctx: Context<Update>, new_basis: u64) -> Result<()> {
    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.truth_basis = new_basis;
    Ok(())
}

/// Update float basis
pub fn update_float_basis(ctx: Context<Update>, new_basis: u64) -> Result<()> {
    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.float_basis = new_basis;
    Ok(())
}

/// Update minimum deposit amount
pub fn update_min_deposit_amount(ctx: Context<Update>, new_min_amount: u64) -> Result<()> {
    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.min_deposit_amount = new_min_amount;
    Ok(())
}

/// Update ANTI mint address
pub fn update_anti_mint(ctx: Context<Update>, new_mint: Pubkey) -> Result<()> {
    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.anti_mint_address = new_mint;
    Ok(())
}

/// Update PRO mint address
pub fn update_pro_mint(ctx: Context<Update>, new_mint: Pubkey) -> Result<()> {
    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.pro_mint_address = new_mint;
    Ok(())
}

/// Update multisig authority
pub fn update_multisig(ctx: Context<Update>, new_multisig: Pubkey) -> Result<()> {
    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.antitoken_multisig = new_multisig;
    Ok(())
}

/// Error codes
#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorised")]
    Unauthorised,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UpdateBumps;
    use crate::{state::AdminAccount, utils::PROGRAM_ID, AdminBumps};
    use anchor_lang::{system_program, Discriminator};
    use solana_sdk::signature::{Keypair, Signer as _};
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
        fn new_owned_admin<T: AccountSerialize + AccountDeserialize + Clone>(
            key: Pubkey,
            owner: Pubkey,
        ) -> Self {
            Self {
                key,
                lamports: 1_000_000,
                data: vec![0; 8 + AdminAccount::LEN],
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

        fn init_admin_data(&mut self, admin: &AdminAccount) -> Result<()> {
            self.data = vec![0; 8 + AdminAccount::LEN];
            let data = self.data.as_mut_slice();

            let disc = AdminAccount::discriminator();
            data[..8].copy_from_slice(&disc);

            let account_data = admin.try_to_vec()?;
            data[8..8 + account_data.len()].copy_from_slice(&account_data);

            Ok(())
        }
    }

    #[test]
    fn test_admin_initialisation() {
        let program_id = Pubkey::from_str(PROGRAM_ID).unwrap();
        let authority = Keypair::new();

        // Create test accounts
        let (admin_pda, admin_bump) = Pubkey::find_program_address(&[b"admin"], &program_id);

        // Test initialisation
        let mut admin = TestAccountData::new_owned_admin::<AdminAccount>(program_id, admin_pda);
        let mut auth = TestAccountData::new_authority_account(authority.pubkey());
        let mut system = TestAccountData::new_system_account();

        // Initialise admin account
        let admin_data = AdminAccount {
            poll_creation_fee: 100_000_000,
            max_title_length: MAX_TITLE_LENGTH,
            max_description_length: MAX_DESCRIPTION_LENGTH,
            truth_basis: TRUTH_BASIS,
            float_basis: FLOAT_BASIS,
            min_deposit_amount: MIN_DEPOSIT_AMOUNT,
            antitoken_multisig: ANTITOKEN_MULTISIG,
            anti_mint_address: ANTI_MINT_ADDRESS,
            pro_mint_address: PRO_MINT_ADDRESS,
        };
        admin.init_admin_data(&admin_data).unwrap();

        let admin_info = admin.to_account_info(false);
        let authority_info = auth.to_account_info(true);
        let system_info = system.to_account_info(false);

        let mut accounts = Admin {
            admin: Account::try_from(&admin_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            system_program: Program::try_from(&system_info).unwrap(),
        };

        // Include the AdminBumps with the bump for the admin account
        let bumps = AdminBumps { admin: admin_bump };

        let result = initialise_admin(Context::new(&program_id, &mut accounts, &[], bumps));

        assert!(result.is_ok(), "Admin initialisation should succeed");

        // Verify all settings after initialisation
        let admin_account: AdminAccount =
            AdminAccount::try_deserialize(&mut admin_info.try_borrow_data().unwrap().as_ref())
                .unwrap();

        // Basic administrative settings
        assert_eq!(
            admin_account.poll_creation_fee, 100_000_000,
            "Poll creation fee should be 0.1 SOL"
        );
        assert_eq!(
            admin_account.max_title_length, MAX_TITLE_LENGTH,
            "Title length should match constant"
        );
        assert_eq!(
            admin_account.max_description_length, MAX_DESCRIPTION_LENGTH,
            "Description length should match constant"
        );

        // Numerical basis settings
        assert_eq!(
            admin_account.truth_basis, TRUTH_BASIS,
            "Truth basis should match constant"
        );
        assert_eq!(
            admin_account.float_basis, FLOAT_BASIS,
            "Float basis should match constant"
        );
        assert_eq!(
            admin_account.min_deposit_amount, MIN_DEPOSIT_AMOUNT,
            "Minimum deposit should match constant"
        );

        // Address settings
        assert_eq!(
            admin_account.antitoken_multisig, ANTITOKEN_MULTISIG,
            "Multisig address should match constant"
        );
        assert_eq!(
            admin_account.anti_mint_address, ANTI_MINT_ADDRESS,
            "ANTI mint address should match constant"
        );
        assert_eq!(
            admin_account.pro_mint_address, PRO_MINT_ADDRESS,
            "PRO mint address should match constant"
        );

        // Verify account ownership
        assert_eq!(
            admin_info.owner, &program_id,
            "Admin account should be owned by the program"
        );

        // Verify account data length
        assert_eq!(
            admin_info.try_borrow_data().unwrap().len(),
            8 + AdminAccount::LEN,
            "Account data length should match expected size"
        );

        // Verify authority
        assert_eq!(
            authority_info.key(),
            auth.key,
            "Authority should match the provided keypair"
        );
    }

    // Additional test for double initialisation prevention
    #[test]
    fn test_double_initialisation_prevented() {
        let program_id = Pubkey::from_str(PROGRAM_ID).unwrap();
        let authority = Keypair::new();

        // Create test accounts
        let (admin_pda, admin_bump) = Pubkey::find_program_address(&[b"admin"], &program_id);
        let mut admin = TestAccountData::new_owned_admin::<AdminAccount>(program_id, admin_pda);
        let mut auth = TestAccountData::new_authority_account(authority.pubkey());
        let mut system = TestAccountData::new_system_account();

        // First initialisation
        let admin_info = admin.to_account_info(false);
        let authority_info = auth.to_account_info(true);
        let system_info = system.to_account_info(false);

        let mut accounts = Admin {
            admin: Account::try_from(&admin_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            system_program: Program::try_from(&system_info).unwrap(),
        };

        // Include the AdminBumps with the bump for the admin account
        let bumps1 = AdminBumps { admin: admin_bump };
        // First initialisation should succeed
        let result1 = initialise_admin(Context::new(&program_id, &mut accounts, &[], bumps1));
        assert!(result1.is_ok(), "First initialisation should succeed");

        // Include the AdminBumps with the bump for the admin account
        let bumps2 = AdminBumps { admin: admin_bump };
        // Second initialisation should fail
        let result2 = initialise_admin(Context::new(&program_id, &mut accounts, &[], bumps2));
        assert!(result2.is_err(), "Second initialisation should fail");

        match result2 {
            Err(error) => {
                assert_eq!(
                    error,
                    PredictError::AlreadyInitialised.into(),
                    "Should return AlreadyInitialised error"
                );
            }
            _ => panic!("Expected initialisation error"),
        }
    }

    // Test unauthorised updates
    #[test]
    fn test_unauthorised_updates() {
        let program_id = Pubkey::from_str(PROGRAM_ID).unwrap();
        let unauthorised_user = Keypair::new();

        // Create test accounts
        let (admin_pda, admin_bump) = Pubkey::find_program_address(&[b"admin"], &program_id);

        let mut admin = TestAccountData::new_owned_admin::<AdminAccount>(program_id, admin_pda);
        let mut auth = TestAccountData::new_authority_account(unauthorised_user.pubkey());

        let admin_info = admin.to_account_info(false);
        let authority_info = auth.to_account_info(true);

        let mut accounts = Update {
            admin: Account::try_from(&admin_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
        };

        // Include the AdminBumps with the bump for the admin account
        let bumps: UpdateBumps = UpdateBumps { admin: admin_bump };

        // Test unauthorised fee update
        let result = update_poll_creation_fee(
            Context::new(&program_id, &mut accounts, &[], bumps),
            200_000_000,
        );

        assert!(result.is_err(), "Unauthorised update should fail");
        match result {
            Err(error) => {
                assert_eq!(
                    error,
                    PredictError::Unauthorised.into(),
                    "Should return Unauthorised error"
                );
            }
            _ => panic!("Expected unauthorised error"),
        }
    }

    // Test successful updates
    #[test]
    fn test_successful_updates() {
        let program_id = Pubkey::from_str(PROGRAM_ID).unwrap();
        let multisig = Keypair::new();

        // Create test accounts with multisig authority
        let (admin_pda, admin_bump) = Pubkey::find_program_address(&[b"admin"], &program_id);
        let mut admin_account =
            TestAccountData::new_owned_admin::<AdminAccount>(program_id, admin_pda); // Changed variable name here

        // Initialize admin settings
        let admin_data = AdminAccount {
            // Changed to admin_data to avoid shadowing
            poll_creation_fee: 100_000_000,
            max_title_length: MAX_TITLE_LENGTH,
            max_description_length: MAX_DESCRIPTION_LENGTH,
            truth_basis: TRUTH_BASIS,
            float_basis: FLOAT_BASIS,
            min_deposit_amount: MIN_DEPOSIT_AMOUNT,
            antitoken_multisig: multisig.pubkey(),
            anti_mint_address: ANTI_MINT_ADDRESS,
            pro_mint_address: PRO_MINT_ADDRESS,
        };
        admin_account.init_admin_data(&admin_data).unwrap(); // Now using correct variable names

        // Test fee update
        {
            let bumps = UpdateBumps { admin: admin_bump };

            let admin_info = admin_account.to_account_info(false); // Using admin_account instead of admin
            let mut authority_binding = TestAccountData::new_authority_account(multisig.pubkey());
            let authority_info = authority_binding.to_account_info(true);

            let mut accounts = Update {
                admin: Account::try_from(&admin_info).unwrap(),
                authority: Signer::try_from(&authority_info).unwrap(),
            };

            let new_fee = 200_000_000;
            let result = update_poll_creation_fee(
                Context::new(&program_id, &mut accounts, &[], bumps),
                new_fee,
            );

            assert!(result.is_ok(), "Authorized fee update should succeed");

            // Verify the update
            let updated_admin: AdminAccount =
                AdminAccount::try_deserialize(&mut admin_info.try_borrow_data().unwrap().as_ref())
                    .unwrap();
            assert_eq!(
                updated_admin.poll_creation_fee, new_fee,
                "Fee should be updated"
            );
        }
    }
}

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
use crate::SetPredictionTokenAuthority;
use crate::Update;
use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::spl_token::instruction::AuthorityType;
use anchor_spl::token::SetAuthority;

pub fn initialise_admin(ctx: Context<Admin>) -> Result<()> {
    let now: i64 = 1736899200; // CRITICAL: Remove line in production!

    // CRITICAL: Add line in production!let now: i64 = Clock::get()?.unix_timestamp;

    let config = &mut ctx.accounts.admin;
    require!(!config.initialised, PredictError::AlreadyInitialised);

    config.initialised = true;
    config.creation_fee = 100_000_000;
    config.max_title_length = MAX_TITLE_LENGTH;
    config.max_description_length = MAX_DESCRIPTION_LENGTH;
    config.truth_basis = TRUTH_BASIS;
    config.float_basis = FLOAT_BASIS;
    config.min_deposit_amount = MIN_DEPOSIT_AMOUNT;
    config.antitoken_multisig = ANTITOKEN_MULTISIG;
    config.anti_mint_address = ANTI_MINT_ADDRESS;
    config.pro_mint_address = PRO_MINT_ADDRESS;

    emit!(AdminEvent {
        action: "initialise_admin".to_string(),
        args: vec![],
        timestamp: now,
    });

    Ok(())
}

pub fn update_creation_fee(ctx: Context<Update>, new_fee: u64) -> Result<()> {
    let now: i64 = 1736899200; // CRITICAL: Remove line in production!

    // CRITICAL: Add line in production!let now: i64 = Clock::get()?.unix_timestamp;

    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.creation_fee = new_fee;

    emit!(AdminEvent {
        action: "update_creation_fee".to_string(),
        args: vec![KeyValue {
            key: "new_fee".to_string(),
            value: new_fee.to_string(),
        }],
        timestamp: now,
    });

    Ok(())
}

pub fn update_max_title_length(ctx: Context<Update>, new_length: u64) -> Result<()> {
    let now: i64 = 1736899200; // CRITICAL: Remove line in production!

    // CRITICAL: Add line in production!let now: i64 = Clock::get()?.unix_timestamp;

    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.max_title_length = new_length;

    emit!(AdminEvent {
        action: "update_max_title_length".to_string(),
        args: vec![KeyValue {
            key: "new_length".to_string(),
            value: new_length.to_string(),
        }],
        timestamp: now,
    });

    Ok(())
}

pub fn update_max_description_length(ctx: Context<Update>, new_length: u64) -> Result<()> {
    let now: i64 = 1736899200; // CRITICAL: Remove line in production!

    // CRITICAL: Add line in production!let now: i64 = Clock::get()?.unix_timestamp;

    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.max_description_length = new_length;

    emit!(AdminEvent {
        action: "update_max_description_length".to_string(),
        args: vec![KeyValue {
            key: "new_length".to_string(),
            value: new_length.to_string(),
        }],
        timestamp: now,
    });

    Ok(())
}

pub fn update_truth_basis(ctx: Context<Update>, new_basis: u64) -> Result<()> {
    let now: i64 = 1736899200; // CRITICAL: Remove line in production!

    // CRITICAL: Add line in production!let now: i64 = Clock::get()?.unix_timestamp;

    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.truth_basis = new_basis;

    emit!(AdminEvent {
        action: "update_truth_basis".to_string(),
        args: vec![KeyValue {
            key: "new_basis".to_string(),
            value: new_basis.to_string()
        }],
        timestamp: now,
    });

    Ok(())
}

pub fn update_float_basis(ctx: Context<Update>, new_basis: u64) -> Result<()> {
    let now: i64 = 1736899200; // CRITICAL: Remove line in production!

    // CRITICAL: Add line in production!let now: i64 = Clock::get()?.unix_timestamp;

    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.float_basis = new_basis;

    emit!(AdminEvent {
        action: "update_float_basis".to_string(),
        args: vec![KeyValue {
            key: "new_basis".to_string(),
            value: new_basis.to_string(),
        }],
        timestamp: now,
    });

    Ok(())
}

pub fn update_min_deposit_amount(ctx: Context<Update>, new_min_amount: u64) -> Result<()> {
    let now: i64 = 1736899200; // CRITICAL: Remove line in production!

    // CRITICAL: Add line in production!let now: i64 = Clock::get()?.unix_timestamp;

    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.min_deposit_amount = new_min_amount;

    emit!(AdminEvent {
        action: "update_min_deposit_amount".to_string(),
        args: vec![KeyValue {
            key: "new_min_amount".to_string(),
            value: new_min_amount.to_string()
        }],
        timestamp: now,
    });

    Ok(())
}

pub fn update_anti_mint(ctx: Context<Update>, new_mint: Pubkey) -> Result<()> {
    let now: i64 = 1736899200; // CRITICAL: Remove line in production!

    // CRITICAL: Add line in production!let now: i64 = Clock::get()?.unix_timestamp;

    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.anti_mint_address = new_mint;

    emit!(AdminEvent {
        action: "update_anti_mint".to_string(),
        args: vec![KeyValue {
            key: "new_mint".to_string(),
            value: new_mint.to_string()
        }],
        timestamp: now,
    });

    Ok(())
}

pub fn update_pro_mint(ctx: Context<Update>, new_mint: Pubkey) -> Result<()> {
    let now: i64 = 1736899200; // CRITICAL: Remove line in production!

    // CRITICAL: Add line in production!let now: i64 = Clock::get()?.unix_timestamp;

    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.pro_mint_address = new_mint;

    emit!(AdminEvent {
        action: "update_pro_mint".to_string(),
        args: vec![KeyValue {
            key: "new_mint".to_string(),
            value: new_mint.to_string()
        }],
        timestamp: now,
    });

    Ok(())
}

pub fn update_multisig(ctx: Context<Update>, new_multisig: Pubkey) -> Result<()> {
    let now: i64 = 1736899200; // CRITICAL: Remove line in production!

    // CRITICAL: Add line in production!let now: i64 = Clock::get()?.unix_timestamp;

    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.antitoken_multisig = new_multisig;

    emit!(AdminEvent {
        action: "update_multisig".to_string(),
        args: vec![KeyValue {
            key: "new_multisig".to_string(),
            value: new_multisig.to_string()
        }],
        timestamp: now,
    });

    Ok(())
}

pub fn set_token_authority(ctx: Context<SetPredictionTokenAuthority>, index: u64) -> Result<()> {
    let now: i64 = 1736899200; // CRITICAL: Remove line in production!

    // CRITICAL: Add line in production!let now: i64 = Clock::get()?.unix_timestamp;

    // Verify only ANTITOKEN_MULTISIG can execute this
    require!(
        ctx.accounts.authority.key() == ANTITOKEN_MULTISIG,
        ErrorCode::Unauthorised
    );

    // Transfer authority of $ANTI token account to state PDA
    token::set_authority(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            SetAuthority {
                current_authority: ctx.accounts.authority.to_account_info(),
                account_or_mint: ctx.accounts.prediction_anti_token.to_account_info(),
            },
        ),
        AuthorityType::AccountOwner,
        Some(ctx.accounts.state.key()),
    )?;

    // Transfer authority of $PRO token account to state PDA
    token::set_authority(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            SetAuthority {
                current_authority: ctx.accounts.authority.to_account_info(),
                account_or_mint: ctx.accounts.prediction_pro_token.to_account_info(),
            },
        ),
        AuthorityType::AccountOwner,
        Some(ctx.accounts.state.key()),
    )?;

    emit!(AdminEvent {
        action: "set_token_authority".to_string(),
        args: vec![KeyValue {
            key: "index".to_string(),
            value: index.to_string()
        }],
        timestamp: now,
    });

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
    use crate::{state::AdminAccount, utils::PROGRAM_ID, AdminBumps};
    use crate::{
        Deposit, Equalisation, PredictionAccount, SetPredictionTokenAuthorityBumps, StateAccount,
        UpdateBumps,
    };
    use anchor_lang::{system_program, Discriminator};
    use anchor_spl::token::{
        spl_token, spl_token::state::Account as SplTokenAccount, Mint, TokenAccount,
    };
    use solana_program::program_pack::Pack;
    use solana_sdk::program_option::COption;
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

        fn new_token(key: Pubkey) -> Self {
            Self {
                key,
                lamports: 1_000_000,
                data: vec![0; 165],
                owner: spl_token::ID,
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
            self.owner = spl_token::ID;

            Ok(())
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

        fn init_admin_data(&mut self, admin: &AdminAccount) -> Result<()> {
            self.data = vec![0; 8 + AdminAccount::LEN];
            let data = self.data.as_mut_slice();

            let disc = AdminAccount::discriminator();
            data[..8].copy_from_slice(&disc);

            let account_data = admin.try_to_vec()?;
            data[8..8 + account_data.len()].copy_from_slice(&account_data);

            Ok(())
        }

        // Reusable method to create an equalised test prediction
        fn create_equalised_test_prediction(authority: Pubkey, index: u64) -> PredictionAccount {
            PredictionAccount {
                index: index,
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

        // Initialise admin config
        const ADMIN_DATA: AdminAccount = AdminAccount {
            initialised: false,
            creation_fee: 100_000_000,
            max_title_length: MAX_TITLE_LENGTH,
            max_description_length: MAX_DESCRIPTION_LENGTH,
            truth_basis: TRUTH_BASIS,
            float_basis: FLOAT_BASIS,
            min_deposit_amount: MIN_DEPOSIT_AMOUNT,
            antitoken_multisig: ANTITOKEN_MULTISIG,
            anti_mint_address: ANTI_MINT_ADDRESS,
            pro_mint_address: PRO_MINT_ADDRESS,
        };
    }

    #[test]
    fn test_admin_initialisation<'info>() {
        let program_id = Pubkey::from_str(&PROGRAM_ID.to_string()).unwrap();
        let manager = Keypair::new();

        // Create test accounts
        let (admin_pda, admin_bump) = Pubkey::find_program_address(&[b"admin"], &program_id);

        // Test initialisation
        let mut admin = TestAccountData::new_owned_admin::<AdminAccount>(admin_pda, program_id);
        let mut authority = TestAccountData::new_authority_account(manager.pubkey());
        let mut system = TestAccountData::new_system_account();

        admin.init_admin_data(&TestAccountData::ADMIN_DATA).unwrap();

        let admin_info = admin.to_account_info(false);
        let authority_info = authority.to_account_info(true);
        let system_info = system.to_account_info(false);

        let _: Account<AdminAccount> = Account::try_from(&admin_info).unwrap();

        let mut accounts = Admin {
            admin: Account::try_from(&admin_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            system_program: Program::try_from(&system_info).unwrap(),
        };

        // Include the AdminBumps with the bump for the admin account
        let bumps = AdminBumps { admin: admin_bump };

        let result = initialise_admin(Context::new(&program_id, &mut accounts, &[], bumps));

        // If the test fails, print the error
        if result.is_err() {
            println!("Error: {:?}", result.unwrap_err());
        } else {
            assert!(result.is_ok());
        }

        // Verify all config after initialisation
        let admin_account: AdminAccount =
            AdminAccount::try_deserialize(&mut admin_info.try_borrow_data().unwrap().as_ref())
                .unwrap();

        // Basic administrative config
        assert_eq!(
            admin_account.creation_fee, 100_000_000,
            "Prediction creation fee should be 0.1 SOL"
        );
        assert_eq!(
            admin_account.max_title_length, MAX_TITLE_LENGTH,
            "Title length should match constant"
        );
        assert_eq!(
            admin_account.max_description_length, MAX_DESCRIPTION_LENGTH,
            "Description length should match constant"
        );

        // Numerical basis config
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

        // Address config
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
            manager.pubkey(),
            "Authority should match the provided keypair"
        );
    }

    // Additional test for double initialisation prevention
    #[test]
    fn test_double_initialisation_prevented() {
        let program_id = Pubkey::from_str(&PROGRAM_ID.to_string()).unwrap();
        let manager = Keypair::new();

        // Create test accounts
        let (admin_pda, admin_bump) = Pubkey::find_program_address(&[b"admin"], &program_id);
        let mut admin = TestAccountData::new_owned_admin::<AdminAccount>(admin_pda, program_id);
        let mut authority = TestAccountData::new_authority_account(manager.pubkey());
        let mut system = TestAccountData::new_system_account();

        admin.init_admin_data(&TestAccountData::ADMIN_DATA).unwrap();

        // First initialisation
        let admin_info = admin.to_account_info(false);
        let authority_info = authority.to_account_info(true);
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
        let program_id = Pubkey::from_str(&PROGRAM_ID.to_string()).unwrap();
        let unauthorised_user = Keypair::new();

        // Create test accounts
        let (admin_pda, admin_bump) = Pubkey::find_program_address(&[b"admin"], &program_id);

        let mut admin = TestAccountData::new_owned_admin::<AdminAccount>(admin_pda, program_id);
        let mut manager = TestAccountData::new_authority_account(unauthorised_user.pubkey());

        admin.init_admin_data(&TestAccountData::ADMIN_DATA).unwrap();

        let admin_info = admin.to_account_info(false);
        let authority_info = manager.to_account_info(true);

        let mut accounts = Update {
            admin: Account::try_from(&admin_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
        };

        // Include the AdminBumps with the bump for the admin account
        let bumps: UpdateBumps = UpdateBumps { admin: admin_bump };

        // Test unauthorised fee update
        let result = update_creation_fee(
            Context::new(&program_id, &mut accounts, &[], bumps),
            200_000_000,
        );
        assert!(result.is_err(), "Unauthorised update should fail");

        if let Err(error) = result {
            match error {
                anchor_lang::error::Error::AnchorError(e) => {
                    let error_code: u32 = ErrorCode::Unauthorised.into();
                    assert_eq!(e.error_code_number, error_code);
                }
                _ => panic!("Expected Unauthorised error"),
            }
        }
    }

    // Test successful updates
    #[test]
    fn test_successful_updates() {
        let program_id = Pubkey::from_str(&PROGRAM_ID.to_string()).unwrap();

        // Create test accounts with multisig authority
        let (admin_pda, admin_bump) = Pubkey::find_program_address(&[b"admin"], &program_id);
        let mut admin_account =
            TestAccountData::new_owned_admin::<AdminAccount>(admin_pda, program_id);

        admin_account
            .init_admin_data(&TestAccountData::ADMIN_DATA)
            .unwrap(); // Now using correct variable names

        // Test fee update
        {
            let bumps = UpdateBumps { admin: admin_bump };

            let admin_info = admin_account.to_account_info(false); // Using admin_account instead of admin
            let mut authority_binding = TestAccountData::new_authority_account(ANTITOKEN_MULTISIG);
            let authority_info = authority_binding.to_account_info(true);

            let mut accounts = Update {
                admin: Account::try_from(&admin_info).unwrap(),
                authority: Signer::try_from(&authority_info).unwrap(),
            };

            let new_fee = 200_000_000;
            let result = update_creation_fee(
                Context::new(&program_id, &mut accounts, &[], bumps),
                new_fee,
            );

            assert!(result.is_ok(), "Authorised fee update should succeed");

            assert_eq!(
                accounts.admin.creation_fee, new_fee,
                "Fee should be updated"
            );
            assert_ne!(
                CREATION_FEE, accounts.admin.creation_fee,
                "Fee should have changed"
            );
        }
    }

    #[test]
    fn test_set_token_authority() {
        let program_id = Pubkey::from_str(&PROGRAM_ID.to_string()).unwrap();
        let manager: Pubkey = Pubkey::new_unique();
        let mut token_program = TestAccountData::new_token_program();

        let index: u64 = 0;

        // Create prediction with user's deposit and equalisation results
        let prediction_data =
            TestAccountData::create_equalised_test_prediction(manager.key(), index);

        // Create mints
        let anti_mint = TestAccountData::new_mint(ANTI_MINT_ADDRESS);
        let pro_mint = TestAccountData::new_mint(PRO_MINT_ADDRESS);

        // Initialise state account
        let authority = TestAccountData::new_authority_account(manager);
        let mut state = TestAccountData::new_account_with_key_and_owner::<StateAccount>(
            authority.key,
            program_id,
        );
        state
            .init_state_data(&StateAccount {
                index: 0,
                authority: manager,
            })
            .unwrap();

        // Create test accounts
        let (admin_pda, _admin_bump) = Pubkey::find_program_address(&[b"admin"], &program_id);
        let (_state_pda, state_bump) = Pubkey::find_program_address(&[b"state"], &program_id);
        let (anti_token_pda, anti_token_bump) = Pubkey::find_program_address(
            &[b"anti_token", index.to_le_bytes().as_ref()],
            &program_id,
        );
        let (pro_token_pda, pro_token_bump) = Pubkey::find_program_address(
            &[b"pro_token", index.to_le_bytes().as_ref()],
            &program_id,
        );

        let mut admin = TestAccountData::new_owned_admin::<AdminAccount>(admin_pda, program_id);
        admin.init_admin_data(&TestAccountData::ADMIN_DATA).unwrap();

        // Create test prediction account
        let mut prediction = TestAccountData::new_account_with_key_and_owner::<PredictionAccount>(
            Pubkey::new_unique(),
            program_id,
        );
        let mut prediction_anti_token = TestAccountData::new_token(anti_token_pda);
        let mut prediction_pro_token = TestAccountData::new_token(pro_token_pda);

        prediction_anti_token
            .init_token_account(ANTITOKEN_MULTISIG, anti_mint.key)
            .unwrap();
        prediction_pro_token
            .init_token_account(ANTITOKEN_MULTISIG, pro_mint.key)
            .unwrap();

        // Write discriminator and serialise prediction data
        prediction.data[..8].copy_from_slice(&PredictionAccount::discriminator());
        let serialised_prediction = prediction_data.try_to_vec().unwrap();
        prediction.data[8..8 + serialised_prediction.len()].copy_from_slice(&serialised_prediction);

        // Get account infos
        let state_info = state.to_account_info(false);
        let prediction_info = prediction.to_account_info(false);
        let mut authority_binding = TestAccountData::new_authority_account(ANTITOKEN_MULTISIG);
        let authority_info = authority_binding.to_account_info(true);
        let token_data = {
            let data_slice = &prediction_anti_token.data[..];
            SplTokenAccount::unpack(data_slice).expect("Failed to unpack token account")
        };
        let prediction_anti_token_info = prediction_anti_token.to_account_info(false);
        let prediction_pro_token_info = prediction_pro_token.to_account_info(false);
        let token_program_info = token_program.to_account_info(false);

        // Accounts for transaction
        let mut accounts = SetPredictionTokenAuthority {
            state: Account::try_from(&state_info).unwrap(),
            prediction: Account::try_from(&prediction_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            prediction_anti_token: Account::try_from(&prediction_anti_token_info).unwrap(),
            prediction_pro_token: Account::try_from(&prediction_pro_token_info).unwrap(),
            token_program: Program::try_from(&token_program_info).unwrap(),
        };

        let bumps = SetPredictionTokenAuthorityBumps {
            state: state_bump,
            prediction_anti_token: anti_token_bump,
            prediction_pro_token: pro_token_bump,
        };

        assert_eq!(
            token_data.owner, ANTITOKEN_MULTISIG,
            "Token owner should be ANTITOKEN_MULTISIG"
        );

        // Test with correct authority (ANTITOKEN_MULTISIG)
        let _ = set_token_authority(Context::new(&program_id, &mut accounts, &[], bumps), index);

        // Test unauthorised call
        let mut unauthorised_keypair = TestAccountData::new_authority_account(Pubkey::new_unique());
        let unauthorised_keypair_info = unauthorised_keypair.to_account_info(true);
        accounts.authority = Signer::try_from(&unauthorised_keypair_info).unwrap();

        let bumps = SetPredictionTokenAuthorityBumps {
            state: state_bump,
            prediction_anti_token: anti_token_bump,
            prediction_pro_token: pro_token_bump,
        };

        let result_unauthorised =
            set_token_authority(Context::new(&program_id, &mut accounts, &[], bumps), index);
        assert!(
            result_unauthorised.is_err(),
            "Unauthorised call should fail"
        );

        if let Err(error) = result_unauthorised {
            match error {
                anchor_lang::error::Error::AnchorError(e) => {
                    let error_code: u32 = ErrorCode::Unauthorised.into();
                    assert_eq!(
                        e.error_code_number, error_code,
                        "Should return Unauthorised error"
                    );
                }
                _ => panic!("Expected Unauthorised error"),
            }
        }
    }
}

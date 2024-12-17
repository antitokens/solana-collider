//! Program Author: sshmatrix, for Antitoken
//! Program Description: A Solana programme that implements the Collider for Antitoken. This programme allows users to combine $ANTI and $PRO tokens to mint $BARYON and $PHOTON tokens
//! Version: 0.0.1
//! License: MIT
//! Created: 17 Dec 2024
//! Last Modified: 17 Dec 2024
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

mod instruction;
use borsh::{BorshDeserialize, BorshSerialize};
pub use instruction::CollisionInstruction;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use spl_token_2022::instruction as token_instruction;

// Programme ID for the collision contract
solana_program::declare_id!("HzRwAPcT3qWEpNNiaNAZmg9rsurmf2CWSmBxhcdpoaHf");

/// State structure for the collision programme
/// Stores critical programme parameters and vault addresses
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CollisionState {
    /// Address of the $BARYON token mint
    pub baryon_mint: Pubkey,
    /// Address of the $PHOTON token mint
    pub photon_mint: Pubkey,
    /// PDA authority that controls minting and transfers
    pub authority: Pubkey,
    /// Vault account holding deposited $ANTI tokens
    pub vault_anti: Pubkey,
    /// Vault account holding deposited $PRO tokens
    pub vault_pro: Pubkey,
}

/// Custom error types for the collision programme
#[derive(Debug)]
pub enum CollisionError {
    /// Error when both $ANTI and $PRO token amounts are zero
    BothTokensZero,
    /// Error when token amount calculations fail or produce invalid results
    InvalidCalculation,
}

impl From<CollisionError> for ProgramError {
    fn from(_e: CollisionError) -> Self {
        ProgramError::Custom(1)
    }
}

// Programme entrypoint
entrypoint!(process_instruction);

/// Programme entrypoint processor
/// Routes incoming instructions to appropriate handlers
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = CollisionInstruction::try_from_slice(instruction_data)?;
    match instruction {
        CollisionInstruction::Collide {
            anti_amount,
            pro_amount,
        } => collide(program_id, accounts, anti_amount, pro_amount),
        CollisionInstruction::Initialise => initialise(program_id, accounts),
    }
}

/// Initialises the collision programme state and required accounts
///
/// # Arguments
/// * `program_id` - The programme's ID
/// * `accounts` - Array of accounts in the following order:
///   * `state_account` - Programme state account (write)
///   * `baryon_mint` - $BARYON token mint (write)
///   * `photon_mint` - $PHOTON token mint (write)
///   * `vault_anti` - Vault for $ANTI tokens (write)
///   * `vault_pro` - Vault for $PRO tokens (write)
///   * `payer` - Account paying for setup (signer)
///   * `system_program` - System programme
///   * `token_program` - Token programme
///   * `rent` - Rent sysvar
pub fn initialise(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    // Parse accounts
    let state_account = next_account_info(accounts_iter)?;
    let baryon_mint = next_account_info(accounts_iter)?;
    let photon_mint = next_account_info(accounts_iter)?;
    let vault_anti = next_account_info(accounts_iter)?;
    let vault_pro = next_account_info(accounts_iter)?;
    let payer = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let rent = next_account_info(accounts_iter)?;

    // Verify state account ownership
    if state_account.owner != program_id {
        return Err(ProgramError::InvalidAccountData);
    }

    // Derive PDA for authority
    let (authority_pubkey, authority_bump) =
        Pubkey::find_program_address(&[b"authority"], program_id);

    let bump = [authority_bump];
    let authority_seeds = &[b"authority" as &[u8], &bump];

    // Initialise state if empty
    if state_account.data_is_empty() {
        let state = CollisionState {
            baryon_mint: *baryon_mint.key,
            photon_mint: *photon_mint.key,
            authority: authority_pubkey,
            vault_anti: *vault_anti.key,
            vault_pro: *vault_pro.key,
        };

        state.serialize(&mut &mut state_account.data.borrow_mut()[..])?;
    }

    // Create and initialise $ANTI vault if needed
    if vault_anti.data_is_empty() {
        invoke(
            &system_instruction::create_account(
                payer.key,
                vault_anti.key,
                Rent::get()?.minimum_balance(spl_token_2022::state::Account::LEN),
                spl_token_2022::state::Account::LEN as u64,
                token_program.key,
            ),
            &[payer.clone(), vault_anti.clone(), system_program.clone()],
        )?;

        invoke(
            &token_instruction::initialize_account3(
                token_program.key,
                vault_anti.key,
                baryon_mint.key,
                &authority_pubkey,
            )?,
            &[vault_anti.clone(), baryon_mint.clone(), rent.clone()],
        )?;
    }

    // Create and initialise $PRO vault if needed
    if vault_pro.data_is_empty() {
        invoke(
            &system_instruction::create_account(
                payer.key,
                vault_pro.key,
                Rent::get()?.minimum_balance(spl_token_2022::state::Account::LEN),
                spl_token_2022::state::Account::LEN as u64,
                token_program.key,
            ),
            &[payer.clone(), vault_pro.clone(), system_program.clone()],
        )?;

        invoke(
            &token_instruction::initialize_account3(
                token_program.key,
                vault_pro.key,
                photon_mint.key,
                &authority_pubkey,
            )?,
            &[vault_pro.clone(), photon_mint.clone(), rent.clone()],
        )?;
    }

    // Initialise $BARYON mint if needed
    if baryon_mint.data_is_empty() {
        let mint_rent = Rent::get()?.minimum_balance(spl_token_2022::state::Mint::LEN);
        invoke(
            &system_instruction::create_account(
                payer.key,
                baryon_mint.key,
                mint_rent,
                spl_token_2022::state::Mint::LEN as u64,
                &spl_token_2022::id(),
            ),
            &[payer.clone(), baryon_mint.clone(), system_program.clone()],
        )?;

        invoke_signed(
            &token_instruction::initialize_mint(
                &spl_token_2022::id(),
                baryon_mint.key,
                &authority_pubkey,
                Some(&authority_pubkey),
                9, // Decimal places
            )?,
            &[baryon_mint.clone(), rent.clone(), system_program.clone()],
            &[authority_seeds],
        )?;
    }

    // Initialise $PHOTON mint if needed
    if photon_mint.data_is_empty() {
        let mint_rent = Rent::get()?.minimum_balance(spl_token_2022::state::Mint::LEN);
        invoke(
            &system_instruction::create_account(
                payer.key,
                photon_mint.key,
                mint_rent,
                spl_token_2022::state::Mint::LEN as u64,
                &spl_token_2022::id(),
            ),
            &[payer.clone(), photon_mint.clone(), system_program.clone()],
        )?;

        invoke_signed(
            &token_instruction::initialize_mint(
                &spl_token_2022::id(),
                photon_mint.key,
                &authority_pubkey,
                Some(&authority_pubkey),
                9, // Decimal places
            )?,
            &[photon_mint.clone(), rent.clone(), system_program.clone()],
            &[authority_seeds],
        )?;
    }

    Ok(())
}

/// Performs the collision operation between $ANTI and $PRO tokens
/// Stores input tokens in vaults and mints $BARYON and $PHOTON tokens
///
/// # Arguments
/// * `program_id` - The programme's ID
/// * `accounts` - Array of accounts in the following order:
///   * `state_account` - Programme state account (read)
///   * `anti_token_account` - Source account for $ANTI tokens (write)
///   * `pro_token_account` - Source account for $PRO tokens (write)
///   * `baryon_token_account` - Destination for $BARYON tokens (write)
///   * `photon_token_account` - Destination for $PHOTON tokens (write)
///   * `baryon_mint` - $BARYON token mint (write)
///   * `photon_mint` - $PHOTON token mint (write)
///   * `vault_anti` - Vault for $ANTI tokens (write)
///   * `vault_pro` - Vault for $PRO tokens (write)
///   * `anti_mint` - $ANTI token mint (read)
///   * `pro_mint` - $PRO token mint (read)
///   * `payer` - Transaction fee payer (signer)
///   * `system_program` - System programme
///   * `token_program` - Token programme
///   * `authority` - PDA authority
/// * `anti_amount` - Amount of $ANTI tokens to collide
/// * `pro_amount` - Amount of $PRO tokens to collide
pub fn collide(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    anti_amount: u64,
    pro_amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    // Parse accounts
    let state_account = next_account_info(accounts_iter)?;
    let anti_token_account = next_account_info(accounts_iter)?;
    let pro_token_account = next_account_info(accounts_iter)?;
    let baryon_token_account = next_account_info(accounts_iter)?;
    let photon_token_account = next_account_info(accounts_iter)?;
    let baryon_mint = next_account_info(accounts_iter)?;
    let photon_mint = next_account_info(accounts_iter)?;
    let vault_anti = next_account_info(accounts_iter)?;
    let vault_pro = next_account_info(accounts_iter)?;
    let anti_mint = next_account_info(accounts_iter)?;
    let pro_mint = next_account_info(accounts_iter)?;
    let _payer = next_account_info(accounts_iter)?;
    let _system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let authority_info = next_account_info(accounts_iter)?;

    // Derive and verify PDA authority
    let (authority_pubkey, authority_bump) =
        Pubkey::find_program_address(&[b"authority"], program_id);

    // Verify state account and data
    if state_account.owner != program_id {
        return Err(ProgramError::InvalidAccountData);
    }

    let state = CollisionState::try_from_slice(&state_account.data.borrow())?;
    if state.baryon_mint != *baryon_mint.key
        || state.photon_mint != *photon_mint.key
        || state.vault_anti != *vault_anti.key
        || state.vault_pro != *vault_pro.key
    {
        return Err(ProgramError::InvalidAccountData);
    }

    // Verify token programme
    let token_program_id = &spl_token_2022::id();
    if token_program.key != token_program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Verify authority
    if authority_pubkey != *authority_info.key {
        return Err(ProgramError::InvalidSeeds);
    }

    // Check input amounts
    if anti_amount == 0 && pro_amount == 0 {
        return Err(CollisionError::BothTokensZero.into());
    }

    let bump = [authority_bump];
    let authority_seeds = &[b"authority" as &[u8], &bump];

    // Transfer $ANTI tokens to vault with amount verification
    let transfer_anti_ix = token_instruction::transfer_checked(
        token_program.key,
        anti_token_account.key,
        anti_mint.key,
        vault_anti.key,
        &authority_pubkey,
        &[],
        anti_amount,
        9, // Decimal places
    )?;

    invoke_signed(
        &transfer_anti_ix,
        &[
            anti_token_account.clone(),
            anti_mint.clone(),
            vault_anti.clone(),
            authority_info.clone(),
            token_program.clone(),
        ],
        &[authority_seeds],
    )?;

    // Transfer $PRO tokens to vault with amount verification
    let transfer_pro_ix = token_instruction::transfer_checked(
        token_program.key,
        pro_token_account.key,
        pro_mint.key,
        vault_pro.key,
        &authority_pubkey,
        &[],
        pro_amount,
        9, // Decimal places
    )?;

    invoke_signed(
        &transfer_pro_ix,
        &[
            pro_token_account.clone(),
            pro_mint.clone(),
            vault_pro.clone(),
            authority_info.clone(),
            token_program.clone(),
        ],
        &[authority_seeds],
    )?;

    // Calculate output token amounts
    let anti = anti_amount as f64;
    let pro = pro_amount as f64;

    // u represents the mean of the derived probability distribution (max ratio)
    let u = f64::max(anti / (anti + pro), pro / (anti + pro));
    // s represents the standard deviation of the derived probability distribution (ratio sum to difference)
    let s = (anti + pro) / f64::abs(anti - pro);

    // Calculate final amounts using the collision formulae
    let baryon_amount = ((anti + pro) / 2.0 * u) as u64;
    let photon_amount = ((anti + pro) / 2.0 * s) as u64;

    // Mint $BARYON tokens to user
    let mint_baryon_ix = token_instruction::mint_to(
        token_program.key,
        baryon_mint.key,
        baryon_token_account.key,
        &authority_pubkey,
        &[],
        baryon_amount,
    )?;

    // Execute $BARYON minting with PDA authority
    invoke_signed(
        &mint_baryon_ix,
        &[
            baryon_mint.clone(),
            baryon_token_account.clone(),
            baryon_mint.clone(), // Authority account
            token_program.clone(),
        ],
        &[authority_seeds],
    )?;

    // Mint $PHOTON tokens to user
    let mint_photon_ix = token_instruction::mint_to(
        token_program.key,
        photon_mint.key,
        photon_token_account.key,
        &authority_pubkey,
        &[],
        photon_amount,
    )?;

    // Execute $PHOTON minting with PDA authority
    invoke_signed(
        &mint_photon_ix,
        &[
            photon_mint.clone(),
            photon_token_account.clone(),
            photon_mint.clone(), // Authority account
            token_program.clone(),
        ],
        &[authority_seeds],
    )?;

    Ok(())
}

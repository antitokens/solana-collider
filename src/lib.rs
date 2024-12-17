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

solana_program::declare_id!("6mC548mJ3rtFKcSTmQpQnLkRJ3UNzgg9qTDYpojLkvNV");

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CollisionState {
    pub baryon_mint: Pubkey,
    pub photon_mint: Pubkey,
    pub authority: Pubkey,
    pub vault_anti: Pubkey,
    pub vault_pro: Pubkey,
}

#[derive(Debug)]
pub enum CollisionError {
    BothTokensZero,
    InvalidCalculation,
}

impl From<CollisionError> for ProgramError {
    fn from(_e: CollisionError) -> Self {
        ProgramError::Custom(1)
    }
}

entrypoint!(process_instruction);

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

pub fn initialise(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let state_account = next_account_info(accounts_iter)?;
    let baryon_mint = next_account_info(accounts_iter)?;
    let photon_mint = next_account_info(accounts_iter)?;
    let vault_anti = next_account_info(accounts_iter)?;
    let vault_pro = next_account_info(accounts_iter)?;
    let payer = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let rent = next_account_info(accounts_iter)?;

    if state_account.owner != program_id {
        return Err(ProgramError::InvalidAccountData);
    }

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

    if vault_anti.data_is_empty() {
        // Create and initialise the ANTI vault account
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

    if vault_pro.data_is_empty() {
        // Create and initialise the PRO vault account
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

    // Initialise mints if needed
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
                9,
            )?,
            &[baryon_mint.clone(), rent.clone(), system_program.clone()],
            &[authority_seeds],
        )?;
    }

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
                9,
            )?,
            &[photon_mint.clone(), rent.clone(), system_program.clone()],
            &[authority_seeds],
        )?;
    }

    Ok(())
}

pub fn collide(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    anti_amount: u64,
    pro_amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

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

    // Derive PDA and verify
    let (authority_pubkey, authority_bump) =
        Pubkey::find_program_address(&[b"authority"], program_id);

    // Verify state account and its data
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

    let token_program_id = &spl_token_2022::id();
    if token_program.key != token_program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    if authority_pubkey != *authority_info.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let bump = [authority_bump];
    let authority_seeds = &[b"authority" as &[u8], &bump];

    if anti_amount == 0 && pro_amount == 0 {
        return Err(CollisionError::BothTokensZero.into());
    }

    // Transfer tokens to vault with amount verification
    let transfer_anti_ix = token_instruction::transfer_checked(
        token_program.key,
        anti_token_account.key,
        anti_mint.key,
        vault_anti.key,
        &authority_pubkey,
        &[],
        anti_amount,
        9,
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

    let transfer_pro_ix = token_instruction::transfer_checked(
        token_program.key,
        pro_token_account.key,
        pro_mint.key,
        vault_pro.key,
        &authority_pubkey,
        &[],
        pro_amount,
        9,
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

    // Convert to f64 for calculations
    let anti = anti_amount as f64;
    let pro = pro_amount as f64;

    let u = f64::max(anti / (anti + pro), pro / (anti + pro));
    let s = (anti + pro) / f64::abs(anti - pro);
    let baryon_amount = ((anti + pro) / 2.0 * u) as u64;
    let photon_amount = ((anti + pro) / 2.0 * s) as u64;

    // Mint output tokens
    let mint_baryon_ix = token_instruction::mint_to(
        token_program.key,
        baryon_mint.key,
        baryon_token_account.key,
        &authority_pubkey,
        &[],
        baryon_amount,
    )?;

    invoke_signed(
        &mint_baryon_ix,
        &[
            baryon_mint.clone(),
            baryon_token_account.clone(),
            baryon_mint.clone(),
            token_program.clone(),
        ],
        &[authority_seeds],
    )?;

    let mint_photon_ix = token_instruction::mint_to(
        token_program.key,
        photon_mint.key,
        photon_token_account.key,
        &authority_pubkey,
        &[],
        photon_amount,
    )?;

    invoke_signed(
        &mint_photon_ix,
        &[
            photon_mint.clone(),
            photon_token_account.clone(),
            photon_mint.clone(),
            token_program.clone(),
        ],
        &[authority_seeds],
    )?;

    Ok(())
}

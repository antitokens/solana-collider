mod instruction;
pub use instruction::CollisionInstruction;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program::{invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    program_pack::Pack,
};
use spl_token::instruction as token_instruction;
use borsh::{BorshDeserialize, BorshSerialize};

// Declare the program ID
solana_program::declare_id!("3K81PoodXnhxB9XUQM6ZtRhshdrkDDFwVbABV8PgPziw");

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CollisionState {
    pub baryon_mint: Pubkey,
    pub photon_mint: Pubkey,
    pub authority: Pubkey,
}

// Custom errors
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

// Program entrypoint
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    
    let anti_token_account = next_account_info(accounts_iter)?;
    let pro_token_account = next_account_info(accounts_iter)?;
    let _baryon_token_account = next_account_info(accounts_iter)?;
    let _photon_token_account = next_account_info(accounts_iter)?;
    let baryon_mint = next_account_info(accounts_iter)?;
    let photon_mint = next_account_info(accounts_iter)?;
    let payer = next_account_info(accounts_iter)?;
    let _system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Parse amounts from token accounts
    let anti_amount = spl_token::state::Account::unpack(&anti_token_account.data.borrow())?.amount;
    let pro_amount = spl_token::state::Account::unpack(&pro_token_account.data.borrow())?.amount;

    if anti_amount == 0 && pro_amount == 0 {
        return Err(CollisionError::BothTokensZero.into());
    }

    // Convert to f64 for calculations
    let anti = anti_amount as f64;
    let pro = pro_amount as f64;

    // Step 1: Calculate u (mean)
    let u = f64::max(anti / (anti + pro), pro / (anti + pro));

    // Step 2: Calculate s (standard deviation)
    let s = (anti + pro) / f64::abs(anti - pro);

    // Calculate BARYON and PHOTON amounts
    let baryon_amount = ((anti + pro) / 2.0 * u) as u64;
    let photon_amount = ((anti + pro) / 2.0 * s) as u64;

    // Mint BARYON tokens
    let mint_baryon_ix = token_instruction::mint_to(
        token_program.key,
        baryon_mint.key,
        &payer.key,
        &program_id,
        &[],
        baryon_amount,
    )?;

    invoke_signed(
        &mint_baryon_ix,
        &[
            baryon_mint.clone(),
            payer.clone(),
            token_program.clone(),
        ],
        &[&[&program_id.to_bytes()]],
    )?;

    // Mint PHOTON tokens
    let mint_photon_ix = token_instruction::mint_to(
        token_program.key,
        photon_mint.key,
        &payer.key,
        &program_id,
        &[],
        photon_amount,
    )?;

    invoke_signed(
        &mint_photon_ix,
        &[
            photon_mint.clone(),
            payer.clone(),
            token_program.clone(),
        ],
        &[&[&program_id.to_bytes()]],
    )?;

    // Burn input tokens
    let burn_anti_ix = token_instruction::burn(
        token_program.key,
        anti_token_account.key,
        &payer.key,
        &program_id,
        &[],
        anti_amount,
    )?;

    let burn_pro_ix = token_instruction::burn(
        token_program.key,
        pro_token_account.key,
        &payer.key,
        &program_id,
        &[],
        pro_amount,
    )?;

    invoke_signed(
        &burn_anti_ix,
        &[
            anti_token_account.clone(),
            payer.clone(),
            token_program.clone(),
        ],
        &[&[&program_id.to_bytes()]],
    )?;

    invoke_signed(
        &burn_pro_ix,
        &[
            pro_token_account.clone(),
            payer.clone(),
            token_program.clone(),
        ],
        &[&[&program_id.to_bytes()]],
    )?;

    Ok(())
}

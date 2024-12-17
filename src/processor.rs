//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's processor
//! Version: 0.0.1
//! License: MIT
//! Created: 17 Dec 2024
//! Last Modified: 17 Dec 2024
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

use crate::{error::CollisionError, instruction::CollisionInstruction, state::CollisionState};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use spl_token_2022::instruction as token_instruction;

pub struct Processor;

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = CollisionInstruction::try_from_slice(instruction_data)?;

        match instruction {
            CollisionInstruction::Initialise => Self::process_initialise(accounts, program_id),
            CollisionInstruction::Collide {
                anti_amount,
                pro_amount,
            } => Self::process_collide(accounts, anti_amount, pro_amount, program_id),
        }
    }

    fn process_initialise(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let state_account = next_account_info(accounts_iter)?;
        let baryon_mint = next_account_info(accounts_iter)?;
        let photon_mint = next_account_info(accounts_iter)?;
        let vault_anti = next_account_info(accounts_iter)?;
        let vault_pro = next_account_info(accounts_iter)?;
        let payer = next_account_info(accounts_iter)?;
        let system_program = next_account_info(accounts_iter)?;
        let token_program = next_account_info(accounts_iter)?;
        let rent_sysvar = next_account_info(accounts_iter)?;

        // Verify the payer signed the transaction
        if !payer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let (authority_pubkey, _authority_bump) =
            Pubkey::find_program_address(&[b"authority"], program_id);

        // Create and initialise state account
        let rent = &Rent::from_account_info(rent_sysvar)?;
        let space = std::mem::size_of::<CollisionState>();
        let rent_lamports = rent.minimum_balance(space);

        invoke(
            &system_instruction::create_account(
                payer.key,
                state_account.key,
                rent_lamports,
                space as u64,
                program_id,
            ),
            &[payer.clone(), state_account.clone(), system_program.clone()],
        )?;

        // Initialise state
        let state = CollisionState {
            baryon_mint: *baryon_mint.key,
            photon_mint: *photon_mint.key,
            authority: authority_pubkey,
            vault_anti: *vault_anti.key,
            vault_pro: *vault_pro.key,
        };

        CollisionState::pack(state, &mut state_account.data.borrow_mut())?;

        // Create and initialise vault accounts if needed
        if vault_anti.data_is_empty() {
            invoke(
                &system_instruction::create_account(
                    payer.key,
                    vault_anti.key,
                    rent.minimum_balance(spl_token_2022::state::Account::LEN),
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
                &[vault_anti.clone(), baryon_mint.clone(), rent_sysvar.clone()],
            )?;
        }

        if vault_pro.data_is_empty() {
            invoke(
                &system_instruction::create_account(
                    payer.key,
                    vault_pro.key,
                    rent.minimum_balance(spl_token_2022::state::Account::LEN),
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
                &[vault_pro.clone(), photon_mint.clone(), rent_sysvar.clone()],
            )?;
        }

        Ok(())
    }

    fn process_collide(
        accounts: &[AccountInfo],
        anti_amount: u64,
        pro_amount: u64,
        program_id: &Pubkey,
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
        let payer = next_account_info(accounts_iter)?;
        let system_program = next_account_info(accounts_iter)?;
        let token_program = next_account_info(accounts_iter)?;
        let authority = next_account_info(accounts_iter)?;

        // Verify the payer signed the transaction
        if !payer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Verify state account and load data
        if state_account.owner != program_id {
            return Err(ProgramError::InvalidAccountData);
        }

        let state = CollisionState::unpack(&state_account.data.borrow())?;
        if state.baryon_mint != *baryon_mint.key
            || state.photon_mint != *photon_mint.key
            || state.vault_anti != *vault_anti.key
            || state.vault_pro != *vault_pro.key
        {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check for zero amounts
        if anti_amount == 0 && pro_amount == 0 {
            return Err(CollisionError::BothTokensZero.into());
        }

        // Calculate collided amounts
        let anti = anti_amount as f64;
        let pro = pro_amount as f64;

        let u = f64::max(anti / (anti + pro), pro / (anti + pro));
        let s = (anti + pro) / f64::abs(anti - pro);
        let baryon_amount = ((anti + pro) / 2.0 * u) as u64;
        let photon_amount = ((anti + pro) / 2.0 * s) as u64;

        // Derive PDA for authority
        let (authority_pubkey, authority_bump) =
            Pubkey::find_program_address(&[b"authority"], program_id);
        let authority_seeds = &[b"authority" as &[u8], &[authority_bump]];

        // Transfer tokens to vaults using transfer_checked
        invoke_signed(
            &token_instruction::transfer_checked(
                token_program.key,
                anti_token_account.key,
                anti_mint.key,
                vault_anti.key,
                &authority_pubkey,
                &[],
                anti_amount,
                9, // Decimals
            )?,
            &[
                anti_token_account.clone(),
                anti_mint.clone(),
                vault_anti.clone(),
                authority.clone(),
                token_program.clone(),
            ],
            &[authority_seeds],
        )?;

        invoke_signed(
            &token_instruction::transfer_checked(
                token_program.key,
                pro_token_account.key,
                pro_mint.key,
                vault_pro.key,
                &authority_pubkey,
                &[],
                pro_amount,
                9, // Decimals
            )?,
            &[
                pro_token_account.clone(),
                pro_mint.clone(),
                vault_pro.clone(),
                authority.clone(),
                token_program.clone(),
            ],
            &[authority_seeds],
        )?;

        // Mint output tokens
        invoke_signed(
            &token_instruction::mint_to(
                token_program.key,
                baryon_mint.key,
                baryon_token_account.key,
                &authority_pubkey,
                &[],
                baryon_amount,
            )?,
            &[
                baryon_mint.clone(),
                baryon_token_account.clone(),
                baryon_mint.clone(),
                token_program.clone(),
            ],
            &[authority_seeds],
        )?;

        invoke_signed(
            &token_instruction::mint_to(
                token_program.key,
                photon_mint.key,
                photon_token_account.key,
                &authority_pubkey,
                &[],
                photon_amount,
            )?,
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
}

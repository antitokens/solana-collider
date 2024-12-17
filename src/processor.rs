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
            CollisionInstruction::Initialise => {
                Self::process_initialise(accounts, program_id)
            }
            CollisionInstruction::Collide { anti_amount, pro_amount } => {
                Self::process_collide(accounts, anti_amount, pro_amount, program_id)
            }
        }
    }

    fn process_initialise(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        
        let Initialiser = next_account_info(accounts_iter)?;
        let state_account = next_account_info(accounts_iter)?;
        let baryon_mint = next_account_info(accounts_iter)?;
        let photon_mint = next_account_info(accounts_iter)?;
        let system_program = next_account_info(accounts_iter)?;

        // Verify the Initialiser signed the transaction
        if !Initialiser.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Create state account
        let rent = Rent::get()?;
        let space = CollisionState::get_packed_len();
        let rent_lamports = rent.minimum_balance(space);

        invoke(
            &system_instruction::create_account(
                Initialiser.key,
                state_account.key,
                rent_lamports,
                space as u64,
                program_id,
            ),
            &[
                Initialiser.clone(),
                state_account.clone(),
                system_program.clone(),
            ],
        )?;

        // Initialise state
        let state = CollisionState {
            baryon_mint: *baryon_mint.key,
            photon_mint: *photon_mint.key,
            authority: *Initialiser.key,
        };

        CollisionState::pack(state, &mut state_account.data.borrow_mut())?;

        Ok(())
    }

    fn process_collide(
        accounts: &[AccountInfo],
        anti_amount: u64,
        pro_amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let user = next_account_info(accounts_iter)?;
        let anti_token_account = next_account_info(accounts_iter)?;
        let pro_token_account = next_account_info(accounts_iter)?;
        let baryon_token_account = next_account_info(accounts_iter)?;
        let photon_token_account = next_account_info(accounts_iter)?;
        let baryon_mint = next_account_info(accounts_iter)?;
        let photon_mint = next_account_info(accounts_iter)?;
        let token_program = next_account_info(accounts_iter)?;

        // Verify signer
        if !user.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Check for zero amounts
        if anti_amount == 0 && pro_amount == 0 {
            return Err(CollisionError::BothTokensZero.into());
        }

        // Calculate collided amounts
        let anti = anti_amount as f64;
        let pro = pro_amount as f64;

        // Calculate u (mean)
        let u = f64::max(anti / (anti + pro), pro / (anti + pro));

        // Calculate s (standard deviation)
        let s = (anti + pro) / f64::abs(anti - pro);

        // Calculate output amounts
        let baryon_amount = ((anti + pro) / 2.0 * u) as u64;
        let photon_amount = ((anti + pro) / 2.0 * s) as u64;

        // Burn input tokens
        invoke(
            &token_instruction::burn(
                token_program.key,
                anti_token_account.key,
                user.key,
                user.key,
                &[],
                anti_amount,
            )?,
            &[
                anti_token_account.clone(),
                user.clone(),
                token_program.clone(),
            ],
        )?;

        invoke(
            &token_instruction::burn(
                token_program.key,
                pro_token_account.key,
                user.key,
                user.key,
                &[],
                pro_amount,
            )?,
            &[
                pro_token_account.clone(),
                user.clone(),
                token_program.clone(),
            ],
        )?;

        // Mint output tokens
        let seeds = &[program_id.as_ref()];
        let (authority_pubkey, bump_seed) = Pubkey::find_program_address(seeds, program_id);
        let authority_seeds = &[program_id.as_ref(), &[bump_seed]];

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
                token_program.clone(),
            ],
            &[authority_seeds],
        )?;

        Ok(())
    }
}
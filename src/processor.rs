use crate::{error::CollisionError, instruction::CollisionInstruction, state::CollisionState};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

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
        // Implementation here
        Ok(())
    }

    fn process_collide(
        accounts: &[AccountInfo],
        anti_amount: u64,
        pro_amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        // Implementation here
        Ok(())
    }
}

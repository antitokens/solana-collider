use thiserror::Error;
use solana_program::program_error::ProgramError;

#[derive(Error, Debug)]
pub enum CollisionError {
    #[error("Both ANTI and PRO tokens cannot be zero")]
    BothTokensZero,
}

impl From<CollisionError> for ProgramError {
    fn from(e: CollisionError) -> Self {
        ProgramError::Custom(1)
    }
}
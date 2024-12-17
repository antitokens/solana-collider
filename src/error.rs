use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CollisionError {
    #[error("Both ANTI and PRO tokens cannot be zero")]
    BothTokensZero,

    #[error("Invalid vault account")]
    InvalidVault,

    #[error("Invalid authority")]
    InvalidAuthority,

    #[error("Invalid token mint")]
    InvalidMint,

    #[error("Invalid state account")]
    InvalidState,

    #[error("Calculation error")]
    InvalidCalculation,

    #[error("Transfer failed")]
    TransferFailed,
}

impl From<CollisionError> for ProgramError {
    fn from(e: CollisionError) -> Self {
        ProgramError::Custom(match e {
            CollisionError::BothTokensZero => 1,
            CollisionError::InvalidVault => 2,
            CollisionError::InvalidAuthority => 3,
            CollisionError::InvalidMint => 4,
            CollisionError::InvalidState => 5,
            CollisionError::InvalidCalculation => 6,
            CollisionError::TransferFailed => 7,
        })
    }
}

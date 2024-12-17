//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's errors
//! Version: 0.0.1
//! License: MIT
//! Created: 17 Dec 2024
//! Last Modified: 17 Dec 2024
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

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

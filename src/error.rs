use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum CollisionError {
    #[error("Both ANTI and PRO tokens cannot be zero")]
    BothTokensZero,
    #[error("Invalid calculation result")]
    InvalidCalculation,
}
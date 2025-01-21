//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 0.0.1
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

// instructions/mod.rs
use anchor_lang::prelude::*;

pub mod create_poll;
pub mod deposit;
pub mod equalise;
pub mod initialise;
pub mod withdraw;

// Re-export the instruction structs
pub use create_poll::creator;
pub use deposit::depositor;
pub use equalise::equaliser;
pub use initialise::initialiser;
pub use withdraw::withdrawer;

// Add instruction data structs
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreatePollArgs {
    pub title: String,
    pub description: String,
    pub start_time: String,
    pub end_time: String,
    pub etc: Option<Vec<u8>>,
}

// Add other instruction args structs as needed

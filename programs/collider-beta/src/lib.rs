//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's core functions
//! Version: 0.0.1
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

// lib.rs
use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod utils;

declare_id!("5eR98MdgS8jYpKB2iD9oz3MtBdLJ6s7gAVWJZFMvnL9G");

#[program]
pub mod collider_beta {
    use super::*;

    pub fn initialize(ctx: Context<Initialise>) -> Result<()> {
        instructions::initialise::handler(ctx)
    }

    pub fn create_poll(
        ctx: Context<CreatePoll>,
        title: String,
        description: String,
        start_time: String,
        end_time: String,
        etc: Option<Vec<u8>>,
    ) -> Result<()> {
        instructions::create_poll::handler(ctx, title, description, start_time, end_time, etc)
    }

    pub fn deposit_tokens(
        ctx: Context<DepositTokens>,
        poll_index: u64,
        anti_amount: u64,
        pro_amount: u64,
    ) -> Result<()> {
        instructions::deposit::handler(ctx, poll_index, anti_amount, pro_amount)
    }

    pub fn equalise(
        ctx: Context<EqualiseTokens>,
        poll_index: u64,
        truth_values: Vec<u64>,
    ) -> Result<()> {
        instructions::equalise::handler(ctx, poll_index, truth_values)
    }

    pub fn withdraw_tokens(ctx: Context<WithdrawTokens>, poll_index: u64) -> Result<()> {
        instructions::withdraw::handler(ctx, poll_index)
    }
}

// Re-export common types for convenience
pub use instructions::{CreatePoll, DepositTokens, EqualiseTokens, Initialise, WithdrawTokens};
pub use state::{EqualisationResult, PollAccount, StateAccount, UserDeposit};
pub use utils::{DepositEvent, EqualisationEvent, PollCreatedEvent, PredictError};

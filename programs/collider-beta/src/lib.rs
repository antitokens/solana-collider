//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider core
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

pub mod instructions;
pub mod state;
pub mod utils;

declare_id!("5eR98MdgS8jYpKB2iD9oz3MtBdLJ6s7gAVWJZFMvnL9G");

#[program]
pub mod collider_beta {
    use super::*;
    use crate::instructions::create_poll;
    use crate::instructions::initialise;
    use instructions::deposit;
    use instructions::equalise;
    use instructions::withdraw;

    pub fn initialiser(ctx: Context<Initialise>) -> Result<()> {
        initialise::initialise(ctx)
    }

    pub fn create_poll(
        ctx: Context<CreatePoll>,
        title: String,
        description: String,
        start_time: String,
        end_time: String,
        etc: Option<Vec<u8>>,
        unix_timestamp: Option<i64>, // CRITICAL: Remove in production
    ) -> Result<()> {
        create_poll::create(
            ctx,
            title,
            description,
            start_time,
            end_time,
            etc,
            unix_timestamp, // CRITICAL: Remove in production
        )
    }

    pub fn deposit_tokens(
        ctx: Context<DepositTokens>,
        poll_index: u64,
        anti: u64,
        pro: u64,
        unix_timestamp: Option<i64>, // CRITICAL: Remove in production
    ) -> Result<()> {
        deposit::deposit(
            ctx,
            poll_index,
            anti,
            pro,
            unix_timestamp, // CRITICAL: Remove in production
        )
    }

    pub fn equalise_tokens(
        ctx: Context<EqualiseTokens>,
        poll_index: u64,
        truth: Vec<u64>,
        unix_timestamp: Option<i64>, // CRITICAL: Remove in production
    ) -> Result<()> {
        equalise::equalise(
            ctx,
            poll_index,
            truth,
            unix_timestamp, // CRITICAL: Remove in production
        )
    }

    pub fn withdraw_tokens(ctx: Context<WithdrawTokens>, poll_index: u64) -> Result<()> {
        withdraw::withdraw(ctx, poll_index)
    }
}

#[derive(Accounts)]
pub struct Initialise<'info> {
    #[account(init, payer = authority, space = 8 + StateAccount::LEN, seeds = [b"state"], bump)]
    pub state: Account<'info, StateAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(title: String, description: String, start_time: String, end_time: String)]
pub struct CreatePoll<'info> {
    #[account(mut)]
    pub state: Account<'info, StateAccount>,
    #[account(
        init,
        payer = authority,
        space = 8 + PollAccount::LEN,
        seeds = [b"poll", state.poll_index.to_le_bytes().as_ref()],
        bump
    )]
    pub poll: Account<'info, PollAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub payment: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

impl Default for PollAccount {
    fn default() -> Self {
        Self {
            index: 0,
            title: String::new(),
            description: String::new(),
            start_time: String::new(),
            end_time: String::new(),
            etc: None,
            anti: 0,
            pro: 0,
            deposits: vec![],
            equalised: false,
            equalisation_results: None,
        }
    }
}

#[derive(Accounts)]
#[instruction(poll_index: u64)]
pub struct DepositTokens<'info> {
    #[account(
        mut,
        seeds = [b"poll", poll_index.to_le_bytes().as_ref()],
        bump
    )]
    pub poll: Account<'info, PollAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        constraint = user_anti_token.owner == authority.key() @ PredictError::InvalidTokenAccount,
        constraint = user_anti_token.mint == poll_anti_token.mint @ PredictError::InvalidTokenAccount
    )]
    pub user_anti_token: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = user_pro_token.owner == authority.key() @ PredictError::InvalidTokenAccount,
        constraint = user_pro_token.mint == poll_pro_token.mint @ PredictError::InvalidTokenAccount
    )]
    pub user_pro_token: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = poll_anti_token.owner == poll.key() @ PredictError::InvalidTokenAccount
    )]
    pub poll_anti_token: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = poll_pro_token.owner == poll.key() @ PredictError::InvalidTokenAccount
    )]
    pub poll_pro_token: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct EqualiseTokens<'info> {
    #[account(mut)]
    pub poll: Account<'info, PollAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub user_anti_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_pro_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub poll_anti_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub poll_pro_token: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(poll_index: u64)]
pub struct WithdrawTokens<'info> {
    #[account(
        mut,
        seeds = [b"poll", poll_index.to_le_bytes().as_ref()],
        bump
    )]
    pub poll: Account<'info, PollAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        constraint = user_anti_token.owner == authority.key() @ PredictError::InvalidTokenAccount
    )]
    pub user_anti_token: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = user_pro_token.owner == authority.key() @ PredictError::InvalidTokenAccount
    )]
    pub user_pro_token: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = poll_anti_token.owner == poll.key() @ PredictError::InvalidTokenAccount
    )]
    pub poll_anti_token: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = poll_pro_token.owner == poll.key() @ PredictError::InvalidTokenAccount
    )]
    pub poll_pro_token: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

// Re-export common types for convenience
pub use state::{EqualisationResult, PollAccount, StateAccount, UserDeposit};
pub use utils::{DepositEvent, EqualisationEvent, PollCreatedEvent, PredictError};

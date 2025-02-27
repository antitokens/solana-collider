//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider core
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

use crate::utils::ANTI_MINT_ADDRESS;
use crate::utils::PRO_MINT_ADDRESS;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};


pub mod instructions;
pub mod state;
pub mod utils;

declare_id!("5eR98MdgS8jYpKB2iD9oz3MtBdLJ6s7gAVWJZFMvnL9G");

#[program]
pub mod collider_beta {
    use super::*;
    use crate::instructions::admin_actions;
    use crate::instructions::create_poll;
    use crate::instructions::initialise_program;
    use instructions::deposit_to_poll;
    use instructions::equalise_poll;
    use instructions::bulk_withdraw_from_poll;
    use instructions::user_withdraw_from_poll;

    pub fn initialise_admin(ctx: Context<Admin>) -> Result<()> {
        admin_actions::initialise_admin(ctx)
    }
    
    pub fn update_poll_creation_fee(ctx: Context<Update>, new_fee: u64) -> Result<()> {
        admin_actions::update_poll_creation_fee(ctx, new_fee)
    }
    
    pub fn update_max_title_length(ctx: Context<Update>, new_length: u64) -> Result<()> {
        admin_actions::update_max_title_length(ctx, new_length)
    }

    pub fn update_max_description_length(ctx: Context<Update>, new_length: u64) -> Result<()> {
        admin_actions::update_max_description_length(ctx, new_length)
    }
    
    pub fn update_truth_basis(ctx: Context<Update>, new_basis: u64) -> Result<()> {
        admin_actions::update_truth_basis(ctx, new_basis)
    }
    
    pub fn update_float_basis(ctx: Context<Update>, new_basis: u64) -> Result<()> {
        admin_actions::update_float_basis(ctx, new_basis)
    }
    
    pub fn update_min_deposit_amount(ctx: Context<Update>, new_min_amount: u64) -> Result<()> {
        admin_actions::update_min_deposit_amount(ctx, new_min_amount)
    }
    
    pub fn update_anti_mint(ctx: Context<Update>, new_mint: Pubkey) -> Result<()> {
        admin_actions::update_anti_mint(ctx, new_mint)
    }
    
    pub fn update_pro_mint(ctx: Context<Update>, new_mint: Pubkey) -> Result<()> {
        admin_actions::update_pro_mint(ctx, new_mint)
    }
    
    pub fn update_multisig(ctx: Context<Update>, new_multisig: Pubkey) -> Result<()> {
        admin_actions::update_multisig(ctx, new_multisig)
    }

    pub fn set_authority(ctx: Context<SetPollTokenAuthority>, poll_index: u64) -> Result<()> {
        admin_actions::set_token_authority(ctx, poll_index)
    }

    pub fn initialiser(ctx: Context<Initialise>) -> Result<()> {
        initialise_program::initialise(ctx)
    }

    pub fn create_poll(
        ctx: Context<CreatePoll>,
        title: String,
        description: String,
        start_time: String,
        end_time: String,
        etc: Option<Vec<u8>>,
        unix_timestamp: Option<i64>, // CRITICAL: Remove line in production!
    ) -> Result<()> {
        create_poll::create(
            ctx,
            title,
            description,
            start_time,
            end_time,
            etc,
            unix_timestamp, // CRITICAL: Remove line in production!
        )
    }

    pub fn deposit_tokens(
        ctx: Context<DepositTokens>,
        poll_index: u64,
        anti: u64,
        pro: u64,
        unix_timestamp: Option<i64>, // CRITICAL: Remove line in production!
    ) -> Result<()> {
        deposit_to_poll::deposit(
            ctx,
            poll_index,
            anti,
            pro,
            unix_timestamp, // CRITICAL: Remove line in production!
        )
    }

    pub fn equalise_tokens(
        ctx: Context<EqualiseTokens>,
        poll_index: u64,
        truth: Vec<u64>,
        unix_timestamp: Option<i64>, // CRITICAL: Remove line in production!
    ) -> Result<()> {
        equalise_poll::equalise(
            ctx,
            poll_index,
            truth,
            unix_timestamp, // CRITICAL: Remove line in production!
        )
    }

    pub fn bulk_withdraw_tokens<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, BulkWithdrawTokens<'info>>,
        poll_index: u64,
    ) -> Result<()> {
        bulk_withdraw_from_poll::bulk_withdraw(ctx, poll_index)
    }

    pub fn user_withdraw_tokens<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, UserWithdrawTokens<'info>>,
        poll_index: u64,
    ) -> Result<()> {
        user_withdraw_from_poll::user_withdraw(ctx, poll_index)
    }
}

#[derive(Accounts)]
pub struct Admin<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + AdminAccount::LEN as usize,
        seeds = [b"admin"],
        bump
    )]
    pub admin: Account<'info, AdminAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut, seeds = [b"admin"], bump)]
    pub admin: Account<'info, AdminAccount>,
    #[account(signer)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Initialise<'info> {
    #[account(init, payer = authority, space = 8 + StateAccount::LEN as usize, seeds = [b"state"], bump)]
    pub state: Account<'info, StateAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(title: String, description: String, start_time: String, end_time: String)]
pub struct CreatePoll<'info> {
    #[account(
        mut,
        seeds = [b"state"], 
        bump,
        owner = crate::ID, 
        constraint = state.to_account_info().data_len() >= 8 + StateAccount::LEN as usize
    )]
    pub state: Account<'info, StateAccount>,
    #[account(
        init,
        payer = authority,
        space = 8 + PollAccount::LEN as usize,
        seeds = [b"poll", state.poll_index.to_le_bytes().as_ref()],
        bump
    )]
    pub poll: Account<'info, PollAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        token::mint = anti_mint,
        token::authority = authority,
        seeds = [b"anti_token", state.poll_index.to_le_bytes().as_ref()],
        bump
    )]
    pub poll_anti_token: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = authority,
        token::mint = pro_mint,
        token::authority = authority,
        seeds = [b"pro_token", state.poll_index.to_le_bytes().as_ref()],
        bump
    )]
    pub poll_pro_token: Account<'info, TokenAccount>,
    #[account(constraint = anti_mint.key() == ANTI_MINT_ADDRESS @ PredictError::InvalidTokenAccount)]
    /// CHECK: This is Antitoken CA
    pub anti_mint: AccountInfo<'info>,
    #[account(constraint = pro_mint.key() == PRO_MINT_ADDRESS @ PredictError::InvalidTokenAccount)]
    /// CHECK: This is Protoken CA
    pub pro_mint: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    #[account(mut, address = ANTITOKEN_MULTISIG @ PredictError::InvalidTokenAccount)]
    /// CHECK: This is Antitoken squads vault
    pub vault: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
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
        seeds = [b"anti_token", poll.index.to_le_bytes().as_ref()],
        bump,
        constraint = poll_anti_token.owner == ANTITOKEN_MULTISIG @ PredictError::InvalidTokenAccount
    )]
    pub poll_anti_token: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"pro_token", poll.index.to_le_bytes().as_ref()],
        bump,
        constraint = poll_pro_token.owner == ANTITOKEN_MULTISIG @ PredictError::InvalidTokenAccount
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
pub struct BulkWithdrawTokens<'info> {
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
        seeds = [b"anti_token", poll_index.to_le_bytes().as_ref()],
        bump,
        constraint = poll_anti_token.owner == ANTITOKEN_MULTISIG @ PredictError::InvalidTokenAccount
    )]
    pub poll_anti_token: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"pro_token", poll_index.to_le_bytes().as_ref()],
        bump,
        constraint = poll_pro_token.owner == ANTITOKEN_MULTISIG @ PredictError::InvalidTokenAccount
    )]
    pub poll_pro_token: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}


#[derive(Accounts)]
#[instruction(poll_index: u64)]
pub struct UserWithdrawTokens<'info> {
    #[account(
        mut,
        seeds = [b"state"],
        bump,
    )]
    pub state: Account<'info, StateAccount>,
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
        seeds = [b"anti_token", poll_index.to_le_bytes().as_ref()],
        bump,
    )]
    pub poll_anti_token: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"pro_token", poll_index.to_le_bytes().as_ref()],
        bump,
    )]
    pub poll_pro_token: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    /// CHECK: This is Antitoken squads vault 
    pub vault: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(poll_index: u64)]
pub struct SetPollTokenAuthority<'info> {
    #[account(
        seeds = [b"state"],
        bump
    )]
    pub state: Account<'info, StateAccount>,
    pub poll: Account<'info, PollAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"anti_token", poll_index.to_le_bytes().as_ref()],
        bump,
        constraint = poll_anti_token.owner == ANTITOKEN_MULTISIG @ PredictError::Unauthorised
    )]
    pub poll_anti_token: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"pro_token", poll_index.to_le_bytes().as_ref()],
        bump,
        constraint = poll_pro_token.owner == ANTITOKEN_MULTISIG @ PredictError::Unauthorised
    )]
    pub poll_pro_token: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

// Re-export common types for convenience
use state::AdminAccount;
use utils::ANTITOKEN_MULTISIG;
pub use state::{EqualisationResult, PollAccount, StateAccount, UserDeposit};
pub use utils::{DepositEvent, EqualisationEvent, PollCreatedEvent, PredictError};

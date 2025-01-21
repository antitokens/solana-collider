//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 0.0.1
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

// instructions/deposit.rs
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::utils::*;

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
        constraint = user_anti_token.owner == authority.key() @ VotingError::InvalidTokenAccount,
        constraint = user_anti_token.mint == poll_anti_token.mint @ VotingError::InvalidTokenAccount
    )]
    pub user_anti_token: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = user_pro_token.owner == authority.key() @ VotingError::InvalidTokenAccount,
        constraint = user_pro_token.mint == poll_pro_token.mint @ VotingError::InvalidTokenAccount
    )]
    pub user_pro_token: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = poll_anti_token.owner == poll.key() @ VotingError::InvalidTokenAccount
    )]
    pub poll_anti_token: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = poll_pro_token.owner == poll.key() @ VotingError::InvalidTokenAccount
    )]
    pub poll_pro_token: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

pub fn handler(
    ctx: Context<DepositTokens>,
    poll_index: u64,
    anti_amount: u64,
    pro_amount: u64,
) -> Result<()> {
    let poll = &mut ctx.accounts.poll;
    
    // Verify poll is active
    let clock = Clock::get()?;
    require!(
        poll.is_active(clock.unix_timestamp),
        VotingError::PollInactive
    );

    // Verify minimum deposit
    require!(
        anti_amount >= MIN_DEPOSIT_AMOUNT || pro_amount >= MIN_DEPOSIT_AMOUNT,
        VotingError::InsufficientDeposit
    );

    // Transfer ANTI tokens if amount > 0
    if anti_amount > 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_anti_token.to_account_info(),
                    to: ctx.accounts.poll_anti_token.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            anti_amount,
        )?;
    }

    // Transfer PRO tokens if amount > 0
    if pro_amount > 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_pro_token.to_account_info(),
                    to: ctx.accounts.poll_pro_token.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            pro_amount,
        )?;
    }

    // Calculate metrics (u and s values)
    let (u_value, s_value) = calculate_metrics(anti_amount, pro_amount, false)?;

    // Create deposit record
    let deposit = UserDeposit {
        user: ctx.accounts.authority.key(),
        anti_amount,
        pro_amount,
        u_value,
        s_value,
        withdrawn: false,
    };

    // Update poll state
    poll.deposits.push(deposit);
    poll.total_anti = poll.total_anti.checked_add(anti_amount)
        .ok_or(error!(VotingError::MathError))?;
    poll.total_pro = poll.total_pro.checked_add(pro_amount)
        .ok_or(error!(VotingError::MathError))?;

    // Emit deposit event
    emit!(DepositEvent {
        poll_index,
        depositor: ctx.accounts.authority.key(),
        anti_amount,
        pro_amount,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::solana_program::clock::Clock;

    #[test]
    fn test_deposit_validation() {
        // Mock test environment here
        // Test minimum deposit validation
        // Test token account ownership
        // Test poll active status
    }

    #[test]
    fn test_deposit_calculation() {
        // Test metric calculations
        // Test state updates
        // Test event emission
    }
}
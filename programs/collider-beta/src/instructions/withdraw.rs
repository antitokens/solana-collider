//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 0.0.1
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

use crate::state::*;
use crate::utils::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct WithdrawTokens<'info> {
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

pub fn handler(ctx: Context<WithdrawTokens>, poll_index: u64) -> Result<()> {
    let poll = &mut ctx.accounts.poll;

    // Verify poll has been equalised
    require!(poll.equalised, VotingError::NotEqualised);

    // Find user's deposit and returns
    let deposit_index = poll
        .deposits
        .iter()
        .position(|d| d.user == ctx.accounts.authority.key())
        .ok_or(error!(VotingError::NoDeposit))?;

    let equalisation = poll
        .equalisation_results
        .as_ref()
        .ok_or(error!(VotingError::NotEqualised))?;

    // Get return amounts
    let anti_return = equalisation.anti_returns[deposit_index];
    let pro_return = equalisation.pro_returns[deposit_index];

    // Transfer ANTI tokens
    if anti_return > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.poll_anti_token.to_account_info(),
                    to: ctx.accounts.user_anti_token.to_account_info(),
                    authority: ctx.accounts.poll.to_account_info(),
                },
                &[&[
                    b"poll",
                    poll_index.to_le_bytes().as_ref(),
                    &[ctx.bumps.poll],
                ]],
            ),
            anti_return,
        )?;
    }

    // Transfer PRO tokens
    if pro_return > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.poll_pro_token.to_account_info(),
                    to: ctx.accounts.user_pro_token.to_account_info(),
                    authority: ctx.accounts.poll.to_account_info(),
                },
                &[&[
                    b"poll",
                    poll_index.to_le_bytes().as_ref(),
                    &[ctx.bumps.poll],
                ]],
            ),
            pro_return,
        )?;
    }

    // Update user's withdrawal status
    poll.deposits[deposit_index].withdrawn = true;

    // Emit withdrawal event
    emit!(WithdrawEvent {
        poll_index,
        user: ctx.accounts.authority.key(),
        anti_amount: anti_return,
        pro_amount: pro_return,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

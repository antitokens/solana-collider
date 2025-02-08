//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

// instructions/deposit.rs
use crate::state::*;
use crate::utils::*;
use crate::DepositTokens;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

pub fn deposit(
    ctx: Context<DepositTokens>,
    poll_index: u64,
    anti: u64,
    pro: u64,
) -> Result<()> {
    let poll = &mut ctx.accounts.poll;

    // Get current time, supporting local testing override
let now = Clock::get()?.unix_timestamp;

    // Verify poll is active
    require!(poll.is_active(now), PredictError::PollInactive);

    // Verify minimum deposit
    require!(
        anti >= MIN_DEPOSIT_AMOUNT || pro >= MIN_DEPOSIT_AMOUNT,
        PredictError::InsufficientDeposit
    );

    // Check poll token account authorities
    require!(
        ctx.accounts.poll_anti_token.owner == ANTITOKEN_MULTISIG,
        PredictError::InvalidTokenAccount
    );
    require!(
        ctx.accounts.poll_pro_token.owner == ANTITOKEN_MULTISIG,
        PredictError::InvalidTokenAccount
    );

    // Transfer $ANTI tokens if amount > 0
    if anti > 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_anti_token.to_account_info(),
                    to: ctx.accounts.poll_anti_token.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            anti,
        )?;
    }

    // Transfer $PRO tokens if amount > 0
    if pro > 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_pro_token.to_account_info(),
                    to: ctx.accounts.poll_pro_token.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            pro,
        )?;
    }

    // Calculate metrics (u and s values)
    let (u, s) = collide(anti, pro)?;

    // Serialise and update poll data
    let poll_info = poll.to_account_info();
    let mut data_poll = poll_info.try_borrow_mut_data()?;

    // Create deposit record
    let deposit = UserDeposit {
        address: ctx.accounts.authority.key(),
        anti,
        pro,
        u,
        s,
        withdrawn: false,
    };

    // Update poll state
    poll.deposits.push(deposit);
    poll.anti = poll
        .anti
        .checked_add(anti)
        .ok_or(error!(PredictError::MathError))?;
    poll.pro = poll
        .pro
        .checked_add(pro)
        .ok_or(error!(PredictError::MathError))?;

    // Serialise updated poll state
    let serialised_poll = poll.try_to_vec()?;
    data_poll[8..8 + serialised_poll.len()].copy_from_slice(&serialised_poll);

    // Emit deposit event
    emit!(DepositEvent {
        poll_index,
        address: ctx.accounts.authority.key(),
        anti,
        pro,
        u,
        s,
        timestamp: now,
    });

    Ok(())
}

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
    index: u64,
    anti: u64,
    pro: u64,
) -> Result<()> {
    let prediction = &mut ctx.accounts.prediction;

    // Get current time, supporting local testing override
let now = Clock::get()?.unix_timestamp;

    // Verify prediction is active
    require!(prediction.is_active(now), PredictError::PredictionInactive);

    // Verify minimum deposit
    require!(
        anti >= MIN_DEPOSIT_AMOUNT || pro >= MIN_DEPOSIT_AMOUNT,
        PredictError::InsufficientDeposit
    );

    // Check prediction token account authorities
    require!(
        ctx.accounts.prediction_anti_token.owner == ANTITOKEN_MULTISIG,
        PredictError::InvalidTokenAccount
    );
    require!(
        ctx.accounts.prediction_pro_token.owner == ANTITOKEN_MULTISIG,
        PredictError::InvalidTokenAccount
    );

    // Transfer $ANTI tokens if amount > 0
    if anti > 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_anti_token.to_account_info(),
                    to: ctx.accounts.prediction_anti_token.to_account_info(),
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
                    to: ctx.accounts.prediction_pro_token.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            pro,
        )?;
    }

    // Calculate metrics (mean and stddev values)
    let (mean, stddev) = collide(anti, pro)?;

    // Serialise and update prediction data
    let prediction_info = prediction.to_account_info();
    let mut data_prediction = prediction_info.try_borrow_mut_data()?;

    // Create deposit record
    let deposit = Deposit {
        address: ctx.accounts.authority.key(),
        anti,
        pro,
        mean,
        stddev,
        withdrawn: false,
    };

    // Update prediction state
    prediction.deposits.push(deposit);
    prediction.anti = prediction
        .anti
        .checked_add(anti)
        .ok_or(error!(PredictError::MathError))?;
    prediction.pro = prediction
        .pro
        .checked_add(pro)
        .ok_or(error!(PredictError::MathError))?;

    // Serialise updated prediction state
    let serialised_prediction = prediction.try_to_vec()?;
    data_prediction[8..8 + serialised_prediction.len()].copy_from_slice(&serialised_prediction);

    // Emit deposit event
    emit!(DepositEvent {
        index,
        address: ctx.accounts.authority.key(),
        anti,
        pro,
        mean,
        stddev,
        timestamp: now,
    });

    Ok(())
}

//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

// instructions/user_withdraw.rs
use crate::utils::*;
use crate::UserWithdrawTokens;
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Transfer};
use borsh::BorshSerialize;

pub fn user_withdraw<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, UserWithdrawTokens<'info>>,
    index: u64,
) -> Result<()> {
    // Check token account authorities
    let anti_token_authority = ctx.accounts.prediction_anti_token.owner;
    let pro_token_authority = ctx.accounts.prediction_pro_token.owner;

    // Both token accounts must have the same authority
    require!(
        anti_token_authority == pro_token_authority,
        PredictError::InvalidTokenAccount
    );

    let current_authority = anti_token_authority;

    // If authority is still multisig, user withdrawals aren't enabled yet
    if current_authority == ANTITOKEN_MULTISIG {
        return err!(PredictError::UserWithdrawalsNotEnabled);
    }

    // Authority must be either multisig or state PDA
    let state_pda = ctx.accounts.state.key();
    require!(current_authority == state_pda, PredictError::Unauthorised);

    // Verify prediction has been equalised
    require!(
        &ctx.accounts.prediction.equalised,
        PredictError::NotEqualised
    );

    let equalisation = &ctx
        .accounts
        .prediction
        .equalisation
        .clone()
        .ok_or(error!(PredictError::NotEqualised))?;

    // Get current deposit for the user
    let user_key = ctx.accounts.authority.key();
    let deposit_index = ctx
        .accounts
        .prediction
        .deposits
        .iter()
        .position(|d| d.address == user_key)
        .ok_or(error!(PredictError::NoDeposit))?;

    let deposit = ctx.accounts.prediction.deposits[deposit_index].clone();
    require!(!deposit.withdrawn, PredictError::AlreadyWithdrawn);

    // Get withdrawal amounts
    let anti_return = equalisation.anti[deposit_index];
    let pro_return = equalisation.pro[deposit_index];

    // Calculate and transfer payment (e.g., 0.001 SOL)
    let payment_amount = 1_000_000;
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.authority.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        ),
        payment_amount,
    )?;

    // Transfer ANTI tokens if any
    if anti_return > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.prediction_anti_token.to_account_info(),
                    to: ctx.accounts.user_anti_token.to_account_info(),
                    authority: ctx.accounts.state.to_account_info(),
                },
                &[&[b"state", &[ctx.bumps.state]]],
            ),
            anti_return,
        )?;
    }

    // Transfer PRO tokens if any
    if pro_return > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.prediction_pro_token.to_account_info(),
                    to: ctx.accounts.user_pro_token.to_account_info(),
                    authority: ctx.accounts.state.to_account_info(),
                },
                &[&[b"state", &[ctx.bumps.state]]],
            ),
            pro_return,
        )?;
    }

    // Mark deposit as withdrawn
    let prediction = &mut ctx.accounts.prediction;
    prediction.deposits[deposit_index].withdrawn = true;

    // Serialise updated prediction state
    let prediction_info = prediction.to_account_info();
    let mut prediction_data = prediction_info.try_borrow_mut_data()?;
    let serialised_prediction = prediction.try_to_vec()?;
    prediction_data[8..8 + serialised_prediction.len()].copy_from_slice(&serialised_prediction);

    // Emit withdrawal event
    emit!(WithdrawEvent {
        index,
        address: user_key,
        anti: anti_return,
        pro: pro_return,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

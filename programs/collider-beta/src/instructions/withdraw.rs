//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

use crate::utils::*;
use crate::WithdrawTokens;
use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use anchor_spl::token::{self, Transfer};

pub fn withdraw<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, WithdrawTokens<'info>>,
    poll_index: u64,
) -> Result<()> {
    // Verify only ANTITOKEN_MULTISIG can execute this
    let authority_key = ctx.accounts.authority.key();
    require!(
        authority_key == ANTITOKEN_MULTISIG,
        PredictError::Unauthorised
    );

    // Get the poll account
    let poll = &mut ctx.accounts.poll;

    // Verify poll has been equalised
    require!(poll.equalised, PredictError::NotEqualised);

    let equalisation = poll
        .equalisation_results
        .clone() // Clone to avoid immutable borrow issue
        .ok_or(error!(PredictError::NotEqualised))?;

    // Create seeds for PDA signing
    let index_bytes: [u8; 8] = poll_index.to_le_bytes(); // Explicitly define array size

    let seeds: &[&[u8]] = &[
        b"poll",           // Already `&[u8]`
        &index_bytes,      // Explicitly reference as slice
        &[ctx.bumps.poll], // Wrap in slice to match type
    ];

    let signer_seeds = &[&seeds[..]];

    // Track total withdrawn amounts for verification
    let mut total_anti_withdrawn: u64 = 0;
    let mut total_pro_withdrawn: u64 = 0;

    // Use explicit lifetime for `remaining_accounts`
    let remaining_accounts: &'info [AccountInfo<'info>] = ctx.remaining_accounts;
    let mut deposits = poll.deposits.clone();
    let num_deposits = deposits.len();
    let enum_deposits = deposits.iter_mut().enumerate();

    require!(
        remaining_accounts.len() == num_deposits * 2,
        PredictError::InvalidTokenAccount
    );

    // Iterate through deposits
    for (deposit_index, deposit) in enum_deposits {
        // Skip if already withdrawn
        if deposit.withdrawn {
            continue;
        }

        // Get return amounts for this deposit
        let anti_return = equalisation.anti[deposit_index];
        let pro_return = equalisation.pro[deposit_index];

        // Fix: Explicitly specify lifetime for `Account<TokenAccount>`
        let user_anti_token: Account<'info, TokenAccount> =
            Account::try_from(&remaining_accounts[deposit_index])?;
        let user_pro_token: Account<'info, TokenAccount> =
            Account::try_from(&remaining_accounts[deposit_index + num_deposits])?;

        // Transfer $ANTI tokens
        if anti_return > 0 {
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.poll_anti_token.to_account_info(),
                        to: user_anti_token.to_account_info(),
                        authority: poll.to_account_info(),
                    },
                    signer_seeds,
                ),
                anti_return,
            )?;
            total_anti_withdrawn += anti_return;
        }

        // Transfer $PRO tokens
        if pro_return > 0 {
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.poll_pro_token.to_account_info(),
                        to: user_pro_token.to_account_info(),
                        authority: poll.to_account_info(),
                    },
                    signer_seeds,
                ),
                pro_return,
            )?;
            total_pro_withdrawn += pro_return;
        }

        // Mark as withdrawn
        deposit.withdrawn = true;
    }

    // Verify total withdrawn matches equalisation results
    require!(
        total_anti_withdrawn == equalisation.anti.iter().copied().sum::<u64>(),
        PredictError::InvalidEqualisation
    );
    require!(
        total_pro_withdrawn == equalisation.pro.iter().copied().sum::<u64>(),
        PredictError::InvalidEqualisation
    );

    // Emit bulk withdrawal event
    emit!(WithdrawEvent {
        poll_index,
        address: ctx.accounts.authority.key(),
        anti: total_anti_withdrawn,
        pro: total_pro_withdrawn,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

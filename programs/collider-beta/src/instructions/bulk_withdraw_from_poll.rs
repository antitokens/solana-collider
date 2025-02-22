//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

use crate::utils::*;
use crate::BulkWithdrawTokens;
use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use anchor_spl::token::{self, Transfer};
use borsh::BorshSerialize;

pub fn bulk_withdraw<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, BulkWithdrawTokens<'info>>,
    poll_index: u64,
) -> Result<()> {
    let authority_key = ctx.accounts.authority.key();
    require!(
        authority_key == ANTITOKEN_MULTISIG,
        PredictError::Unauthorised
    );
    require!(&ctx.accounts.poll.equalised, PredictError::NotEqualised);

    let equalisation = ctx
        .accounts
        .poll
        .equalisation_results
        .clone()
        .ok_or(error!(PredictError::NotEqualised))?;

    let poll_info = ctx.accounts.poll.to_account_info();
    let mut poll_data = poll_info.try_borrow_mut_data()?;

    let poll = &mut ctx.accounts.poll;
    let remaining_accounts: &[AccountInfo<'info>] = ctx.remaining_accounts;
    let mut deposits = poll.deposits.clone();
    let num_deposits = deposits.len();

    require!(
        remaining_accounts.len() == num_deposits * 2,
        PredictError::InvalidTokenAccount
    );

    let mut total_anti_withdrawn: u64 = 0;
    let mut total_pro_withdrawn: u64 = 0;

    for (deposit_index, deposit) in deposits.iter_mut().enumerate() {
        if deposit.withdrawn {
            continue;
        }

        let anti_return = equalisation.anti[deposit_index];
        let pro_return = equalisation.pro[deposit_index];

        let user_anti_token: Account<'info, TokenAccount> =
            Account::try_from(&remaining_accounts[deposit_index * 2])?;
        let user_pro_token: Account<'info, TokenAccount> =
            Account::try_from(&remaining_accounts[deposit_index * 2 + 1])?;

        if anti_return > 0 {
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.poll_anti_token.to_account_info(),
                        to: user_anti_token.to_account_info(),
                        authority: ctx.accounts.authority.to_account_info(),
                    },
                    &[],
                ),
                anti_return,
            )?;
            total_anti_withdrawn += anti_return;
        }

        if pro_return > 0 {
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.poll_pro_token.to_account_info(),
                        to: user_pro_token.to_account_info(),
                        authority: ctx.accounts.authority.to_account_info(),
                    },
                    &[],
                ),
                pro_return,
            )?;
            total_pro_withdrawn += pro_return;
        }

        deposit.withdrawn = true;
    }

    // Verify total withdrawals match equalisation sums
    require!(
        total_anti_withdrawn == equalisation.anti.iter().copied().sum::<u64>(),
        PredictError::InvalidEqualisation
    );
    require!(
        total_pro_withdrawn == equalisation.pro.iter().copied().sum::<u64>(),
        PredictError::InvalidEqualisation
    );

    // Serialise updated poll state and store it in account data
    poll.deposits = deposits;
    let serialised_poll = poll.try_to_vec()?;
    poll_data[8..8 + serialised_poll.len()].copy_from_slice(&serialised_poll);

    emit!(WithdrawEvent {
        poll_index,
        address: ctx.accounts.authority.key(),
        anti: total_anti_withdrawn,
        pro: total_pro_withdrawn,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

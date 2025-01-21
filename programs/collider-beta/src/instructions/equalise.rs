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
use crate::EqualiseTokens;

pub fn equaliser(
    ctx: Context<EqualiseTokens>,
    poll_index: u64,
    truth_values: Vec<u64>,
) -> Result<()> {
    let poll = &mut ctx.accounts.poll;

    // Verify poll has ended
    let clock = Clock::get()?;
    let end_time = parse_iso_timestamp(&poll.end_time)?;
    require!(clock.unix_timestamp >= end_time, PredictError::PollEnded);

    // Validate truth values
    require!(
        truth_values.len() == 2 && truth_values.iter().all(|v| *v <= BASIS_POINTS),
        PredictError::InvalidTruthValues
    );

    // Calculate distributions and returns
    let (anti_returns, pro_returns) = calculate_equalisation(
        &poll.deposits,
        poll.total_anti,
        poll.total_pro,
        &truth_values,
    )?;

    // Update poll state with equalisation results
    poll.equalised = true;
    poll.equalisation_results = Some(EqualisationResult {
        anti_returns,
        pro_returns,
        truth_values: truth_values.clone(),
        timestamp: clock.unix_timestamp,
    });

    // Emit equalisation event
    emit!(EqualisationEvent {
        poll_index,
        truth_values,
        total_anti: poll.total_anti,
        total_pro: poll.total_pro,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

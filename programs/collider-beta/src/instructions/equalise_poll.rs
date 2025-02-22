//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

use crate::state::*;
use crate::utils::*;
use crate::EqualiseTokens;
use anchor_lang::prelude::*;

pub fn equalise(
    ctx: Context<EqualiseTokens>,
    poll_index: u64,
    truth: Vec<u64>,
) -> Result<()> {
    let poll = &mut ctx.accounts.poll;

    // Verify poll has ended
    // Get current time, supporting local testing override
let now = Clock::get()?.unix_timestamp;
    
    let end_time = parse_iso_timestamp(&poll.end_time)?;
    require!(now >= end_time, PredictError::PollActive);

    // Validate truth values
    require!(
        truth.len() == 2 && truth.iter().all(|v| *v <= TRUTH_BASIS),
        PredictError::InvalidTruthValues
    );

    // Check if poll not already equalised
    require!(!poll.equalised, PredictError::AlreadyEqualised);

    // Calculate distributions and returns
    let (anti, pro) = equalise_with_truth(&poll.deposits, poll.anti, poll.pro, &truth)?;

    // Update poll state with equalisation results
    poll.equalised = true;
    poll.equalisation_results = Some(EqualisationResult {
        anti,
        pro,
        truth: truth.clone(),
        timestamp: now,
    });

    // Get account info and serialise
    let poll_info = poll.to_account_info();
    let mut data = poll_info.try_borrow_mut_data()?;
    let serialised_poll = poll.try_to_vec()?;
    data[8..8 + serialised_poll.len()].copy_from_slice(&serialised_poll);

    // Emit equalisation event
    emit!(EqualisationEvent {
        poll_index,
        truth,
        anti: poll.anti,
        pro: poll.pro,
        timestamp: now,
    });

    Ok(())
}

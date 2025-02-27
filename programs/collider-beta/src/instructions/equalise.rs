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
    index: u64,
    truth: Vec<u64>,
) -> Result<()> {
    let prediction = &mut ctx.accounts.prediction;

    // Verify prediction has ended
    // Get current time, supporting local testing override
let now = Clock::get()?.unix_timestamp;

    let end_time = parse_iso_timestamp(&prediction.end_time)?;
    require!(now >= end_time, PredictError::PredictionActive);

    // Validate truth values
    require!(
        truth.len() == 2 && truth.iter().all(|v| *v <= TRUTH_BASIS),
        PredictError::InvalidTruthValues
    );

    // Check if prediction not already equalised
    require!(!prediction.equalised, PredictError::AlreadyEqualised);

    // Calculate distributions and returns
    let (anti, pro) = equalise_with_truth(
        &prediction.deposits,
        prediction.anti,
        prediction.pro,
        &truth,
    )?;

    // Update prediction state with equalisation results
    prediction.equalised = true;
    prediction.equalisation = Some(Equalisation {
        anti,
        pro,
        truth: truth.clone(),
        timestamp: now,
    });

    // Get account info and serialise
    let prediction_info = prediction.to_account_info();
    let mut data = prediction_info.try_borrow_mut_data()?;
    let serialised_prediction = prediction.try_to_vec()?;
    data[8..8 + serialised_prediction.len()].copy_from_slice(&serialised_prediction);

    // Emit equalisation event
    emit!(EqualisationEvent {
        index,
        truth,
        anti: prediction.anti,
        pro: prediction.pro,
        timestamp: now,
    });

    Ok(())
}

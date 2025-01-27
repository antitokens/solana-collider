//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's utils
//! Version: 0.0.1
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

// utils.rs
use crate::state::*;
use anchor_lang::prelude::*;

pub const BASIS_POINTS: u64 = 10000; // For fixed-point arithmetic
pub const MAX_TITLE_LENGTH: usize = 100;
pub const MAX_DESC_LENGTH: usize = 500;
pub const MIN_DEPOSIT_AMOUNT: u64 = 1000; // 0.001 tokens minimum deposit

#[error_code]
pub enum PredictError {
    #[msg("Insufficient payment for creating poll")]
    InsufficientPayment,
    #[msg("Poll is not active")]
    PollInactive,
    #[msg("Poll has already ended")]
    PollEnded,
    #[msg("Title exceeds maximum length")]
    TitleTooLong,
    #[msg("Description exceeds maximum length")]
    DescriptionTooLong,
    #[msg("Invalid time format")]
    InvalidTimeFormat,
    #[msg("End time must be after start time")]
    InvalidTimeRange,
    #[msg("Start time must be in the future")]
    StartTimeInPast,
    #[msg("Insufficient deposit amount")]
    InsufficientDeposit,
    #[msg("Invalid token account ownership")]
    InvalidTokenAccount,
    #[msg("Unauthorised operation")]
    Unauthorised,
    #[msg("Already initialised")]
    AlreadyInitialised,
    #[msg("Invalid truth values provided")]
    InvalidTruthValues,
    #[msg("Arithmetic operation failed")]
    MathError,
    #[msg("Poll title already exists")]
    TitleExists,
    #[msg("Poll not found")]
    PollNotFound,
    #[msg("Poll not yet equalised")]
    NotEqualised,
    #[msg("No deposit found for user")]
    NoDeposit,
    #[msg("Tokens already withdrawn")]
    AlreadyWithdrawn,
    #[msg("Invalid equalisation calculation")]
    InvalidEqualisation,
    #[msg("Prediction already equalised")]
    AlreadyEqualised,
    #[msg("No deposits in prediction pool")]
    NoDeposits,
}

// Event emitted when a new poll is created
#[event]
pub struct PollCreatedEvent {
    pub poll_index: u64,
    pub address: Pubkey,
    pub title: String,
    pub start_time: String,
    pub end_time: String,
    pub timestamp: i64,
}

// Event emitted when tokens are deposited
#[event]
pub struct DepositEvent {
    pub poll_index: u64,
    pub address: Pubkey,
    pub anti: u64,
    pub pro: u64,
    pub u: u64,
    pub s: u64,
    pub timestamp: i64,
}

// Event emitted when equalisation occurs
#[event]
pub struct EqualisationEvent {
    pub poll_index: u64,
    pub truth: Vec<u64>,
    pub anti: u64,
    pub pro: u64,
    pub timestamp: i64,
}

// Event emitted when tokens are withdrawn
#[event]
pub struct WithdrawEvent {
    pub poll_index: u64,
    pub address: Pubkey,
    pub anti: u64,
    pub pro: u64,
    pub timestamp: i64,
}

// Event for updates to poll parameters
#[event]
pub struct PollUpdateEvent {
    pub poll_index: u64,
    pub field_updated: String,
    pub timestamp: i64,
}

// Event for administrative actions
#[event]
pub struct AdminEvent {
    pub action: String,
    pub poll_index: u64,
    pub timestamp: i64,
}

// Utility functions for calculations
pub fn calculate_metrics(anti: u64, pro: u64, flag: bool) -> Result<(u64, u64)> {
    if flag {
        return Ok((anti, pro));
    }

    let anti_f = anti * BASIS_POINTS;
    let pro_f = pro * BASIS_POINTS;
    let sum = anti_f.checked_add(pro_f).ok_or(PredictError::MathError)?;
    let diff = if anti_f > pro_f {
        anti_f.checked_sub(pro_f)
    } else {
        pro_f.checked_sub(anti_f)
    }
    .ok_or(PredictError::MathError)?;

    // Calculate u
    let u = if sum < BASIS_POINTS {
        0
    } else if diff > 0 && diff < BASIS_POINTS {
        diff
    } else {
        diff
    };

    // Calculate s
    let s = if sum < BASIS_POINTS {
        0
    } else if diff == sum {
        0
    } else if diff > 0 && diff < BASIS_POINTS {
        (sum * BASIS_POINTS) / BASIS_POINTS
    } else if diff == 0 {
        sum
    } else {
        (sum * BASIS_POINTS) / diff
    };

    Ok((u / BASIS_POINTS, s / BASIS_POINTS))
}

// Function to parse date
pub fn parse_iso_timestamp(time_str: &str) -> Result<i64> {
    // Basic ISO string validation (YYYY-MM-DDTHH:mm:ssZ)
    if time_str.len() != 20 || !time_str.ends_with('Z') {
        return Err(error!(PredictError::InvalidTimeFormat));
    }

    // Parse components
    let year: i64 = time_str[0..4]
        .parse()
        .map_err(|_| error!(PredictError::InvalidTimeFormat))?;
    let month: i64 = time_str[5..7]
        .parse()
        .map_err(|_| error!(PredictError::InvalidTimeFormat))?;
    let day: i64 = time_str[8..10]
        .parse()
        .map_err(|_| error!(PredictError::InvalidTimeFormat))?;
    let hour: i64 = time_str[11..13]
        .parse()
        .map_err(|_| error!(PredictError::InvalidTimeFormat))?;
    let minute: i64 = time_str[14..16]
        .parse()
        .map_err(|_| error!(PredictError::InvalidTimeFormat))?;
    let second: i64 = time_str[17..19]
        .parse()
        .map_err(|_| error!(PredictError::InvalidTimeFormat))?;

    // Basic validation
    if month < 1 || month > 12 || day < 1 || day > 31 || hour >= 24 || minute >= 60 || second >= 60
    {
        return Err(error!(PredictError::InvalidTimeFormat));
    }

    // Convert to Unix timestamp (simplified)
    let timestamp = second
        + minute * 60
        + hour * 3600
        + day * 86400
        + month * 2592000
        + (year - 1970) * 31536000;

    Ok(timestamp)
}

// Function to check if a title exists in state
pub fn state_has_title(_state: &Account<StateAccount>, _title: &str) -> bool {
    // TODO: Implement title uniqueness check based on your state structure
    // For now, returning false as placeholder
    false
}

// Helper function to validate poll parameters
pub fn validate_poll_params(
    title: &str,
    description: &str,
    start_time: &str,
    end_time: &str,
) -> Result<()> {
    require!(title.len() <= MAX_TITLE_LENGTH, PredictError::TitleTooLong);
    require!(
        description.len() <= MAX_DESC_LENGTH,
        PredictError::DescriptionTooLong
    );

    let start = parse_iso_timestamp(start_time)?;
    let end = parse_iso_timestamp(end_time)?;
    let now = Clock::get()?.unix_timestamp;

    require!(end > start, PredictError::InvalidTimeRange);
    require!(start > now, PredictError::StartTimeInPast);

    Ok(())
}

// Calculates equalisation in the pool given some truth
pub fn equalise_with_truth(
    deposits: &[UserDeposit],
    total_anti: u64,
    total_pro: u64,
    truth: &[u64],
) -> Result<(Vec<u64>, Vec<u64>)> {
    const NUM_BINS: usize = 100;
    let bin_size = BASIS_POINTS / NUM_BINS as u64;

    // Initialise bins
    let mut bins = vec![0u64; NUM_BINS];
    let mut items_in_bins = vec![Vec::new(); NUM_BINS];
    let mut value_sums = vec![(0u64, 0u64); NUM_BINS];

    // Calculate normalised overlap with truth
    let mut overlap_values = Vec::with_capacity(deposits.len());
    for deposit in deposits {
        let parity = if (truth[0] > truth[1]) == (deposit.anti > deposit.pro) {
            1i64
        } else {
            -1i64
        };

        let baryon = deposit.u;
        let photon = deposit.s;

        // Calculate overlap value
        let overlap = calculate_overlap(baryon, photon, parity)?;
        overlap_values.push(overlap);
    }

    // Populate bins
    for (i, &overlap) in overlap_values.iter().enumerate() {
        if overlap <= BASIS_POINTS {
            let bin_index = (overlap / bin_size) as usize;
            let bin_index = bin_index.min(NUM_BINS - 1);

            bins[bin_index] += 1;
            items_in_bins[bin_index].push(i);
            value_sums[bin_index].0 += deposits[i].anti;
            value_sums[bin_index].1 += deposits[i].pro;
        }
    }

    // Calculate distribution and returns
    let mut anti_returns = vec![0u64; deposits.len()];
    let mut pro_returns = vec![0u64; deposits.len()];

    for (bin_idx, indices) in items_in_bins.iter().enumerate() {
        if indices.is_empty() {
            continue;
        }

        let bin_anti_total = value_sums[bin_idx].0;
        let bin_pro_total = value_sums[bin_idx].1;

        for &deposit_idx in indices {
            let deposit = &deposits[deposit_idx];

            // Calculate proportional returns
            if bin_anti_total > 0 {
                anti_returns[deposit_idx] = (deposit.anti * total_anti) / bin_anti_total;
            }
            if bin_pro_total > 0 {
                pro_returns[deposit_idx] = (deposit.pro * total_pro) / bin_pro_total;
            }
        }
    }

    Ok((anti_returns, pro_returns))
}

// Helper function for equalise_with_truth
fn calculate_overlap(baryon: u64, photon: u64, parity: i64) -> Result<u64> {
    const TWO_E9: u64 = 2_000_000_000;

    if baryon >= TWO_E9 {
        return Ok(0);
    }

    let x = TWO_E9 - baryon;
    let log_x = (BASIS_POINTS * x.ilog2() as u64) / 10; // Simplified log calculation

    let photon_term = if photon <= BASIS_POINTS {
        BASIS_POINTS
    } else {
        BASIS_POINTS + (BASIS_POINTS * photon.ilog2() as u64) / 10
    };

    let result = if parity > 0 {
        (BASIS_POINTS * BASIS_POINTS) / (BASIS_POINTS + log_x / photon_term)
    } else {
        (log_x * photon_term) / BASIS_POINTS
    };

    Ok(result.min(BASIS_POINTS))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_metrics() {
        let (u, s) = calculate_metrics(100, 50, false).unwrap();
        assert_eq!(u, 50); // |100 - 50|
        assert_eq!(s, 3); // (100 + 50) / |100 - 50|

        let (u, s) = calculate_metrics(50, 50, false).unwrap();
        assert_eq!(u, 0);
        assert_eq!(s, 100);
    }

    #[test]
    fn test_parse_iso_timestamp() {
        assert!(parse_iso_timestamp("2025-01-20T00:00:00Z").is_ok());
        assert!(parse_iso_timestamp("2025-13-20T00:00:00Z").is_err()); // Invalid month
        assert!(parse_iso_timestamp("invalid").is_err());
    }
}

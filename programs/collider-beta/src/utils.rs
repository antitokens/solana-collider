//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's utils
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

// utils.rs
use crate::state::*;
use anchor_lang::prelude::*;
use chrono::NaiveDateTime;

pub const TRUTH_BASIS: u64 = 100_000; // Truth limit = [0, 1]
pub const BASIS_POINTS: u64 = 10_000; // For fixed-point arithmetic up to 0.01
pub const MAX_TITLE_LENGTH: usize = 256;
pub const MAX_DESCRIPTION_LENGTH: usize = 1_024;
pub const MIN_DEPOSIT_AMOUNT: u64 = 10_000; // 1 token minimum deposit

#[error_code]
pub enum PredictError {
    #[msg("Insufficient payment for creating poll")]
    InsufficientPayment,
    #[msg("Poll is not active")]
    PollInactive,
    #[msg("Poll is still active")]
    PollActive,
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
pub fn collide(anti: u64, pro: u64) -> Result<(u64, u64)> {
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
    // Validate basic ISO string format (YYYY-MM-DDTHH:mm:ssZ)
    if time_str.len() != 20 || !time_str.ends_with('Z') {
        return Err(error!(PredictError::InvalidTimeFormat));
    }

    // Parse components using chrono
    let naive_datetime = NaiveDateTime::parse_from_str(&time_str[..19], "%Y-%m-%dT%H:%M:%S")
        .map_err(|_| error!(PredictError::InvalidTimeFormat))?;

    // Convert to UTC Unix timestamp
    let unix_timestamp = naive_datetime.and_utc().timestamp();

    Ok(unix_timestamp)
}

// Function to check if a title exists in state
pub fn state_has_title(_state: &Account<StateAccount>, _title: &str) -> bool {
    // Allow title repetition
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
        description.len() <= MAX_DESCRIPTION_LENGTH,
        PredictError::DescriptionTooLong
    );

    let start = parse_iso_timestamp(start_time)?;
    let end = parse_iso_timestamp(end_time)?;
    let now = Clock::get()?.unix_timestamp;

    require!(end > start, PredictError::InvalidTimeRange);
    require!(start > now, PredictError::StartTimeInPast);

    Ok(())
}

pub fn equalise_with_truth(
    deposits: &[UserDeposit],
    anti_pool: u64,
    pro_pool: u64,
    truth: &[u64],
) -> Result<(Vec<u64>, Vec<u64>)> {
    const NUM_BINS: usize = 100;

    // Calculate overlaps
    let mut overlaps = Vec::with_capacity(deposits.len());
    for deposit in deposits {
        let baryon = deposit.u as f64;
        let photon = (deposit.s as f64) / (BASIS_POINTS as f64);
        let parity = if (truth[0] > truth[1]) == (deposit.anti > deposit.pro) {
            1.0
        } else {
            -1.0
        };

        let overlap_val = overlap(baryon, photon, parity)?;
        overlaps.push(overlap_val);
    }
    println!("{:?}", overlaps);
    // Initialise forward distribution structures
    let mut bins = vec![0u64; NUM_BINS];
    let mut items_in_bins: Vec<Vec<usize>> = vec![Vec::new(); NUM_BINS];

    // Populate bins
    for (i, &overlap_val) in overlaps.iter().enumerate() {
        if overlap_val >= 0.0 && overlap_val <= 1.0 {
            let bin_index = (overlap_val * (NUM_BINS as f64)).floor() as usize;
            let bin_index = bin_index.min(NUM_BINS - 1);

            bins[bin_index] += 1;
            items_in_bins[bin_index].push(i);
        }
    }

    // Find non-zero bins
    let non_zero_indices: Vec<usize> = bins
        .iter()
        .enumerate()
        .filter(|(_, &count)| count > 0)
        .map(|(i, _)| i)
        .collect();

    // Calculate bin values (scatterer)
    let mut bin_values = vec![0f64; NUM_BINS];
    if !non_zero_indices.is_empty() {
        let total_bins = (non_zero_indices.len() + 1) as f64;

        for (i, &bin_index) in non_zero_indices.iter().enumerate() {
            let reversed_index = non_zero_indices.len() - 1 - i;
            let value = ((reversed_index + 1) as f64) / total_bins;
            let normalised = value
                / non_zero_indices
                    .iter()
                    .enumerate()
                    .map(|(j, _)| ((non_zero_indices.len() - j) as f64) / total_bins)
                    .sum::<f64>();
            bin_values[bin_index] = normalised;
        }
    }

    // Initialise return arrays
    let mut anti_returns = vec![0u64; deposits.len()];
    let mut pro_returns = vec![0u64; deposits.len()];

    // Calculate returns (localiser)
    for (bin_idx, indices) in items_in_bins.iter().enumerate() {
        if indices.is_empty() || bin_values[bin_idx] == 0.0 {
            continue;
        }

        let bin_anti = (bin_values[bin_idx] * anti_pool as f64).round() as u64;
        let bin_pro = (bin_values[bin_idx] * pro_pool as f64).round() as u64;

        let total_anti: u64 = indices.iter().map(|&i| deposits[i].anti).sum();
        let total_pro: u64 = indices.iter().map(|&i| deposits[i].pro).sum();

        for &i in indices {
            if total_anti > 0 {
                anti_returns[i] =
                    ((bin_anti as u128 * deposits[i].anti as u128) / total_anti as u128) as u64;
            }
            if total_pro > 0 {
                pro_returns[i] =
                    ((bin_pro as u128 * deposits[i].pro as u128) / total_pro as u128) as u64;
            }
        }
    }

    Ok((anti_returns, pro_returns))
}

fn overlap(baryon: f64, photon: f64, parity: f64) -> Result<f64> {
    const TWO_E9: f64 = 2_000_000_000.0;

    // Early return if baryon too large
    if baryon >= TWO_E9 {
        return Ok(0.0);
    }

    // Calculate raw overlap value
    let log_term = -((TWO_E9 - baryon).ln().powi(2));
    let photon_term = 2.0
        * if photon <= 1.0 {
            1.0
        } else {
            (1.0 + photon.ln()).powi(2)
        };

    let raw_overlap = parity * (log_term / photon_term).exp();

    // Apply inverse log normalisation
    let normalised = if raw_overlap == 0.0 {
        0.0
    } else if raw_overlap == 1.0 {
        1.0
    } else if raw_overlap > 0.0 {
        1.0 / raw_overlap.ln().abs()
    } else {
        1.0 - 1.0 / raw_overlap.abs().ln().abs()
    };

    Ok(normalised.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collide() {
        let (u, s) = collide(100, 50).unwrap();
        assert_eq!(u, 50); // |100 - 50|
        assert_eq!(s, 3); // (100 + 50) / |100 - 50|

        let (u, s) = collide(50, 50).unwrap();
        assert_eq!(u, 0);
        assert_eq!(s, 100);
    }

    #[test]
    fn test_equalise_with_truth() {
        let deposits = vec![
            UserDeposit {
                address: Pubkey::new_unique(),
                anti: 7_000,
                pro: 3_000,
                u: 4_000,
                s: 25_000,
                withdrawn: false,
            },
            UserDeposit {
                address: Pubkey::new_unique(),
                anti: 3_000,
                pro: 7_000,
                u: 4_000,
                s: 25_000,
                withdrawn: false,
            },
        ];

        let total_anti = 10_000;
        let total_pro = 10_000;
        let truth = vec![0, 100_000];

        let result = equalise_with_truth(&deposits, total_anti, total_pro, &truth);

        assert!(result.is_ok(), "Equalisation should succeed");

        let (equalised_anti, equalised_pro) = result.unwrap();

        // Ensure distribution follows the truth ratio
        assert_eq!(equalised_anti.len(), deposits.len());
        assert_eq!(equalised_pro.len(), deposits.len());

        // Values from independent TypeScript simulations
        let expected_anti_split_1 = 3333;
        let expected_anti_split_2 = 6667;
        let expected_pro_split_1 = 3333;
        let expected_pro_split_2 = 6667;

        // Check for matches
        assert_eq!(equalised_anti[0], expected_anti_split_1);
        assert_eq!(equalised_anti[1], expected_anti_split_2);
        assert_eq!(equalised_pro[0], expected_pro_split_1);
        assert_eq!(equalised_pro[1], expected_pro_split_2);
    }

    #[test]
    fn test_parse_iso_timestamp() {
        assert!(parse_iso_timestamp("2025-01-20T00:00:00Z").is_ok());
        assert!(parse_iso_timestamp("2025-13-20T00:00:00Z").is_err()); // Invalid month
        assert!(parse_iso_timestamp("invalid").is_err());
    }
}

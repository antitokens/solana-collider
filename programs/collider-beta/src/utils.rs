//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider'stddev utils
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
use solana_security_txt;

pub const CREATION_FEE: u64 = 100_000_000; // Fee to create prediction (0.1 SOL)
pub const MAX_TITLE_LENGTH: u64 = 256; // Maximum title length
pub const MAX_DESCRIPTION_LENGTH: u64 = 1_024; // Maximum description length
pub const TRUTH_BASIS: u64 = 100_000; // Truth limit = [0, 1]
pub const FLOAT_BASIS: u64 = 10_000; // For fixed-point arithmetic up to 0.01
pub const MIN_DEPOSIT_AMOUNT: u64 = 10_000; // 1 token minimum deposit
pub const ANTITOKEN_MULTISIG: Pubkey =
    solana_program::pubkey!("7rFEa4g8UZs7eBBoq66FmLeobtb81dfCPx2Hmt61kJ5t");
pub const ANTI_MINT_ADDRESS: Pubkey =
    solana_program::pubkey!("674rRAKuyAizM6tWKLpo8zDqAtvxYS7ce6DoGBfocmrT");
pub const PRO_MINT_ADDRESS: Pubkey =
    solana_program::pubkey!("6bDmnBGtGo9pb2vhVkrzQD9uHYcYpBCCSgU61534MyTm");
pub const PROGRAM_ID: Pubkey =
    solana_program::pubkey!("C6BpSPd2mvtCP9tXQDDFAPP1NXLGDhQVmYMwsS9tkZUK");

#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    // Required fields
    name: "Antitoken Collider",
    project_url: "https://antitoken.pro",
    contacts: "email:dev@antitoken.pro,link:https://antitoken.pro/security",
    policy: "https://github.com/antitokens/solana-collider/SECURITY.md",

    // Optional Fields
    preferred_languages: "English",
    source_code: "https://github.com/antitokens/solana-collider",
    source_revision: "748c281a21fd5cce3ea75d9908cc516694450833",
    source_release: "v1.0.0-alpha",
    auditors: "None",
    acknowledgements: "Claude Haiku/3.5 Sonnet, ChatGPT o1/o3-mini"
}

#[error_code]
pub enum PredictError {
    #[msg("Insufficient payment for creating prediction")]
    InsufficientPayment,
    #[msg("Prediction is not active")]
    PredictionInactive,
    #[msg("Prediction is still active")]
    PredictionActive,
    #[msg("Prediction has already ended")]
    PredictionEnded,
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
    #[msg("Prediction title already exists")]
    TitleExists,
    #[msg("Prediction not found")]
    PredictionNotFound,
    #[msg("Prediction not yet equalised")]
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
    #[msg("User withdrawals not enabled yet")]
    UserWithdrawalsNotEnabled,
}

// Event emitted when a new prediction is created
#[event]
pub struct CreationEvent {
    pub index: u64,
    pub address: Pubkey,
    pub title: String,
    pub start_time: String,
    pub end_time: String,
    pub timestamp: i64,
}

// Event emitted when tokens are deposited
#[event]
pub struct DepositEvent {
    pub index: u64,
    pub address: Pubkey,
    pub anti: u64,
    pub pro: u64,
    pub mean: u64,
    pub stddev: u64,
    pub timestamp: i64,
}

// Event emitted when equalisation occurs
#[event]
pub struct EqualisationEvent {
    pub index: u64,
    pub truth: Vec<u64>,
    pub anti: u64,
    pub pro: u64,
    pub timestamp: i64,
}

// Event emitted when tokens are withdrawn
#[event]
pub struct WithdrawEvent {
    pub index: u64,
    pub address: Pubkey,
    pub anti: u64,
    pub pro: u64,
    pub timestamp: i64,
}

// Event for updates to prediction parameters
#[event]
pub struct PredictionUpdateEvent {
    pub index: u64,
    pub field_updated: String,
    pub timestamp: i64,
}

// AdminEvent for logging actions with arguments
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}
#[event]
pub struct AdminEvent {
    pub action: String,
    pub args: Vec<KeyValue>, // Dynamic key-value pairs for arguments
    pub timestamp: i64,
}

// Utility functions for calculations
pub fn collide(anti: u64, pro: u64) -> Result<(u64, u64)> {
    let anti_f = anti * FLOAT_BASIS;
    let pro_f = pro * FLOAT_BASIS;
    let sum = anti_f.checked_add(pro_f).ok_or(PredictError::MathError)?;
    let diff = if anti_f > pro_f {
        anti_f.checked_sub(pro_f)
    } else {
        pro_f.checked_sub(anti_f)
    }
    .ok_or(PredictError::MathError)?;

    // Calculate mean
    let mean = if sum < FLOAT_BASIS {
        0
    } else if diff > 0 && diff < FLOAT_BASIS {
        diff
    } else {
        diff
    };

    // Calculate stddev
    let stddev = if sum < FLOAT_BASIS {
        0
    } else if diff == sum {
        0
    } else if diff > 0 && diff < FLOAT_BASIS {
        (sum * FLOAT_BASIS) / FLOAT_BASIS
    } else if diff == 0 {
        sum
    } else {
        (sum * FLOAT_BASIS) / diff
    };

    Ok((mean / FLOAT_BASIS, stddev / FLOAT_BASIS))
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

// Helper function to validate prediction parameters
pub fn validate_prediction_params(
    title: &str,
    description: &str,
    start_time: &str,
    end_time: &str,
) -> Result<()> {
    require!(
        title.len() <= MAX_TITLE_LENGTH as usize,
        PredictError::TitleTooLong
    );
    require!(
        description.len() <= MAX_DESCRIPTION_LENGTH as usize,
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
    deposits: &[Deposit],
    anti_pool: u64,
    pro_pool: u64,
    truth: &[u64],
) -> Result<(Vec<u64>, Vec<u64>)> {
    const NUM_BINS: usize = 100;

    // Calculate overlaps
    let mut overlaps = Vec::with_capacity(deposits.len());
    for deposit in deposits {
        let baryon = deposit.mean as f64;
        let photon = (deposit.stddev as f64) / (FLOAT_BASIS as f64);
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

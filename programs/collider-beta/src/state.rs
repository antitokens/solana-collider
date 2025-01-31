//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's state enumeration
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 20 Jan 20251.0.0-beta
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

// state.rs
use crate::utils::parse_iso_timestamp;
use anchor_lang::prelude::*;

#[account]
pub struct StateAccount {
    pub poll_index: u64,
    pub authority: Pubkey,
}

impl StateAccount {
    pub const LEN: usize = 8  // Discriminator
        + 8   // poll_index
        + 32; // authority (Pubkey)
}

#[account]
pub struct PollAccount {
    pub index: u64,
    pub title: String,
    pub description: String,
    pub start_time: String,
    pub end_time: String,
    pub etc: Option<Vec<u8>>,
    pub anti: u64,
    pub pro: u64,
    pub deposits: Vec<UserDeposit>,
    pub equalised: bool,
    pub equalisation_results: Option<EqualisationResult>,
}

impl PollAccount {
    pub const LEN: usize = 8 + // discriminator
        8 + // index
        256 + // title max length
        1024 + // description max length
        64 + // start_time
        64 + // end_time
        1024 + // etc max length
        8 + // $ANTI in pool
        8 + // $PRO in pool
        1024 + // deposits vector space
        1 + // equalised
        1024; // equalisation_results

    pub fn is_active(&self, current_time: i64) -> bool {
        match (
            parse_iso_timestamp(&self.start_time),
            parse_iso_timestamp(&self.end_time),
        ) {
            (Ok(start), Ok(end)) => current_time >= start && current_time <= end,
            _ => false, // If timestamps are invalid, poll is not active
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct UserDeposit {
    pub address: Pubkey,
    pub anti: u64,
    pub pro: u64,
    pub u: u64,
    pub s: u64,
    pub withdrawn: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EqualisationResult {
    pub anti: Vec<u64>,
    pub pro: Vec<u64>,
    pub truth: Vec<u64>,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CreatePollBumps {
    pub poll: u8,
    pub poll_anti_token: u8, // FCK
    pub poll_pro_token: u8,  // FCK
}

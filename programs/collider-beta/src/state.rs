//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's state enumeration
//! Version: 0.0.1
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

// state.rs
use anchor_lang::prelude::*;

#[account]
pub struct StateAccount {
    pub poll_count: u64,
    pub authority: Pubkey,
}

impl StateAccount {
    pub const LEN: usize = 8 + 32; // u64 + Pubkey
}

#[account]
pub struct PollAccount {
    pub index: u64,
    pub title: String,
    pub description: String,
    pub start_time: String,
    pub end_time: String,
    pub etc: Option<Vec<u8>>,
    pub total_anti: u64,
    pub total_pro: u64,
    pub deposits: Vec<UserDeposit>,
    pub equalized: bool,
    pub equalization_results: Option<EqualizationResult>,
}

impl PollAccount {
    pub const LEN: usize = 8 + // discriminator
        8 + // index
        256 + // title max length
        1000 + // description max length
        64 + // start_time
        64 + // end_time
        1024 + // etc max length
        8 + // total_anti
        8 + // total_pro
        1024 + // deposits vector space
        1 + // equalized
        1024; // equalization_results

    pub fn is_active(&self, current_time: i64) -> bool {
        // Implementation from utils
        true
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct UserDeposit {
    pub user: Pubkey,
    pub anti_amount: u64,
    pub pro_amount: u64,
    pub u_value: u64,
    pub s_value: u64,
    pub withdrawn: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EqualizationResult {
    pub anti_returns: Vec<u64>,
    pub pro_returns: Vec<u64>,
    pub truth_values: Vec<u64>,
    pub timestamp: i64,
}

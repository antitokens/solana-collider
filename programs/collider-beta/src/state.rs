//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's state enumeration
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 02 Feb 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

// state.rs
use crate::utils::parse_iso_timestamp;
use anchor_lang::prelude::*;

#[account]
pub struct AdminAccount {
    pub initialised: bool,           // Initialisation flag
    pub poll_creation_fee: u64,      // Fee to create poll
    pub max_title_length: u64,       // Maximum title length
    pub max_description_length: u64, // Maximum description length
    pub truth_basis: u64,            // Truth limit
    pub float_basis: u64,            // Fixed-point arithmetic basis
    pub min_deposit_amount: u64,     // Minimum deposit
    pub antitoken_multisig: Pubkey,  // Multisig authority
    pub anti_mint_address: Pubkey,   // $ANTI token mint
    pub pro_mint_address: Pubkey,    // $PRO token mint
}

impl AdminAccount {
    pub const LEN: usize = 1 + (8 * 6) + (32 * 3); // Account size
}

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
    pub state: u8,
    pub poll: u8,
    pub poll_anti_token: u8,
    pub poll_pro_token: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BulkWithdrawTokensBumps {
    pub poll: u8,
    pub poll_anti_token: u8,
    pub poll_pro_token: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct UserWithdrawTokensBumps {
    pub state: u8,
    pub poll: u8,
    pub poll_anti_token: u8,
    pub poll_pro_token: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SetPollTokenAuthorityBumps {
    pub state: u8,
    pub poll_anti_token: u8,
    pub poll_pro_token: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AdminBumps {
    pub admin: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct UpdateBumps {
    pub admin: u8,
}

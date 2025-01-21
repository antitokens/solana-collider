//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 0.0.1
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

// instructions/create_poll.rs
use crate::utils::*;
use crate::CreatePoll;
use anchor_lang::prelude::*;

pub fn creator(
    ctx: Context<CreatePoll>,
    title: String,
    description: String,
    start_time: String,
    end_time: String,
    etc: Option<Vec<u8>>,
) -> Result<()> {
    // Verify payment
    require!(
        ctx.accounts.payment.lamports() >= 100000000, // 0.1 SOL in lamports
        PredictError::InsufficientPayment
    );

    // Validate string lengths
    require!(title.len() <= MAX_TITLE_LENGTH, PredictError::TitleTooLong);
    require!(
        description.len() <= MAX_DESC_LENGTH,
        PredictError::DescriptionTooLong
    );

    // Validate title uniqueness
    require!(
        !state_has_title(&ctx.accounts.state, &title),
        PredictError::TitleExists
    );

    // Validate timestamps
    let start = parse_iso_timestamp(&start_time)?;
    let end = parse_iso_timestamp(&end_time)?;
    let now = Clock::get()?.unix_timestamp;

    require!(end > start, PredictError::InvalidTimeRange);
    require!(start > now, PredictError::StartTimeInPast);

    let poll = &mut ctx.accounts.poll;
    let state = &mut ctx.accounts.state;

    // Initialize poll data
    poll.index = state.poll_count;
    poll.title = title.clone();
    poll.description = description;
    poll.start_time = start_time.clone();
    poll.end_time = end_time.clone();
    poll.etc = etc;
    poll.total_anti = 0;
    poll.total_pro = 0;
    poll.deposits = vec![];
    poll.equalised = false;
    poll.equalisation_results = None;

    state.poll_count += 1;

    // Emit event
    emit!(PollCreatedEvent {
        poll_index: poll.index,
        creator: ctx.accounts.authority.key(),
        title,
        start_time,
        end_time,
        timestamp: now,
    });

    Ok(())
}

// Add instruction data structs
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreatePollArgs {
    pub title: String,
    pub description: String,
    pub start_time: String,
    pub end_time: String,
    pub etc: Option<Vec<u8>>,
}

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

#[derive(Accounts)]
pub struct CreatePoll<'info> {
    #[account(mut)]
    pub state: Account<'info, StateAccount>,
    #[account(
        init,
        payer = authority,
        space = 8 + PollAccount::LEN,
        seeds = [b"poll", state.poll_count.to_le_bytes().as_ref()],
        bump
    )]
    pub poll: Account<'info, PollAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: Payment account
    #[account(mut)]
    pub payment: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreatePoll>,
    title: String,
    description: String,
    start_time: String,
    end_time: String,
    etc: Option<Vec<u8>>,
) -> Result<()> {
    // Validate string lengths
    require!(title.len() <= MAX_TITLE_LENGTH, VotingError::TitleTooLong);
    require!(
        description.len() <= MAX_DESC_LENGTH,
        VotingError::DescriptionTooLong
    );

    // Validate title uniqueness
    require!(
        !state_has_title(&ctx.accounts.state, &title),
        VotingError::TitleExists
    );

    // Validate timestamps
    let start = parse_iso_timestamp(&start_time)?;
    let end = parse_iso_timestamp(&end_time)?;
    let now = Clock::get()?.unix_timestamp;

    require!(end > start, VotingError::InvalidTimeRange);
    require!(start > now, VotingError::StartTimeInPast);
    // Verify SOL payment
    require!(
        ctx.accounts.payment.lamports() >= 100000000, // 0.1 SOL in lamports
        VotingError::InsufficientPayment
    );

    let poll = &mut ctx.accounts.poll;
    let state = &mut ctx.accounts.state;

    // Initialise poll data
    poll.index = state.poll_count;
    poll.title = title;
    poll.description = description;
    poll.start_time = start_time;
    poll.end_time = end_time;
    poll.etc = etc;
    poll.total_anti = 0;
    poll.total_pro = 0;

    // Create token accounts for the poll
    // Token account creation logic here...

    state.poll_count += 1;

    // Emit event
    emit!(PollCreatedEvent {
        poll_index: poll.index,
        creator: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 02 Feb 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

// instructions/create_poll.rs
use crate::utils::*;
use crate::CreatePoll;
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token;
use anchor_spl::token::spl_token::instruction::AuthorityType;
use anchor_spl::token::SetAuthority;

pub fn create(
    ctx: Context<CreatePoll>,
    title: String,
    description: String,
    start_time: String,
    end_time: String,
    etc: Option<Vec<u8>>,
) -> Result<()> {
    // Ensure payment is sufficient
    require!(
        ctx.accounts.authority.lamports() >= POLL_CREATION_FEE,
        PredictError::InsufficientPayment
    );

    // Validate title and description lengths
    require!(
        title.len() <= MAX_TITLE_LENGTH as usize,
        PredictError::TitleTooLong
    );
    require!(
        description.len() <= MAX_DESCRIPTION_LENGTH as usize,
        PredictError::DescriptionTooLong
    );

    // Ensure the title is unique
    require!(
        !state_has_title(&ctx.accounts.state, &title),
        PredictError::TitleExists
    );

    // Parse and validate time ranges
    let start = parse_iso_timestamp(&start_time)?;
    let end = parse_iso_timestamp(&end_time)?;
let now = Clock::get()?.unix_timestamp;

    require!(end > start, PredictError::InvalidTimeRange);
    require!(start > now, PredictError::StartTimeInPast);

    // Transfer payment to state account
    let payment_amount = POLL_CREATION_FEE;
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.authority.to_account_info(),
                to: ctx.accounts.state.to_account_info(),
            },
        ),
        payment_amount,
    )?;

    // Set the token account authority to ANTITOKEN_MULTISIG using token instruction
    let cpi_accounts = SetAuthority {
        account_or_mint: ctx.accounts.poll_anti_token.to_account_info(),
        current_authority: ctx.accounts.authority.to_account_info(),
    };

    token::set_authority(
        CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
        AuthorityType::AccountOwner,
        Some(ANTITOKEN_MULTISIG),
    )?;

    let cpi_accounts = SetAuthority {
        account_or_mint: ctx.accounts.poll_pro_token.to_account_info(),
        current_authority: ctx.accounts.authority.to_account_info(),
    };

    token::set_authority(
        CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
        AuthorityType::AccountOwner,
        Some(ANTITOKEN_MULTISIG),
    )?;

    // Get account infos for manual serialisation
    let state_info = &ctx.accounts.state.to_account_info();
    let poll_info = &ctx.accounts.poll.to_account_info();
    let mut data_state = state_info.try_borrow_mut_data()?;
    let mut data_poll = poll_info.try_borrow_mut_data()?;

    // Set poll data
    ctx.accounts.poll.index = ctx.accounts.state.poll_index;
    ctx.accounts.poll.title = title.clone();
    ctx.accounts.poll.description = description;
    ctx.accounts.poll.start_time = start_time.clone();
    ctx.accounts.poll.end_time = end_time.clone();
    ctx.accounts.poll.etc = etc;
    ctx.accounts.poll.anti = 0;
    ctx.accounts.poll.pro = 0;
    ctx.accounts.poll.deposits = vec![];
    ctx.accounts.poll.equalised = false;
    ctx.accounts.poll.equalisation_results = None;

    // Manual serialisation
    let serialised_poll = ctx.accounts.poll.try_to_vec()?;
    data_poll[8..8 + serialised_poll.len()].copy_from_slice(&serialised_poll);

    // Increment poll index
    ctx.accounts.state.poll_index += 1;

    // Manual serialisation for state
    let serialised_state = ctx.accounts.state.try_to_vec()?;
    data_state[8..8 + serialised_state.len()].copy_from_slice(&serialised_state);

    // Emit event
    emit!(PollCreatedEvent {
        poll_index: ctx.accounts.poll.index,
        address: ctx.accounts.authority.key(),
        title,
        start_time,
        end_time,
        timestamp: now,
    });

    Ok(())
}

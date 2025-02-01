//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

// instructions/create_poll.rs
use crate::utils::*;
use crate::CreatePoll;
use anchor_lang::prelude::*;
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
        ctx.accounts.payment.lamports() >= 100_000_000,
        PredictError::InsufficientPayment
    );

    // Validate title and description lengths
    require!(title.len() <= MAX_TITLE_LENGTH, PredictError::TitleTooLong);
    require!(
        description.len() <= MAX_DESCRIPTION_LENGTH,
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

    // Create poll token accounts for $ANTI and $PRO tokens
    let cpi_accounts = token::InitializeAccount {
        account: ctx.accounts.poll_anti_token.to_account_info(),
        mint: ctx.accounts.anti_mint.to_account_info(),
        authority: ctx.accounts.poll.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };
    token::initialize_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        &[&[
            b"poll",
            ctx.accounts.state.poll_index.to_le_bytes().as_ref(),
            b"anti_token",
            &[ctx.bumps.poll_anti_token],
        ]],
    ))?;

    let cpi_accounts = token::InitializeAccount {
        account: ctx.accounts.poll_pro_token.to_account_info(),
        mint: ctx.accounts.pro_mint.to_account_info(),
        authority: ctx.accounts.poll.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };
    token::initialize_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        &[&[
            b"poll",
            ctx.accounts.state.poll_index.to_le_bytes().as_ref(),
            b"pro_token",
            &[ctx.bumps.poll_pro_token],
        ]],
    ))?;

    // Initialise the poll account
    let poll = &mut ctx.accounts.poll;
    let state = &mut ctx.accounts.state;

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

    let poll_info = poll.to_account_info();
    let state_info = state.to_account_info();
    let mut data_poll = poll_info.try_borrow_mut_data()?;
    let mut data_state = state_info.try_borrow_mut_data()?;

    poll.index = state.poll_index;
    poll.title = title.clone();
    poll.description = description;
    poll.start_time = start_time.clone();
    poll.end_time = end_time.clone();
    poll.etc = etc;
    poll.anti = 0;
    poll.pro = 0;
    poll.deposits = vec![];
    poll.equalised = false;
    poll.equalisation_results = None;

    let serialised_poll = poll.try_to_vec()?;
    data_poll[8..8 + serialised_poll.len()].copy_from_slice(&serialised_poll);

    // Increment poll index
    state.poll_index += 1;

    let serialised_state = state.try_to_vec()?;
    data_state[8..8 + serialised_state.len()].copy_from_slice(&serialised_state);

    // Emit event
    emit!(PollCreatedEvent {
        poll_index: poll.index,
        address: ctx.accounts.authority.key(),
        title,
        start_time,
        end_time,
        timestamp: now,
    });

    Ok(())
}

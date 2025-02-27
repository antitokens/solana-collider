//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 02 Feb 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

// instructions/create_prediction.rs
use crate::utils::*;
use crate::CreatePrediction;
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token;
use anchor_spl::token::spl_token::instruction::AuthorityType;
use anchor_spl::token::SetAuthority;

pub fn create(
    ctx: Context<CreatePrediction>,
    title: String,
    description: String,
    start_time: String,
    end_time: String,
    etc: Option<Vec<u8>>,
) -> Result<()> {
    // Ensure payment is sufficient
    require!(
        ctx.accounts.authority.lamports() >= CREATION_FEE,
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
    let payment_amount = CREATION_FEE;
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
        account_or_mint: ctx.accounts.prediction_anti_token.to_account_info(),
        current_authority: ctx.accounts.authority.to_account_info(),
    };

    token::set_authority(
        CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
        AuthorityType::AccountOwner,
        Some(ANTITOKEN_MULTISIG),
    )?;

    let cpi_accounts = SetAuthority {
        account_or_mint: ctx.accounts.prediction_pro_token.to_account_info(),
        current_authority: ctx.accounts.authority.to_account_info(),
    };

    token::set_authority(
        CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
        AuthorityType::AccountOwner,
        Some(ANTITOKEN_MULTISIG),
    )?;

    // Get account infos for manual serialisation
    let state_info = &ctx.accounts.state.to_account_info();
    let prediction_info = &ctx.accounts.prediction.to_account_info();
    let mut data_state = state_info.try_borrow_mut_data()?;
    let mut data_prediction = prediction_info.try_borrow_mut_data()?;

    // Set prediction data
    ctx.accounts.prediction.index = ctx.accounts.state.index;
    ctx.accounts.prediction.title = title.clone();
    ctx.accounts.prediction.description = description;
    ctx.accounts.prediction.start_time = start_time.clone();
    ctx.accounts.prediction.end_time = end_time.clone();
    ctx.accounts.prediction.etc = etc;
    ctx.accounts.prediction.anti = 0;
    ctx.accounts.prediction.pro = 0;
    ctx.accounts.prediction.deposits = vec![];
    ctx.accounts.prediction.equalised = false;
    ctx.accounts.prediction.equalisation = None;

    // Manual serialisation
    let serialised_prediction = ctx.accounts.prediction.try_to_vec()?;
    data_prediction[8..8 + serialised_prediction.len()].copy_from_slice(&serialised_prediction);

    // Increment prediction index
    ctx.accounts.state.index += 1;

    // Manual serialisation for state
    let serialised_state = ctx.accounts.state.try_to_vec()?;
    data_state[8..8 + serialised_state.len()].copy_from_slice(&serialised_state);

    // Emit event
    emit!(CreationEvent {
        index: ctx.accounts.prediction.index,
        address: ctx.accounts.authority.key(),
        title,
        start_time,
        end_time,
        timestamp: now,
    });

    Ok(())
}

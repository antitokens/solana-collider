// instructions/admin.rs
//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's admin instruction set
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 02 Feb 2025
//! Last Modified: 02 Feb 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

use crate::utils::*;
use crate::Admin;
use crate::SetPollTokenAuthority;
use crate::Update;
use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::spl_token::instruction::AuthorityType;
use anchor_spl::token::SetAuthority;

pub fn initialise_admin(ctx: Context<Admin>) -> Result<()> {

let now: i64 = Clock::get()?.unix_timestamp;

    let config = &mut ctx.accounts.admin;
    require!(!config.initialised, PredictError::AlreadyInitialised);

    config.initialised = true;
    config.poll_creation_fee = 100_000_000;
    config.max_title_length = MAX_TITLE_LENGTH;
    config.max_description_length = MAX_DESCRIPTION_LENGTH;
    config.truth_basis = TRUTH_BASIS;
    config.float_basis = FLOAT_BASIS;
    config.min_deposit_amount = MIN_DEPOSIT_AMOUNT;
    config.antitoken_multisig = ANTITOKEN_MULTISIG;
    config.anti_mint_address = ANTI_MINT_ADDRESS;
    config.pro_mint_address = PRO_MINT_ADDRESS;

    emit!(AdminEvent {
        action: "initialise_admin".to_string(),
        args: vec![],
        timestamp: now,
    });

    Ok(())
}

pub fn update_poll_creation_fee(ctx: Context<Update>, new_fee: u64) -> Result<()> {

let now: i64 = Clock::get()?.unix_timestamp;

    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.poll_creation_fee = new_fee;

    emit!(AdminEvent {
        action: "update_poll_creation_fee".to_string(),
        args: vec![KeyValue {
            key: "new_fee".to_string(),
            value: new_fee.to_string(),
        }],
        timestamp: now,
    });

    Ok(())
}

pub fn update_max_title_length(ctx: Context<Update>, new_length: u64) -> Result<()> {

let now: i64 = Clock::get()?.unix_timestamp;

    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.max_title_length = new_length;

    emit!(AdminEvent {
        action: "update_max_title_length".to_string(),
        args: vec![KeyValue {
            key: "new_length".to_string(),
            value: new_length.to_string(),
        }],
        timestamp: now,
    });

    Ok(())
}

pub fn update_max_description_length(ctx: Context<Update>, new_length: u64) -> Result<()> {

let now: i64 = Clock::get()?.unix_timestamp;

    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.max_description_length = new_length;

    emit!(AdminEvent {
        action: "update_max_description_length".to_string(),
        args: vec![KeyValue {
            key: "new_length".to_string(),
            value: new_length.to_string(),
        }],
        timestamp: now,
    });

    Ok(())
}

pub fn update_truth_basis(ctx: Context<Update>, new_basis: u64) -> Result<()> {

let now: i64 = Clock::get()?.unix_timestamp;

    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.truth_basis = new_basis;

    emit!(AdminEvent {
        action: "update_truth_basis".to_string(),
        args: vec![KeyValue {
            key: "new_basis".to_string(),
            value: new_basis.to_string()
        }],
        timestamp: now,
    });

    Ok(())
}

pub fn update_float_basis(ctx: Context<Update>, new_basis: u64) -> Result<()> {

let now: i64 = Clock::get()?.unix_timestamp;

    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.float_basis = new_basis;

    emit!(AdminEvent {
        action: "update_float_basis".to_string(),
        args: vec![KeyValue {
            key: "new_basis".to_string(),
            value: new_basis.to_string(),
        }],
        timestamp: now,
    });

    Ok(())
}

pub fn update_min_deposit_amount(ctx: Context<Update>, new_min_amount: u64) -> Result<()> {

let now: i64 = Clock::get()?.unix_timestamp;

    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.min_deposit_amount = new_min_amount;

    emit!(AdminEvent {
        action: "update_min_deposit_amount".to_string(),
        args: vec![KeyValue {
            key: "new_min_amount".to_string(),
            value: new_min_amount.to_string()
        }],
        timestamp: now,
    });

    Ok(())
}

pub fn update_anti_mint(ctx: Context<Update>, new_mint: Pubkey) -> Result<()> {

let now: i64 = Clock::get()?.unix_timestamp;

    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.anti_mint_address = new_mint;

    emit!(AdminEvent {
        action: "update_anti_mint".to_string(),
        args: vec![KeyValue {
            key: "new_mint".to_string(),
            value: new_mint.to_string()
        }],
        timestamp: now,
    });

    Ok(())
}

pub fn update_pro_mint(ctx: Context<Update>, new_mint: Pubkey) -> Result<()> {

let now: i64 = Clock::get()?.unix_timestamp;

    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.pro_mint_address = new_mint;

    emit!(AdminEvent {
        action: "update_pro_mint".to_string(),
        args: vec![KeyValue {
            key: "new_mint".to_string(),
            value: new_mint.to_string()
        }],
        timestamp: now,
    });

    Ok(())
}

pub fn update_multisig(ctx: Context<Update>, new_multisig: Pubkey) -> Result<()> {

let now: i64 = Clock::get()?.unix_timestamp;

    require!(
        ctx.accounts.authority.key() == ctx.accounts.admin.antitoken_multisig,
        ErrorCode::Unauthorised
    );
    ctx.accounts.admin.antitoken_multisig = new_multisig;

    emit!(AdminEvent {
        action: "update_multisig".to_string(),
        args: vec![KeyValue {
            key: "new_multisig".to_string(),
            value: new_multisig.to_string()
        }],
        timestamp: now,
    });

    Ok(())
}

pub fn set_token_authority(ctx: Context<SetPollTokenAuthority>, poll_index: u64) -> Result<()> {

let now: i64 = Clock::get()?.unix_timestamp;

    // Verify only ANTITOKEN_MULTISIG can execute this
    require!(
        ctx.accounts.authority.key() == ANTITOKEN_MULTISIG,
        ErrorCode::Unauthorised
    );

    // Transfer authority of $ANTI token account to state PDA
    token::set_authority(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            SetAuthority {
                current_authority: ctx.accounts.authority.to_account_info(),
                account_or_mint: ctx.accounts.poll_anti_token.to_account_info(),
            },
        ),
        AuthorityType::AccountOwner,
        Some(ctx.accounts.state.key()),
    )?;

    // Transfer authority of $PRO token account to state PDA
    token::set_authority(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            SetAuthority {
                current_authority: ctx.accounts.authority.to_account_info(),
                account_or_mint: ctx.accounts.poll_pro_token.to_account_info(),
            },
        ),
        AuthorityType::AccountOwner,
        Some(ctx.accounts.state.key()),
    )?;

    emit!(AdminEvent {
        action: "set_token_authority".to_string(),
        args: vec![KeyValue {
            key: "poll_index".to_string(),
            value: poll_index.to_string()
        }],
        timestamp: now,
    });

    Ok(())
}

/// Error codes
#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorised")]
    Unauthorised,
}

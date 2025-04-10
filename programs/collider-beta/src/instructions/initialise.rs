//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's initialisation
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

// instructions/initialise.rs
use crate::Initialise;
use crate::PredictError;
use anchor_lang::prelude::*;

pub fn initialise(ctx: Context<Initialise>) -> Result<()> {
    let state = &mut ctx.accounts.state;

    // Prevent unnecessary state writes if already initialised
    require!(state.index == 0, PredictError::AlreadyInitialised);

    // Directly set values without redundant references
    state.index = 0;
    state.authority = ctx.accounts.authority.key();

    Ok(())
}

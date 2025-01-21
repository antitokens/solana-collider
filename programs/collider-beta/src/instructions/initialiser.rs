//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's initialisation
//! Version: 0.0.1
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

use anchor_lang::prelude::*;
use crate::Initialise;

pub fn initialiser(ctx: Context<Initialise>) -> Result<()> {
    let state = &mut ctx.accounts.state;
    state.poll_count = 0;
    state.authority = ctx.accounts.authority.key();
    Ok(())
}
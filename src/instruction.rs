//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 0.0.1
//! License: MIT
//! Created: 17 Dec 2024
//! Last Modified: 17 Dec 2024
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum CollisionInstruction {
    /// Initialises a new collider pair with vaults
    ///
    /// Accounts expected:
    /// 0. `[writable]` The collider state account
    /// 1. `[]` The BARYON token mint
    /// 2. `[]` The PHOTON token mint
    /// 3. `[writable]` The vault account for ANTI tokens
    /// 4. `[writable]` The vault account for PRO tokens
    /// 5. `[signer]` The account paying for account creation
    /// 6. `[]` The system program
    /// 7. `[]` The token program
    /// 8. `[]` The rent sysvar
    Initialise,

    /// Performs the collision between ANTI and PRO tokens
    ///
    /// Accounts expected:
    /// 0. `[]` The collider state account
    /// 1. `[writable]` The ANTI token account (source)
    /// 2. `[writable]` The PRO token account (source)
    /// 3. `[writable]` The BARYON token account (destination)
    /// 4. `[writable]` The PHOTON token account (destination)
    /// 5. `[]` The BARYON token mint
    /// 6. `[]` The PHOTON token mint
    /// 7. `[writable]` The vault account for ANTI tokens
    /// 8. `[writable]` The vault account for PRO tokens
    /// 9. `[]` The ANTI token mint (for transfer_checked)
    /// 10. `[]` The PRO token mint (for transfer_checked)
    /// 11. `[signer]` The account paying for the transaction
    /// 12. `[]` The system program
    /// 13. `[]` The token program
    /// 14. `[]` The PDA authority account
    Collide {
        /// Amount of ANTI tokens to collide
        anti_amount: u64,
        /// Amount of PRO tokens to collide
        pro_amount: u64,
    },
}

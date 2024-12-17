use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum CollisionInstruction {
    /// Initializes a new collider pair
    /// 
    /// Accounts expected:
    /// 0. `[signer]` The account of the person initialising the collider
    /// 1. `[writable]` The collider state account
    /// 2. `[]` The BARYON token mint
    /// 3. `[]` The PHOTON token mint
    /// 4. `[]` The system program
    Initialize,

    /// Performs the collision between ANTI and PRO tokens
    /// 
    /// Accounts expected:
    /// 0. `[signer]` The account of the person initialising the collider
    /// 1. `[writable]` The ANTI token account
    /// 2. `[writable]` The PRO token account
    /// 3. `[writable]` The BARYON token account
    /// 4. `[writable]` The PHOTON token account
    /// 5. `[]` The token program
    Collide {
        anti_amount: u64,
        pro_amount: u64,
    },
}

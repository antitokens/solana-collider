use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CollisionState {
    pub baryon_mint: Pubkey,
    pub photon_mint: Pubkey,
    pub authority: Pubkey,
}
//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's state enumeration
//! Version: 0.0.1
//! License: MIT
//! Created: 17 Dec 2024
//! Last Modified: 17 Dec 2024
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_pack::{IsInitialized, Pack, Sealed};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CollisionState {
    pub is_initialized: bool,
    pub baryon_mint: Pubkey,
    pub photon_mint: Pubkey,
    pub authority: Pubkey,
    pub vault_anti: Pubkey,
    pub vault_pro: Pubkey,
}

impl Sealed for CollisionState {}

impl IsInitialized for CollisionState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for CollisionState {
    const LEN: usize = 1 + 32 + 32 + 32 + 32 + 32; // bool + 5 Pubkeys

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap();
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let state = Self::try_from_slice(src).map_err(|_| ProgramError::InvalidAccountData)?;
        Ok(state)
    }
}

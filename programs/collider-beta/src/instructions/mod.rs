//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

// instructions/mod.rs
pub mod admin_actions;
pub mod initialise_program;
pub mod create_poll;
pub mod deposit_to_poll;
pub mod equalise_poll;
pub mod bulk_withdraw_from_poll;
pub mod user_withdraw_from_poll;

// Re-export the instruction structs
pub use admin_actions::*;
pub use initialise_program::*;
pub use create_poll::*;
pub use deposit_to_poll::*;
pub use equalise_poll::*;
pub use bulk_withdraw_from_poll::*;
pub use user_withdraw_from_poll::*;

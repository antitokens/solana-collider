//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

// instructions/mod.rs
pub mod admin;
pub mod initialise;
pub mod create;
pub mod deposit;
pub mod equalise;
pub mod bulk_withdraw;
pub mod user_withdraw;

// Re-export the instruction structs
pub use admin::*;
pub use initialise::*;
pub use create::*;
pub use deposit::*;
pub use equalise::*;
pub use bulk_withdraw::*;
pub use user_withdraw::*;

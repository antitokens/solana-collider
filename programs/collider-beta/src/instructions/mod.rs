//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

// instructions/mod.rs
pub mod create_poll;
pub mod deposit;
pub mod equalise;
pub mod initialise;
pub mod withdraw;

// Re-export the instruction structs
pub use create_poll::create;
pub use deposit::deposit;
pub use equalise::equalise;
pub use initialise::initialise;
pub use withdraw::withdraw;

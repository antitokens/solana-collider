//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 0.0.1
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

pub mod create_poll;
pub mod deposit;
pub mod equalise;
pub mod initialise;
pub mod withdraw;

pub use create_poll::*;
pub use deposit::*;
pub use equalise::*;
pub use initialise::*;
pub use withdraw::*;

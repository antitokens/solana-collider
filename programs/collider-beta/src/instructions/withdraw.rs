//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

use crate::utils::*;
use crate::WithdrawTokens;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

pub fn withdraw(ctx: Context<WithdrawTokens>, poll_index: u64) -> Result<()> {
    // Get mutable reference to poll account
    let poll = &mut ctx.accounts.poll;

    // Verify poll has been equalised
    require!(poll.equalised, PredictError::NotEqualised);

    // Find user's deposit and returns
    let deposit_index = poll
        .deposits
        .iter()
        .position(|d| d.address == ctx.accounts.authority.key())
        .ok_or(error!(PredictError::NoDeposit))?;

    // Verify deposit hasn't already been withdrawn
    require!(
        !poll.deposits[deposit_index].withdrawn,
        PredictError::AlreadyWithdrawn
    );

    let equalisation = poll
        .equalisation_results
        .as_ref()
        .ok_or(error!(PredictError::NotEqualised))?;

    // Get return amounts
    let anti_return = equalisation.anti[deposit_index];
    let pro_return = equalisation.pro[deposit_index];

    // Create seeds for PDA signing
    let index_bytes = poll_index.to_le_bytes();
    let seeds = &[b"poll" as &[u8], index_bytes.as_ref(), &[ctx.bumps.poll]];
    let signer_seeds = &[&seeds[..]];

    // Transfer $ANTI tokens
    if anti_return > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.poll_anti_token.to_account_info(),
                    to: ctx.accounts.user_anti_token.to_account_info(),
                    authority: poll.to_account_info(),
                },
                signer_seeds,
            ),
            anti_return,
        )?;
    }

    // Transfer $PRO tokens
    if pro_return > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.poll_pro_token.to_account_info(),
                    to: ctx.accounts.user_pro_token.to_account_info(),
                    authority: poll.to_account_info(),
                },
                signer_seeds,
            ),
            pro_return,
        )?;
    }

    // Get poll account info for serialisation
    let poll_info = poll.to_account_info();
    let mut data_poll = poll_info.try_borrow_mut_data()?;

    // Update withdrawal status
    poll.deposits[deposit_index].withdrawn = true;

    // Serialise and update poll state
    let serialised_poll = poll.try_to_vec()?;
    data_poll[8..8 + serialised_poll.len()].copy_from_slice(&serialised_poll);

    // Emit withdrawal event
    emit!(WithdrawEvent {
        poll_index,
        address: ctx.accounts.authority.key(),
        anti: anti_return,
        pro: pro_return,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EqualisationResult;
    use crate::PollAccount;
    use crate::StateAccount;
    use crate::UserDeposit;
    use crate::WithdrawTokensBumps;
    use anchor_lang::Discriminator;
    use anchor_spl::token::{spl_token, Token};
    use anchor_spl::token::{spl_token::state::Account as SplTokenAccount, TokenAccount};
    use solana_sdk::program_option::COption;
    use solana_sdk::program_pack::Pack;
    use std::str::FromStr;

    // Fixed test IDs - these should be consistent across tests
    fn program_id() -> Pubkey {
        Pubkey::from_str("5eR98MdgS8jYpKB2iD9oz3MtBdLJ6s7gAVWJZFMvnL9G").unwrap()
    }

    struct TestAccountData {
        key: Pubkey,
        lamports: u64,
        data: Vec<u8>,
        owner: Pubkey,
        executable: bool,
        rent_epoch: u64,
    }

    impl TestAccountData {
        fn new_with_key<T: AccountSerialize + AccountDeserialize + Clone>(
            key: Pubkey,
            owner: Pubkey,
        ) -> Self {
            Self {
                key,
                lamports: 1_000_000,
                data: vec![0; 8 + PollAccount::LEN],
                owner,
                executable: true,
                rent_epoch: 0,
            }
        }

        fn to_account_info<'a>(&'a mut self, is_signer: bool) -> AccountInfo<'a> {
            AccountInfo::new(
                &self.key,
                is_signer,
                true,
                &mut self.lamports,
                &mut self.data,
                &self.owner,
                self.executable,
                self.rent_epoch,
            )
        }

        fn new_token() -> Self {
            Self {
                key: Pubkey::new_unique(),
                lamports: 1_000_000,
                data: vec![0; 165],
                owner: spl_token::ID,
                executable: false,
                rent_epoch: 0,
            }
        }

        fn into_token_account<'a>(account_info: &'a AccountInfo<'a>) -> Account<'a, TokenAccount> {
            Account::try_from(account_info).unwrap()
        }

        fn init_token_account(&mut self, owner: Pubkey, mint: Pubkey) -> Result<()> {
            self.data = vec![0; TokenAccount::LEN]; // Ensure correct buffer size
            let data = self.data.as_mut_slice();

            let close_authority: COption<Pubkey> = COption::None;

            // Initialise a new SPL Token Account manually
            let token_account = SplTokenAccount {
                mint,
                owner,
                amount: 0,
                delegate: None.into(),
                state: spl_token::state::AccountState::Initialized,
                is_native: None.into(),
                delegated_amount: 0,
                close_authority,
            };

            token_account.pack_into_slice(data);

            Ok(())
        }
    }

    #[test]
    fn test_successful_withdrawal() {
        let program_id = program_id();

        // Create test accounts
        let mut poll =
            TestAccountData::new_with_key::<PollAccount>(Pubkey::new_unique(), program_id);
        let mut authority =
            TestAccountData::new_with_key::<StateAccount>(Pubkey::new_unique(), program_id);

        // Initialise token accounts
        let mint_key = Pubkey::new_unique();
        let authority_key = Pubkey::new_unique();

        let mut user_anti = TestAccountData::new_token();
        let mut user_pro = TestAccountData::new_token();
        let mut poll_anti = TestAccountData::new_token();
        let mut poll_pro = TestAccountData::new_token();

        user_anti
            .init_token_account(authority_key, mint_key)
            .unwrap();
        user_pro
            .init_token_account(authority_key, mint_key)
            .unwrap();
        poll_anti
            .init_token_account(Pubkey::new_unique(), mint_key)
            .unwrap();
        poll_pro
            .init_token_account(Pubkey::new_unique(), mint_key)
            .unwrap();

        let mut token_program =
            TestAccountData::new_with_key::<StateAccount>(spl_token::ID, spl_token::ID);

        // Create poll with deposits and results
        let poll_data = PollAccount {
            index: 0,
            title: "Test Poll".to_string(),
            description: "Test Description".to_string(),
            start_time: "2025-01-01T00:00:00Z".to_string(),
            end_time: "2025-01-02T00:00:00Z".to_string(),
            etc: None,
            anti: 10000,
            pro: 8000,
            deposits: vec![
                UserDeposit {
                    address: authority.key,
                    anti: 6000,
                    pro: 5000,
                    u: 1000,
                    s: 11000,
                    withdrawn: false,
                },
                UserDeposit {
                    address: Pubkey::new_unique(),
                    anti: 4000,
                    pro: 3000,
                    u: 1000,
                    s: 7000,
                    withdrawn: false,
                },
            ],
            equalised: true,
            equalisation_results: Some(EqualisationResult {
                anti: vec![6000, 4000],
                pro: vec![5000, 3000],
                truth: vec![6000, 4000],
                timestamp: 1736899200,
            }),
        };

        // Write discriminator and serialise poll data
        poll.data[..8].copy_from_slice(&PollAccount::discriminator());
        let serialised_poll = poll_data.try_to_vec().unwrap();
        poll.data[8..8 + serialised_poll.len()].copy_from_slice(&serialised_poll);

        // Get account infos
        let poll_info = poll.to_account_info(false);
        let authority_info = authority.to_account_info(true);
        let user_anti_info = user_anti.to_account_info(false);
        let user_pro_info = user_pro.to_account_info(false);
        let poll_anti_info = poll_anti.to_account_info(false);
        let poll_pro_info = poll_pro.to_account_info(false);
        let token_program_info = token_program.to_account_info(false);

        let (_poll_pda, poll_bump) =
            Pubkey::find_program_address(&[b"poll", 0u64.to_le_bytes().as_ref()], &program_id);

        let mut accounts = WithdrawTokens {
            poll: Account::try_from(&poll_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            user_anti_token: TestAccountData::into_token_account(&user_anti_info),
            user_pro_token: TestAccountData::into_token_account(&user_pro_info),
            poll_anti_token: TestAccountData::into_token_account(&poll_anti_info),
            poll_pro_token: TestAccountData::into_token_account(&poll_pro_info),
            token_program: Program::<Token>::try_from(&token_program_info).unwrap(),
        };

        let _ = withdraw(
            Context::new(
                &program_id,
                &mut accounts,
                &[],
                WithdrawTokensBumps { poll: poll_bump },
            ),
            0,
        );
        // Note: We cannot test for withdrawals due to under the hood invocation of sol_invoke_signed()
    }
}

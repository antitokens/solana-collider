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
use anchor_spl::token::TokenAccount;
use anchor_spl::token::{self, Transfer};

pub fn withdraw<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, WithdrawTokens<'info>>,
    poll_index: u64,
) -> Result<()> {
    // Verify only ANTITOKEN_MULTISIG can execute this
    let authority_key = ctx.accounts.authority.key();
    require!(
        authority_key == ANTITOKEN_MULTISIG,
        PredictError::Unauthorised
    );

    // Verify poll has been equalised
    require!(&ctx.accounts.poll.equalised, PredictError::NotEqualised);

    let equalisation = &ctx
        .accounts
        .poll
        .equalisation_results
        .clone()
        .ok_or(error!(PredictError::NotEqualised))?;

    // Track total withdrawn amounts for verification
    let mut total_anti_withdrawn: u64 = 0;
    let mut total_pro_withdrawn: u64 = 0;

    // Get the poll account
    let poll = &mut ctx.accounts.poll;

    // Use explicit lifetime for `remaining_accounts`
    let remaining_accounts: &'info [AccountInfo<'info>] = ctx.remaining_accounts;
    let mut deposits = poll.deposits.clone();
    let num_deposits = deposits.len();
    let enum_deposits = deposits.iter_mut().enumerate();

    require!(
        remaining_accounts.len() == num_deposits * 2,
        PredictError::InvalidTokenAccount
    );

    // Iterate through deposits
    for (deposit_index, deposit) in enum_deposits {
        // Skip if already withdrawn
        if deposit.withdrawn {
            continue;
        }

        // Get return amounts for this deposit
        let anti_return = equalisation.anti[deposit_index];
        let pro_return = equalisation.pro[deposit_index];

        // Fix: Explicitly specify lifetime for `Account<TokenAccount>`
        let user_anti_token: Account<'info, TokenAccount> =
            Account::try_from(&remaining_accounts[deposit_index])?;
        let user_pro_token: Account<'info, TokenAccount> =
            Account::try_from(&remaining_accounts[deposit_index + num_deposits])?;

        // Transfer $ANTI tokens
        if anti_return > 0 {
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.poll_anti_token.to_account_info(),
                        to: user_anti_token.to_account_info(),
                        authority: ctx.accounts.authority.to_account_info(),
                    },
                    &[],
                ),
                anti_return,
            )?;
            total_anti_withdrawn += anti_return;
        }

        // Transfer $PRO tokens
        if pro_return > 0 {
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.poll_pro_token.to_account_info(),
                        to: user_pro_token.to_account_info(),
                        authority: ctx.accounts.authority.to_account_info(),
                    },
                    &[],
                ),
                pro_return,
            )?;
            total_pro_withdrawn += pro_return;
        }

        // Mark as withdrawn
        deposit.withdrawn = true;
    }

    // Verify total withdrawn matches equalisation results
    require!(
        total_anti_withdrawn == equalisation.anti.iter().copied().sum::<u64>(),
        PredictError::InvalidEqualisation
    );
    require!(
        total_pro_withdrawn == equalisation.pro.iter().copied().sum::<u64>(),
        PredictError::InvalidEqualisation
    );

    // Emit bulk withdrawal event
    emit!(WithdrawEvent {
        poll_index,
        address: ctx.accounts.authority.key(),
        anti: total_anti_withdrawn,
        pro: total_pro_withdrawn,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::PROGRAM_ID;
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
        Pubkey::from_str(PROGRAM_ID).unwrap()
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

        fn init_state_data(&mut self, state: &StateAccount) -> Result<()> {
            self.data = vec![0; 8 + StateAccount::LEN];
            let data = self.data.as_mut_slice();

            let disc = StateAccount::discriminator();
            data[..8].copy_from_slice(&disc);

            let account_data = state.try_to_vec()?;
            data[8..8 + account_data.len()].copy_from_slice(&account_data);

            Ok(())
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

        // Initialise state account
        let root: Pubkey = Pubkey::new_unique();
        let mut state = TestAccountData::new_with_key::<StateAccount>(root, program_id);
        state
            .init_state_data(&StateAccount {
                poll_index: 0,
                authority: root,
            })
            .unwrap();

        let (_poll_pda, poll_bump) =
            Pubkey::find_program_address(&[b"poll", 0u64.to_le_bytes().as_ref()], &program_id);

        let (_anti_token_pda, anti_token_bump) = Pubkey::find_program_address(
            &[b"anti_token", state.data[8..16].try_into().unwrap()],
            &program_id,
        );

        let (_pro_token_pda, pro_token_bump) = Pubkey::find_program_address(
            &[b"pro_token", state.data[8..16].try_into().unwrap()],
            &program_id,
        );

        // Use `remaining_accounts` dynamically
        let remaining_accounts = vec![user_anti_info.clone(), user_pro_info.clone()];

        let mut accounts = WithdrawTokens {
            poll: Account::try_from(&poll_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            poll_anti_token: Account::try_from(&poll_anti_info).unwrap(),
            poll_pro_token: Account::try_from(&poll_pro_info).unwrap(),
            token_program: Program::<Token>::try_from(&token_program_info).unwrap(),
        };

        let bumps = WithdrawTokensBumps {
            poll: poll_bump,
            poll_anti_token: anti_token_bump,
            poll_pro_token: pro_token_bump,
        };

        let _ = withdraw(
            Context::new(
                &program_id,
                &mut accounts,
                &remaining_accounts, // Pass dynamically created accounts
                bumps,
            ),
            0,
        );
    }
}

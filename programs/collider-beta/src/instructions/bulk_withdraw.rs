//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 1.0.0-beta
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

use crate::utils::*;
use crate::BulkWithdrawTokens;
use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use anchor_spl::token::{self, Transfer};
use borsh::BorshSerialize;

pub fn bulk_withdraw<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, BulkWithdrawTokens<'info>>,
    index: u64,
) -> Result<()> {
    let authority_key = ctx.accounts.authority.key();
    require!(
        authority_key == ANTITOKEN_MULTISIG,
        PredictError::Unauthorised
    );
    require!(
        &ctx.accounts.prediction.equalised,
        PredictError::NotEqualised
    );

    let equalisation = ctx
        .accounts
        .prediction
        .equalisation
        .clone()
        .ok_or(error!(PredictError::NotEqualised))?;

    let prediction_info = ctx.accounts.prediction.to_account_info();
    let mut prediction_data = prediction_info.try_borrow_mut_data()?;

    let prediction = &mut ctx.accounts.prediction;
    let remaining_accounts: &[AccountInfo<'info>] = ctx.remaining_accounts;
    let mut deposits = prediction.deposits.clone();
    let num_deposits = deposits.len();

    require!(
        remaining_accounts.len() == num_deposits * 2,
        PredictError::InvalidTokenAccount
    );

    let mut total_anti_withdrawn: u64 = 0;
    let mut total_pro_withdrawn: u64 = 0;

    for (deposit_index, deposit) in deposits.iter_mut().enumerate() {
        if deposit.withdrawn {
            continue;
        }

        let anti_return = equalisation.anti[deposit_index];
        let pro_return = equalisation.pro[deposit_index];

        let user_anti_token: Account<'info, TokenAccount> =
            Account::try_from(&remaining_accounts[deposit_index * 2])?;
        let user_pro_token: Account<'info, TokenAccount> =
            Account::try_from(&remaining_accounts[deposit_index * 2 + 1])?;

        if anti_return > 0 {
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.prediction_anti_token.to_account_info(),
                        to: user_anti_token.to_account_info(),
                        authority: ctx.accounts.authority.to_account_info(),
                    },
                    &[],
                ),
                anti_return,
            )?;
            total_anti_withdrawn += anti_return;
        }

        if pro_return > 0 {
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.prediction_pro_token.to_account_info(),
                        to: user_pro_token.to_account_info(),
                        authority: ctx.accounts.authority.to_account_info(),
                    },
                    &[],
                ),
                pro_return,
            )?;
            total_pro_withdrawn += pro_return;
        }

        deposit.withdrawn = true;
    }

    // Verify total withdrawals match equalisation sums
    require!(
        total_anti_withdrawn == equalisation.anti.iter().copied().sum::<u64>(),
        PredictError::InvalidEqualisation
    );
    require!(
        total_pro_withdrawn == equalisation.pro.iter().copied().sum::<u64>(),
        PredictError::InvalidEqualisation
    );

    // Serialise updated prediction state and store it in account data
    prediction.deposits = deposits;
    let serialised_prediction = prediction.try_to_vec()?;
    prediction_data[8..8 + serialised_prediction.len()].copy_from_slice(&serialised_prediction);

    emit!(WithdrawEvent {
        index,
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
    use crate::BulkWithdrawTokensBumps;
    use crate::Deposit;
    use crate::Equalisation;
    use crate::PredictionAccount;
    use crate::StateAccount;
    use anchor_lang::system_program;
    use anchor_lang::Discriminator;
    use anchor_spl::token::{spl_token, Token};
    use anchor_spl::token::{spl_token::state::Account as SplTokenAccount, TokenAccount};
    use solana_sdk::program_option::COption;
    use solana_sdk::program_pack::Pack;
    use std::str::FromStr;

    // Fixed test IDs - these should be consistent across tests
    fn program_id() -> Pubkey {
        Pubkey::from_str(&PROGRAM_ID.to_string()).unwrap()
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
        fn new_account_with_key_and_owner<T: AccountSerialize + AccountDeserialize + Clone>(
            key: Pubkey,
            owner: Pubkey,
        ) -> Self {
            Self {
                key,
                lamports: 1_000_000,
                data: vec![0; 8 + PredictionAccount::LEN],
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

        fn new_authority_account(pubkey: Pubkey) -> Self {
            Self {
                key: pubkey,
                lamports: 1_000_000,
                data: vec![],
                owner: system_program::ID,
                executable: true,
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

        // Reusable method to create an equalised test prediction
        fn create_equalised_test_prediction(authority: Pubkey) -> PredictionAccount {
            PredictionAccount {
                index: 0,
                title: "Test Prediction".to_string(),
                description: "Test Description".to_string(),
                start_time: "2025-01-01T00:00:00Z".to_string(),
                end_time: "2025-01-02T00:00:00Z".to_string(),
                etc: None,
                anti: 10000,
                pro: 8000,
                deposits: vec![
                    Deposit {
                        address: authority,
                        anti: 6000,
                        pro: 5000,
                        mean: 1000,
                        stddev: 11000,
                        withdrawn: false,
                    },
                    Deposit {
                        address: Pubkey::new_unique(),
                        anti: 4000,
                        pro: 3000,
                        mean: 1000,
                        stddev: 7000,
                        withdrawn: false,
                    },
                ],
                equalised: true,
                equalisation: Some(Equalisation {
                    anti: vec![6000, 4000],
                    pro: vec![5000, 3000],
                    truth: vec![6000, 4000],
                    timestamp: 1736899200,
                }),
            }
        }
    }

    #[test]
    fn test_successful_withdrawal() {
        let program_id = program_id();

        // Create test accounts
        let mut prediction = TestAccountData::new_account_with_key_and_owner::<PredictionAccount>(
            Pubkey::new_unique(),
            program_id,
        );
        let mut authority = TestAccountData::new_authority_account(Pubkey::new_unique());

        // Initialise token accounts
        let mint_key = Pubkey::new_unique();
        let authority_key = Pubkey::new_unique();

        let mut user_anti = TestAccountData::new_token();
        let mut user_pro = TestAccountData::new_token();
        let mut prediction_anti = TestAccountData::new_token();
        let mut prediction_pro = TestAccountData::new_token();

        user_anti
            .init_token_account(authority_key, mint_key)
            .unwrap();
        user_pro
            .init_token_account(authority_key, mint_key)
            .unwrap();
        prediction_anti
            .init_token_account(Pubkey::new_unique(), mint_key)
            .unwrap();
        prediction_pro
            .init_token_account(Pubkey::new_unique(), mint_key)
            .unwrap();

        let mut token_program = TestAccountData::new_account_with_key_and_owner::<StateAccount>(
            spl_token::ID,
            spl_token::ID,
        );

        // Create prediction with deposits and results
        let prediction_data = TestAccountData::create_equalised_test_prediction(authority.key);

        // Write discriminator and serialise prediction data
        prediction.data[..8].copy_from_slice(&PredictionAccount::discriminator());
        let serialised_prediction = prediction_data.try_to_vec().unwrap();
        prediction.data[8..8 + serialised_prediction.len()].copy_from_slice(&serialised_prediction);

        // Get account infos
        let prediction_info = prediction.to_account_info(false);
        let authority_info = authority.to_account_info(true);
        let user_anti_info = user_anti.to_account_info(false);
        let user_pro_info = user_pro.to_account_info(false);
        let prediction_anti_info = prediction_anti.to_account_info(false);
        let prediction_pro_info = prediction_pro.to_account_info(false);
        let token_program_info = token_program.to_account_info(false);

        // Initialise state account
        let root: Pubkey = Pubkey::new_unique();
        let mut state =
            TestAccountData::new_account_with_key_and_owner::<StateAccount>(root, program_id);
        state
            .init_state_data(&StateAccount {
                index: 0,
                authority: root,
            })
            .unwrap();

        let (_prediction_pda, prediction_bump) = Pubkey::find_program_address(
            &[b"prediction", 0u64.to_le_bytes().as_ref()],
            &program_id,
        );

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

        let mut accounts = BulkWithdrawTokens {
            prediction: Account::try_from(&prediction_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            prediction_anti_token: Account::try_from(&prediction_anti_info).unwrap(),
            prediction_pro_token: Account::try_from(&prediction_pro_info).unwrap(),
            token_program: Program::<Token>::try_from(&token_program_info).unwrap(),
        };

        let bumps = BulkWithdrawTokensBumps {
            prediction: prediction_bump,
            prediction_anti_token: anti_token_bump,
            prediction_pro_token: pro_token_bump,
        };

        let _ = bulk_withdraw(
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

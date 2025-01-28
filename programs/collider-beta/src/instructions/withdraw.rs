//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 0.0.1
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
    // First, get all the values we need
    let poll = &ctx.accounts.poll;

    // Verify poll has been equalised
    require!(poll.equalised, PredictError::NotEqualised);

    // Find user's deposit and returns
    let deposit_index = poll
        .deposits
        .iter()
        .position(|d| d.address == ctx.accounts.authority.key())
        .ok_or(error!(PredictError::NoDeposit))?;

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

    // Transfer ANTI tokens
    if anti_return > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.poll_anti_token.to_account_info(),
                    to: ctx.accounts.user_anti_token.to_account_info(),
                    authority: ctx.accounts.poll.to_account_info(),
                },
                signer_seeds,
            ),
            anti_return,
        )?;
    }

    // Transfer PRO tokens
    if pro_return > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.poll_pro_token.to_account_info(),
                    to: ctx.accounts.user_pro_token.to_account_info(),
                    authority: ctx.accounts.poll.to_account_info(),
                },
                signer_seeds,
            ),
            pro_return,
        )?;
    }

    // Update user's withdrawal status
    let poll = &mut ctx.accounts.poll;
    poll.deposits[deposit_index].withdrawn = true;

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

/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::EqualisationResult;
    use crate::PollAccount;
    use crate::UserDeposit;
    use crate::WithdrawTokensBumps;
    use anchor_lang::solana_program::system_program;
    use anchor_spl::token::spl_token;
    use anchor_spl::token::{Token, TokenAccount};
    use std::str::FromStr;

    struct TestAccountData {
        key: Pubkey,
        lamports: u64,
        data: Vec<u8>,
        owner: Pubkey,
        executable: bool,
        rent_epoch: u64,
    }

    impl TestAccountData {
        fn new_owned(owner: Pubkey) -> Self {
            Self {
                key: Pubkey::new_unique(),
                lamports: 1_000_000,
                data: vec![0; 1000],
                owner,
                executable: false,
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

        fn into_token_account<'a>(account_info: &'a AccountInfo<'a>) -> Account<'a, TokenAccount> {
            Account::try_from(account_info).unwrap()
        }
    }

    #[test]
    fn test_successful_withdrawal() {
        let program_id = Pubkey::from_str("5eR98MdgS8jYpKB2iD9oz3MtBdLJ6s7gAVWJZFMvnL9G").unwrap();
        let authority = Pubkey::new_unique();

        // Create test accounts
        let mut poll = TestAccountData::new_owned(program_id);
        let mut auth_data = TestAccountData::new_owned(system_program::ID);
        let mut user_anti = TestAccountData::new_owned(program_id);
        let mut user_pro = TestAccountData::new_owned(program_id);
        let mut poll_anti = TestAccountData::new_owned(program_id);
        let mut poll_pro = TestAccountData::new_owned(program_id);
        let mut token_program = TestAccountData::new_owned(spl_token::ID);

        // Create poll with deposits and results
        let poll_data = PollAccount {
            index: 0,
            title: "Test Poll".to_string(),
            description: "Test Description".to_string(),
            start_time: "2025-01-20T00:00:00Z".to_string(),
            end_time: "2025-01-21T00:00:00Z".to_string(),
            etc: None,
            anti: 10000,
            pro: 8000,
            deposits: vec![
                UserDeposit {
                    address: authority,
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
                timestamp: 1706745600,
            }),
        };

        // Get account infos
        let poll_info = poll.to_account_info(false);
        let authority_info = auth_data.to_account_info(true);
        let user_anti_info = user_anti.to_account_info(false);
        let user_pro_info = user_pro.to_account_info(false);
        let poll_anti_info = poll_anti.to_account_info(false);
        let poll_pro_info = poll_pro.to_account_info(false);
        let token_program_info = token_program.to_account_info(false);

        // Serialise poll data
        poll_info
            .try_borrow_mut_data()
            .unwrap()
            .copy_from_slice(&poll_data.try_to_vec().unwrap());

        let mut accounts = WithdrawTokens {
            poll: Account::try_from(&poll_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            user_anti_token: TestAccountData::into_token_account(&user_anti_info),
            user_pro_token: TestAccountData::into_token_account(&user_pro_info),
            poll_anti_token: TestAccountData::into_token_account(&poll_anti_info),
            poll_pro_token: TestAccountData::into_token_account(&poll_pro_info),
            token_program: Program::<Token>::try_from(&token_program_info).unwrap(),
        };

        let result = withdraw(
            Context::new(
                &program_id,
                &mut accounts,
                &[],
                WithdrawTokensBumps { poll: 255 },
            ),
            0,
        );

        assert!(result.is_ok());

        // Verify withdrawal state
        let poll_account: PollAccount =
            PollAccount::try_deserialize(&mut poll_info.try_borrow_data().unwrap().as_ref())
                .unwrap();

        let user_deposit = poll_account
            .deposits
            .iter()
            .find(|d| d.address == authority)
            .unwrap();
        assert!(user_deposit.withdrawn);

        let other_deposit = poll_account
            .deposits
            .iter()
            .find(|d| d.address != authority)
            .unwrap();
        assert!(!other_deposit.withdrawn);
    }

    #[test]
    fn test_withdrawal_validation() {
        let program_id = Pubkey::from_str("5eR98MdgS8jYpKB2iD9oz3MtBdLJ6s7gAVWJZFMvnL9G").unwrap();
        let authority = Pubkey::new_unique();

        // Create test accounts
        let mut poll = TestAccountData::new_owned(program_id);
        let mut auth_data = TestAccountData::new_owned(system_program::ID);
        let mut user_anti = TestAccountData::new_owned(program_id);
        let mut user_pro = TestAccountData::new_owned(program_id);
        let mut poll_anti = TestAccountData::new_owned(program_id);
        let mut poll_pro = TestAccountData::new_owned(program_id);
        let mut token_program = TestAccountData::new_owned(spl_token::ID);

        // Test withdrawal before equalisation
        {
            let poll_data = PollAccount {
                index: 0,
                title: "Test Poll".to_string(),
                description: "Test Description".to_string(),
                start_time: "2025-01-20T00:00:00Z".to_string(),
                end_time: "2025-01-21T00:00:00Z".to_string(),
                etc: None,
                anti: 10000,
                pro: 8000,
                deposits: vec![UserDeposit {
                    address: authority,
                    anti: 6000,
                    pro: 5000,
                    u: 1000,
                    s: 11000,
                    withdrawn: false,
                }],
                equalised: false,
                equalisation_results: None,
            };

            // Get account infos and create context
            let poll_info = poll.to_account_info(false);
            let authority_info = auth_data.to_account_info(true);
            let user_anti_info = user_anti.to_account_info(false);
            let user_pro_info = user_pro.to_account_info(false);
            let poll_anti_info = poll_anti.to_account_info(false);
            let poll_pro_info = poll_pro.to_account_info(false);
            let token_program_info = token_program.to_account_info(false);

            poll_info
                .try_borrow_mut_data()
                .unwrap()
                .copy_from_slice(&poll_data.try_to_vec().unwrap());

            let mut accounts = WithdrawTokens {
                poll: Account::try_from(&poll_info).unwrap(),
                authority: Signer::try_from(&authority_info).unwrap(),
                user_anti_token: TestAccountData::into_token_account(&user_anti_info),
                user_pro_token: TestAccountData::into_token_account(&user_pro_info),
                poll_anti_token: TestAccountData::into_token_account(&poll_anti_info),
                poll_pro_token: TestAccountData::into_token_account(&poll_pro_info),
                token_program: Program::<Token>::try_from(&token_program_info).unwrap(),
            };

            let result = withdraw(
                Context::new(
                    &program_id,
                    &mut accounts,
                    &[],
                    WithdrawTokensBumps { poll: 255 },
                ),
                0,
            );

            match result {
                Err(err) => assert_eq!(err, Error::from(PredictError::NotEqualised)),
                _ => panic!("Expected not equalised error"),
            }
        }

        // Test no deposit
        {
            let poll_data = PollAccount {
                index: 0,
                title: "Test Poll".to_string(),
                description: "Test Description".to_string(),
                start_time: "2025-01-20T00:00:00Z".to_string(),
                end_time: "2025-01-21T00:00:00Z".to_string(),
                etc: None,
                anti: 10000,
                pro: 8000,
                deposits: vec![],
                equalised: true,
                equalisation_results: Some(EqualisationResult {
                    anti: vec![6000, 4000],
                    pro: vec![5000, 3000],
                    truth: vec![6000, 4000],
                    timestamp: 1706745600,
                }),
            };

            // Get account infos and create context with different authority
            let poll_info = poll.to_account_info(false);
            let mut different_authority = TestAccountData::new_owned(system_program::ID);
            let authority_info = different_authority.to_account_info(true);
            let user_anti_info = user_anti.to_account_info(false);
            let user_pro_info = user_pro.to_account_info(false);
            let poll_anti_info = poll_anti.to_account_info(false);
            let poll_pro_info = poll_pro.to_account_info(false);
            let token_program_info = token_program.to_account_info(false);

            poll_info
                .try_borrow_mut_data()
                .unwrap()
                .copy_from_slice(&poll_data.try_to_vec().unwrap());

            let mut accounts = WithdrawTokens {
                poll: Account::try_from(&poll_info).unwrap(),
                authority: Signer::try_from(&authority_info).unwrap(),
                user_anti_token: TestAccountData::into_token_account(&user_anti_info),
                user_pro_token: TestAccountData::into_token_account(&user_pro_info),
                poll_anti_token: TestAccountData::into_token_account(&poll_anti_info),
                poll_pro_token: TestAccountData::into_token_account(&poll_pro_info),
                token_program: Program::<Token>::try_from(&token_program_info).unwrap(),
            };

            let result = withdraw(
                Context::new(
                    &program_id,
                    &mut accounts,
                    &[],
                    WithdrawTokensBumps { poll: 255 },
                ),
                0,
            );

            match result {
                Err(err) => assert_eq!(err, Error::from(PredictError::NoDeposit)),
                _ => panic!("Expected no deposit error"),
            }
        }

        // Test already withdrawn
        {
            let poll_data = PollAccount {
                index: 0,
                title: "Test Poll".to_string(),
                description: "Test Description".to_string(),
                start_time: "2025-01-20T00:00:00Z".to_string(),
                end_time: "2025-01-21T00:00:00Z".to_string(),
                etc: None,
                anti: 10000,
                pro: 8000,
                deposits: vec![UserDeposit {
                    address: authority,
                    anti: 6000,
                    pro: 5000,
                    u: 1000,
                    s: 11000,
                    withdrawn: true,
                }],
                equalised: true,
                equalisation_results: Some(EqualisationResult {
                    anti: vec![6000],
                    pro: vec![5000],
                    truth: vec![6000, 4000],
                    timestamp: 1706745600,
                }),
            };

            // Get account infos and create context
            let poll_info = poll.to_account_info(false);
            let authority_info = auth_data.to_account_info(true);
            let user_anti_info = user_anti.to_account_info(false);
            let user_pro_info = user_pro.to_account_info(false);
            let poll_anti_info = poll_anti.to_account_info(false);
            let poll_pro_info = poll_pro.to_account_info(false);
            let token_program_info = token_program.to_account_info(false);

            poll_info
                .try_borrow_mut_data()
                .unwrap()
                .copy_from_slice(&poll_data.try_to_vec().unwrap());

            let mut accounts = WithdrawTokens {
                poll: Account::try_from(&poll_info).unwrap(),
                authority: Signer::try_from(&authority_info).unwrap(),
                user_anti_token: TestAccountData::into_token_account(&user_anti_info),
                user_pro_token: TestAccountData::into_token_account(&user_pro_info),
                poll_anti_token: TestAccountData::into_token_account(&poll_anti_info),
                poll_pro_token: TestAccountData::into_token_account(&poll_pro_info),
                token_program: Program::<Token>::try_from(&token_program_info).unwrap(),
            };

            let result = withdraw(
                Context::new(
                    &program_id,
                    &mut accounts,
                    &[],
                    WithdrawTokensBumps { poll: 255 },
                ),
                0,
            );

            match result {
                Err(err) => assert_eq!(err, Error::from(PredictError::AlreadyWithdrawn)),
                _ => panic!("Expected already withdrawn error"),
            }
        }
    }

    #[test]
    fn test_withdrawal_amounts() {
        let program_id = Pubkey::from_str("5eR98MdgS8jYpKB2iD9oz3MtBdLJ6s7gAVWJZFMvnL9G").unwrap();
        let authority = Pubkey::new_unique();

        // Create test accounts
        let mut poll = TestAccountData::new_owned(program_id);
        let mut auth_data = TestAccountData::new_owned(system_program::ID);
        let mut user_anti = TestAccountData::new_owned(program_id);
        let mut user_pro = TestAccountData::new_owned(program_id);
        let mut poll_anti = TestAccountData::new_owned(program_id);
        let mut poll_pro = TestAccountData::new_owned(program_id);
        let mut token_program = TestAccountData::new_owned(spl_token::ID);

        // Create poll with deposits and results
        let poll_data = PollAccount {
            index: 0,
            title: "Test Poll".to_string(),
            description: "Test Description".to_string(),
            start_time: "2025-01-20T00:00:00Z".to_string(),
            end_time: "2025-01-21T00:00:00Z".to_string(),
            etc: None,
            anti: 10000,
            pro: 8000,
            deposits: vec![UserDeposit {
                address: authority,
                anti: 6000,
                pro: 5000,
                u: 1000,
                s: 11000,
                withdrawn: false,
            }],
            equalised: true,
            equalisation_results: Some(EqualisationResult {
                anti: vec![6000],
                pro: vec![5000],
                truth: vec![6000, 4000],
                timestamp: 1706745600,
            }),
        };

        // Get account infos
        let poll_info = poll.to_account_info(false);
        let authority_info = auth_data.to_account_info(true);
        let user_anti_info = user_anti.to_account_info(false);
        let user_pro_info = user_pro.to_account_info(false);
        let poll_anti_info = poll_anti.to_account_info(false);
        let poll_pro_info = poll_pro.to_account_info(false);
        let token_program_info = token_program.to_account_info(false);

        // Serialise poll data
        poll_info
            .try_borrow_mut_data()
            .unwrap()
            .copy_from_slice(&poll_data.try_to_vec().unwrap());

        let mut accounts = WithdrawTokens {
            poll: Account::try_from(&poll_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            user_anti_token: TestAccountData::into_token_account(&user_anti_info),
            user_pro_token: TestAccountData::into_token_account(&user_pro_info),
            poll_anti_token: TestAccountData::into_token_account(&poll_anti_info),
            poll_pro_token: TestAccountData::into_token_account(&poll_pro_info),
            token_program: Program::<Token>::try_from(&token_program_info).unwrap(),
        };

        let result = withdraw(
            Context::new(
                &program_id,
                &mut accounts,
                &[],
                WithdrawTokensBumps { poll: 255 },
            ),
            0,
        );

        assert!(result.is_ok());

        // Verify withdrawal amounts
        let poll_account: PollAccount =
            PollAccount::try_deserialize(&mut poll_info.try_borrow_data().unwrap().as_ref())
                .unwrap();

        let results = poll_account.equalisation_results.unwrap();
        let deposit_index = poll_account
            .deposits
            .iter()
            .position(|d| d.address == authority)
            .unwrap();

        assert_eq!(results.anti[deposit_index], 6000);
        assert_eq!(results.pro[deposit_index], 5000);

        // Verify token account balances
        let user_anti_account = TestAccountData::into_token_account(&user_anti_info);
        let user_pro_account = TestAccountData::into_token_account(&user_pro_info);
        let poll_anti_account = TestAccountData::into_token_account(&poll_anti_info);
        let poll_pro_account = TestAccountData::into_token_account(&poll_pro_info);

        // Check token transfers occurred correctly
        assert_eq!(user_anti_account.amount, results.anti[deposit_index]);
        assert_eq!(user_pro_account.amount, results.pro[deposit_index]);
        assert_eq!(
            poll_anti_account.amount,
            poll_data.anti - results.anti[deposit_index]
        );
        assert_eq!(
            poll_pro_account.amount,
            poll_data.pro - results.pro[deposit_index]
        );
    }
}
*/

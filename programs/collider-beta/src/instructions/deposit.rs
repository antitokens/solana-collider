//! Program Author: sshmatrix, for Antitoken
//! Program Description: Collider's instruction set
//! Version: 0.0.1
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

// instructions/deposit.rs
use crate::state::*;
use crate::utils::*;
use crate::DepositTokens;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

pub fn deposit(ctx: Context<DepositTokens>, poll_index: u64, anti: u64, pro: u64) -> Result<()> {
    let poll = &mut ctx.accounts.poll;

    // Verify poll is active
    let clock = Clock::get()?;
    require!(
        poll.is_active(clock.unix_timestamp),
        PredictError::PollInactive
    );

    // Verify minimum deposit
    require!(
        anti >= MIN_DEPOSIT_AMOUNT || pro >= MIN_DEPOSIT_AMOUNT,
        PredictError::InsufficientDeposit
    );

    // Transfer ANTI tokens if amount > 0
    if anti > 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_anti_token.to_account_info(),
                    to: ctx.accounts.poll_anti_token.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            anti,
        )?;
    }

    // Transfer PRO tokens if amount > 0
    if pro > 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_pro_token.to_account_info(),
                    to: ctx.accounts.poll_pro_token.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            pro,
        )?;
    }

    // Calculate metrics (u and s values)
    let (u, s) = calculate_metrics(anti, pro, false)?;

    // Create deposit record
    let deposit = UserDeposit {
        address: ctx.accounts.authority.key(),
        anti,
        pro,
        u,
        s,
        withdrawn: false,
    };

    // Update poll state
    poll.deposits.push(deposit);
    poll.anti = poll
        .anti
        .checked_add(anti)
        .ok_or(error!(PredictError::MathError))?;
    poll.pro = poll
        .pro
        .checked_add(pro)
        .ok_or(error!(PredictError::MathError))?;

    // Emit deposit event
    emit!(DepositEvent {
        poll_index,
        address: ctx.accounts.authority.key(),
        anti,
        pro,
        u,
        s,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DepositTokensBumps;
    use anchor_lang::prelude::AccountInfo;
    use anchor_lang::solana_program::system_program;
    use anchor_spl::token::TokenAccount;
    use anchor_spl::token::{spl_token, Token};
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

    struct TestAccounts {
        pub poll_data: TestAccountData,
        pub authority: TestAccountData,
        pub user_anti_token: TestAccountData,
        pub user_pro_token: TestAccountData,
        pub poll_anti_token: TestAccountData,
        pub poll_pro_token: TestAccountData,
        pub token_program: TestAccountData,
    }

    fn create_test_accounts(program_id: Pubkey) -> TestAccounts {
        TestAccounts {
            poll_data: TestAccountData::new_owned(program_id),
            authority: TestAccountData::new_owned(system_program::ID),
            user_anti_token: TestAccountData::new_owned(program_id),
            user_pro_token: TestAccountData::new_owned(program_id),
            poll_anti_token: TestAccountData::new_owned(program_id),
            poll_pro_token: TestAccountData::new_owned(program_id),
            token_program: TestAccountData::new_owned(spl_token::ID),
        }
    }

    #[test]
    fn test_deposit() {
        let program_id = program_id();
        let mut accounts = create_test_accounts(program_id);

        // Create PollAccount data
        let poll = PollAccount {
            index: 0,
            title: "Test Poll".to_string(),
            description: "Test Description".to_string(),
            start_time: "2025-01-20T00:00:00Z".to_string(),
            end_time: "2025-01-21T00:00:00Z".to_string(),
            etc: None,
            anti: 0,
            pro: 0,
            deposits: vec![],
            equalised: false,
            equalisation_results: None,
        };

        // Get account infos
        let poll_account_info = accounts.poll_data.to_account_info(false);
        let authority_info = accounts.authority.to_account_info(true);
        let user_anti_info = accounts.user_anti_token.to_account_info(false);
        let user_pro_info = accounts.user_pro_token.to_account_info(false);
        let poll_anti_info = accounts.poll_anti_token.to_account_info(false);
        let poll_pro_info = accounts.poll_pro_token.to_account_info(false);
        let token_program_info = accounts.token_program.to_account_info(false);

        // Serialize poll data
        poll_account_info
            .try_borrow_mut_data()
            .unwrap()
            .copy_from_slice(&poll.try_to_vec().unwrap());

        // Create deposit accounts
        let mut deposit_accounts = DepositTokens {
            poll: Account::try_from(&poll_account_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            user_anti_token: TestAccountData::into_token_account(&user_anti_info),
            user_pro_token: TestAccountData::into_token_account(&user_pro_info),
            poll_anti_token: TestAccountData::into_token_account(&poll_anti_info),
            poll_pro_token: TestAccountData::into_token_account(&poll_pro_info),
            token_program: Program::<Token>::try_from(&token_program_info).unwrap(),
        };

        let ctx = Context::new(
            &program_id,
            &mut deposit_accounts,
            &[],
            DepositTokensBumps { poll: 255 },
        );

        // Test deposit
        let result = deposit(ctx, 0, 5000, 5000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_deposit_validation() {
        let program_id = program_id();
        let mut accounts = create_test_accounts(program_id);

        // Create inactive poll
        let poll = PollAccount {
            index: 0,
            title: "Test Poll".to_string(),
            description: "Test Description".to_string(),
            start_time: "2024-01-01T00:00:00Z".to_string(), // Past date
            end_time: "2024-01-02T00:00:00Z".to_string(),   // Past date
            etc: None,
            anti: 0,
            pro: 0,
            deposits: vec![],
            equalised: false,
            equalisation_results: None,
        };

        // Get account infos
        let poll_account_info = accounts.poll_data.to_account_info(false);
        let authority_info = accounts.authority.to_account_info(true);
        let user_anti_info = accounts.user_anti_token.to_account_info(false);
        let user_pro_info = accounts.user_pro_token.to_account_info(false);
        let poll_anti_info = accounts.poll_anti_token.to_account_info(false);
        let poll_pro_info = accounts.poll_pro_token.to_account_info(false);
        let token_program_info = accounts.token_program.to_account_info(false);

        // Serialize poll data
        poll_account_info
            .try_borrow_mut_data()
            .unwrap()
            .copy_from_slice(&poll.try_to_vec().unwrap());

        // Test minimum deposit validation
        {
            let mut deposit_accounts = DepositTokens {
                poll: Account::try_from(&poll_account_info).unwrap(),
                authority: Signer::try_from(&authority_info).unwrap(),
                user_anti_token: TestAccountData::into_token_account(&user_anti_info),
                user_pro_token: TestAccountData::into_token_account(&user_pro_info),
                poll_anti_token: TestAccountData::into_token_account(&poll_anti_info),
                poll_pro_token: TestAccountData::into_token_account(&poll_pro_info),
                token_program: Program::<Token>::try_from(&token_program_info).unwrap(),
            };

            let ctx = Context::new(
                &program_id,
                &mut deposit_accounts,
                &[],
                DepositTokensBumps { poll: 255 },
            );

            let result = deposit(ctx, 0, 100, 100); // Below MIN_DEPOSIT
            match result {
                Err(err) => assert_eq!(err, PredictError::InsufficientDeposit.into()),
                _ => panic!("Expected insufficient deposit error"),
            }
        }

        // Test invalid token account ownership
        {
            // Create new accounts with invalid owner
            let mut invalid_accounts = create_test_accounts(program_id);
            invalid_accounts.user_anti_token.owner = Pubkey::new_unique(); // Set wrong owner

            // Get account infos from the new accounts
            let invalid_user_anti_info = invalid_accounts.user_anti_token.to_account_info(false);

            let mut deposit_accounts = DepositTokens {
                poll: Account::try_from(&poll_account_info).unwrap(),
                authority: Signer::try_from(&authority_info).unwrap(),
                user_anti_token: TestAccountData::into_token_account(&invalid_user_anti_info),
                user_pro_token: TestAccountData::into_token_account(&user_pro_info),
                poll_anti_token: TestAccountData::into_token_account(&poll_anti_info),
                poll_pro_token: TestAccountData::into_token_account(&poll_pro_info),
                token_program: Program::<Token>::try_from(&token_program_info).unwrap(),
            };

            let ctx = Context::new(
                &program_id,
                &mut deposit_accounts,
                &[],
                DepositTokensBumps { poll: 255 },
            );

            let result = deposit(ctx, 0, 5000, 5000);
            match result {
                Err(err) => assert_eq!(err, PredictError::InvalidTokenAccount.into()),
                _ => panic!("Expected invalid token account error"),
            }
        }
    }

    #[test]
    fn test_deposit_calculation() {
        let program_id = program_id();
        let mut accounts = create_test_accounts(program_id);

        // Create active poll
        let poll = PollAccount {
            index: 0,
            title: "Test Poll".to_string(),
            description: "Test Description".to_string(),
            start_time: "2025-01-20T00:00:00Z".to_string(),
            end_time: "2025-01-21T00:00:00Z".to_string(),
            etc: None,
            anti: 0,
            pro: 0,
            deposits: vec![],
            equalised: false,
            equalisation_results: None,
        };

        // Get account infos
        let poll_account_info = accounts.poll_data.to_account_info(false);
        let authority_info = accounts.authority.to_account_info(true);
        let user_anti_info = accounts.user_anti_token.to_account_info(false);
        let user_pro_info = accounts.user_pro_token.to_account_info(false);
        let poll_anti_info = accounts.poll_anti_token.to_account_info(false);
        let poll_pro_info = accounts.poll_pro_token.to_account_info(false);
        let token_program_info = accounts.token_program.to_account_info(false);

        // Serialize poll data
        poll_account_info
            .try_borrow_mut_data()
            .unwrap()
            .copy_from_slice(&poll.try_to_vec().unwrap());

        let anti = 7000;
        let pro = 3000;

        let mut deposit_accounts = DepositTokens {
            poll: Account::try_from(&poll_account_info).unwrap(),
            authority: Signer::try_from(&authority_info).unwrap(),
            user_anti_token: TestAccountData::into_token_account(&user_anti_info),
            user_pro_token: TestAccountData::into_token_account(&user_pro_info),
            poll_anti_token: TestAccountData::into_token_account(&poll_anti_info),
            poll_pro_token: TestAccountData::into_token_account(&poll_pro_info),
            token_program: Program::<Token>::try_from(&token_program_info).unwrap(),
        };

        let ctx = Context::new(
            &program_id,
            &mut deposit_accounts,
            &[],
            DepositTokensBumps { poll: 255 },
        );

        let result = deposit(ctx, 0, anti, pro);
        assert!(result.is_ok());

        // Verify deposit calculations
        let poll_account: PollAccount = PollAccount::try_deserialize(
            &mut poll_account_info.try_borrow_data().unwrap().as_ref(),
        )
        .unwrap();

        assert_eq!(poll_account.anti, anti);
        assert_eq!(poll_account.pro, pro);

        let deposit = &poll_account.deposits[0];
        let (expected_u, expected_s) = calculate_metrics(anti, pro, false).unwrap();

        assert_eq!(deposit.u, expected_u);
        assert_eq!(deposit.s, expected_s);
        assert_eq!(deposit.anti, anti);
        assert_eq!(deposit.pro, pro);
        assert!(!deposit.withdrawn);
        assert_eq!(deposit.address, authority_info.key());
    }
}

use crate::utils::*;
use crate::UserWithdrawTokens;
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Transfer};
use borsh::BorshSerialize;

pub fn user_withdraw<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, UserWithdrawTokens<'info>>,
    poll_index: u64,
) -> Result<()> {
    // Check token account authorities
    let anti_token_authority = ctx.accounts.poll_anti_token.owner;
    let pro_token_authority = ctx.accounts.poll_pro_token.owner;

    // Both token accounts must have the same authority
    require!(
        anti_token_authority == pro_token_authority,
        PredictError::InvalidTokenAccount
    );

    let current_authority = anti_token_authority;

    // If authority is still multisig, user withdrawals aren't enabled yet
    if current_authority == ANTITOKEN_MULTISIG {
        return err!(PredictError::UserWithdrawalsNotEnabled);
    }

    // Authority must be either multisig or state PDA
    let state_pda = ctx.accounts.state.key();
    require!(current_authority == state_pda, PredictError::Unauthorised);

    // Verify poll has been equalised
    require!(&ctx.accounts.poll.equalised, PredictError::NotEqualised);

    let equalisation = &ctx
        .accounts
        .poll
        .equalisation_results
        .clone()
        .ok_or(error!(PredictError::NotEqualised))?;

    // Get current deposit for the user
    let user_key = ctx.accounts.authority.key();
    let deposit_index = ctx
        .accounts
        .poll
        .deposits
        .iter()
        .position(|d| d.address == user_key)
        .ok_or(error!(PredictError::NoDeposit))?;

    let deposit = ctx.accounts.poll.deposits[deposit_index].clone();
    require!(!deposit.withdrawn, PredictError::AlreadyWithdrawn);

    // Get withdrawal amounts
    let anti_return = equalisation.anti[deposit_index];
    let pro_return = equalisation.pro[deposit_index];

    // Calculate and transfer payment (e.g., 0.001 SOL)
    let payment_amount = 1_000_000;
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.authority.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        ),
        payment_amount,
    )?;

    // Transfer ANTI tokens if any
    if anti_return > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.poll_anti_token.to_account_info(),
                    to: ctx.accounts.user_anti_token.to_account_info(),
                    authority: ctx.accounts.state.to_account_info(),
                },
                &[&[b"state", &[ctx.bumps.state]]],
            ),
            anti_return,
        )?;
    }

    // Transfer PRO tokens if any
    if pro_return > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.poll_pro_token.to_account_info(),
                    to: ctx.accounts.user_pro_token.to_account_info(),
                    authority: ctx.accounts.state.to_account_info(),
                },
                &[&[b"state", &[ctx.bumps.state]]],
            ),
            pro_return,
        )?;
    }

    // Mark deposit as withdrawn
    let poll = &mut ctx.accounts.poll;
    poll.deposits[deposit_index].withdrawn = true;

    // Serialise updated poll state
    let poll_info = poll.to_account_info();
    let mut poll_data = poll_info.try_borrow_mut_data()?;
    let serialised_poll = poll.try_to_vec()?;
    poll_data[8..8 + serialised_poll.len()].copy_from_slice(&serialised_poll);

    // Emit withdrawal event
    emit!(WithdrawEvent {
        poll_index,
        address: user_key,
        anti: anti_return,
        pro: pro_return,
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
    use crate::UserWithdrawTokensBumps;
    use anchor_lang::Discriminator;
    use anchor_spl::token::spl_token;
    use anchor_spl::token::Mint;
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
        fn new_account_with_key_and_owner<T: AccountSerialize + AccountDeserialize + Clone>(
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

        fn new_token_program() -> Self {
            Self {
                key: spl_token::ID,
                lamports: 1_000_000,
                data: vec![],
                owner: Pubkey::default(),
                executable: true,
                rent_epoch: 0,
            }
        }

        fn new_vault_with_key() -> Self {
            Self {
                key: ANTITOKEN_MULTISIG,
                lamports: 10_000_000,
                data: vec![],
                owner: system_program::ID,
                executable: false,
                rent_epoch: 0,
            }
        }

        fn new_system_account() -> Self {
            Self {
                key: system_program::ID,
                lamports: 1_000_000,
                data: vec![],
                owner: system_program::ID,
                executable: true,
                rent_epoch: 0,
            }
        }

        fn new_mint(mint: Pubkey) -> Self {
            Self {
                key: mint,
                lamports: 1_000_000,
                data: vec![0; Mint::LEN],
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

        // Reusable method to create an equalised test poll
        fn create_equalised_test_poll(authority: Pubkey) -> PollAccount {
            PollAccount {
                index: 0,
                title: "Test Poll".to_string(),
                description: "Test Description".to_string(),
                start_time: "2025-01-01T00:00:00Z".to_string(),
                end_time: "2025-01-02T00:00:00Z".to_string(), // Already ended
                etc: None,
                anti: 70000,
                pro: 30000,
                deposits: vec![UserDeposit {
                    address: authority,
                    anti: 70000,
                    pro: 30000,
                    u: 40000,
                    s: 100000,
                    withdrawn: false,
                }],
                equalised: true,
                equalisation_results: Some(EqualisationResult {
                    truth: vec![60000, 40000],
                    anti: vec![70000],
                    pro: vec![30000],
                    timestamp: 0,
                }),
            }
        }
    }

    #[test]
    fn test_user_withdrawal() {
        let program_id = program_id();
        let poll_index: u64 = 0;

        // Create mints
        let anti_mint = TestAccountData::new_mint(ANTI_MINT_ADDRESS);
        let pro_mint = TestAccountData::new_mint(PRO_MINT_ADDRESS);

        // Create test accounts
        let mut poll = TestAccountData::new_account_with_key_and_owner::<PollAccount>(
            Pubkey::new_unique(),
            program_id,
        );
        let mut state = TestAccountData::new_account_with_key_and_owner::<StateAccount>(
            Pubkey::new_unique(),
            program_id,
        );
        let mut user = TestAccountData::new_authority_account(Pubkey::new_unique());
        let mut vault = TestAccountData::new_vault_with_key();

        // Initialise token accounts
        let mut user_anti = TestAccountData::new_token();
        let mut user_pro = TestAccountData::new_token();
        let mut poll_anti = TestAccountData::new_token();
        let mut poll_pro = TestAccountData::new_token();

        user_anti
            .init_token_account(user.key, anti_mint.key)
            .unwrap();
        user_pro.init_token_account(user.key, pro_mint.key).unwrap();
        poll_anti
            .init_token_account(state.key, anti_mint.key)
            .unwrap(); // Note: state PDA is the authority
        poll_pro
            .init_token_account(state.key, pro_mint.key)
            .unwrap(); // Note: state PDA is the authority

        let mut token_program = TestAccountData::new_token_program();
        let mut system_program = TestAccountData::new_system_account();

        // Create state data
        let state_data = StateAccount {
            poll_index,
            authority: Pubkey::new_unique(),
        };
        state.init_state_data(&state_data).unwrap();

        // Create poll with user's deposit and equalisation results
        let poll_data = TestAccountData::create_equalised_test_poll(user.key);

        // Write discriminator and serialise poll data
        poll.data[..8].copy_from_slice(&PollAccount::discriminator());
        let serialised_poll = poll_data.try_to_vec().unwrap();
        poll.data[8..8 + serialised_poll.len()].copy_from_slice(&serialised_poll);

        // Get account infos
        let state_info = state.to_account_info(false);
        let poll_info = poll.to_account_info(false);
        let user_info = user.to_account_info(true);
        let user_anti_info = user_anti.to_account_info(false);
        let user_pro_info = user_pro.to_account_info(false);
        let poll_anti_info = poll_anti.to_account_info(false);
        let poll_pro_info = poll_pro.to_account_info(false);
        let token_program_info = token_program.to_account_info(false);
        let system_program_info = system_program.to_account_info(false);
        let vault_info = vault.to_account_info(false);

        // Derive PDAs and bumps
        let (_state_pda, state_bump) = Pubkey::find_program_address(&[b"state"], &program_id);
        let (_poll_pda, poll_bump) = Pubkey::find_program_address(
            &[b"poll", poll_index.to_le_bytes().as_ref()],
            &program_id,
        );
        let (_anti_token_pda, anti_token_bump) = Pubkey::find_program_address(
            &[b"anti_token", poll_data.index.to_le_bytes().as_ref()],
            &program_id,
        );
        let (_pro_token_pda, pro_token_bump) = Pubkey::find_program_address(
            &[b"pro_token", poll_data.index.to_le_bytes().as_ref()],
            &program_id,
        );

        let mut accounts = UserWithdrawTokens {
            state: Account::try_from(&state_info).unwrap(),
            poll: Account::try_from(&poll_info).unwrap(),
            authority: Signer::try_from(&user_info).unwrap(),
            user_anti_token: Account::try_from(&user_anti_info).unwrap(),
            user_pro_token: Account::try_from(&user_pro_info).unwrap(),
            poll_anti_token: Account::try_from(&poll_anti_info).unwrap(),
            poll_pro_token: Account::try_from(&poll_pro_info).unwrap(),
            token_program: Program::try_from(&token_program_info).unwrap(),
            system_program: Program::try_from(&system_program_info).unwrap(),
            vault: vault_info,
        };

        let bumps = UserWithdrawTokensBumps {
            state: state_bump,
            poll: poll_bump,
            poll_anti_token: anti_token_bump,
            poll_pro_token: pro_token_bump,
        };

        let _ = user_withdraw(Context::new(&program_id, &mut accounts, &[], bumps), 0);
    }
}

use crate::utils::*;
use crate::UserWithdrawTokens;
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Transfer};
use borsh::BorshSerialize;

pub fn user_withdraw<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, UserWithdrawTokens<'info>>,
    index: u64,
) -> Result<()> {
    // Check token account authorities
    let anti_token_authority = ctx.accounts.prediction_anti_token.owner;
    let pro_token_authority = ctx.accounts.prediction_pro_token.owner;

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

    // Verify prediction has been equalised
    require!(
        &ctx.accounts.prediction.equalised,
        PredictError::NotEqualised
    );

    let equalisation = &ctx
        .accounts
        .prediction
        .equalisation
        .clone()
        .ok_or(error!(PredictError::NotEqualised))?;

    // Get current deposit for the user
    let user_key = ctx.accounts.authority.key();
    let deposit_index = ctx
        .accounts
        .prediction
        .deposits
        .iter()
        .position(|d| d.address == user_key)
        .ok_or(error!(PredictError::NoDeposit))?;

    let deposit = ctx.accounts.prediction.deposits[deposit_index].clone();
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
                    from: ctx.accounts.prediction_anti_token.to_account_info(),
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
                    from: ctx.accounts.prediction_pro_token.to_account_info(),
                    to: ctx.accounts.user_pro_token.to_account_info(),
                    authority: ctx.accounts.state.to_account_info(),
                },
                &[&[b"state", &[ctx.bumps.state]]],
            ),
            pro_return,
        )?;
    }

    // Mark deposit as withdrawn
    let prediction = &mut ctx.accounts.prediction;
    prediction.deposits[deposit_index].withdrawn = true;

    // Serialise updated prediction state
    let prediction_info = prediction.to_account_info();
    let mut prediction_data = prediction_info.try_borrow_mut_data()?;
    let serialised_prediction = prediction.try_to_vec()?;
    prediction_data[8..8 + serialised_prediction.len()].copy_from_slice(&serialised_prediction);

    // Emit withdrawal event
    emit!(WithdrawEvent {
        index,
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
    use crate::Deposit;
    use crate::Equalisation;
    use crate::PredictionAccount;
    use crate::StateAccount;
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

        // Reusable method to create an equalised test prediction
        fn create_equalised_test_prediction(authority: Pubkey) -> PredictionAccount {
            PredictionAccount {
                index: 0,
                title: "Test Prediction".to_string(),
                description: "Test Description".to_string(),
                start_time: "2025-01-01T00:00:00Z".to_string(),
                end_time: "2025-01-02T00:00:00Z".to_string(), // Already ended
                etc: None,
                anti: 70000,
                pro: 30000,
                deposits: vec![Deposit {
                    address: authority,
                    anti: 70000,
                    pro: 30000,
                    mean: 40000,
                    stddev: 100000,
                    withdrawn: false,
                }],
                equalised: true,
                equalisation: Some(Equalisation {
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
        let index: u64 = 0;

        // Create mints
        let anti_mint = TestAccountData::new_mint(ANTI_MINT_ADDRESS);
        let pro_mint = TestAccountData::new_mint(PRO_MINT_ADDRESS);

        // Create test accounts
        let mut prediction = TestAccountData::new_account_with_key_and_owner::<PredictionAccount>(
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
        let mut prediction_anti = TestAccountData::new_token();
        let mut prediction_pro = TestAccountData::new_token();

        user_anti
            .init_token_account(user.key, anti_mint.key)
            .unwrap();
        user_pro.init_token_account(user.key, pro_mint.key).unwrap();
        prediction_anti
            .init_token_account(state.key, anti_mint.key)
            .unwrap(); // Note: state PDA is the authority
        prediction_pro
            .init_token_account(state.key, pro_mint.key)
            .unwrap(); // Note: state PDA is the authority

        let mut token_program = TestAccountData::new_token_program();
        let mut system_program = TestAccountData::new_system_account();

        // Create state data
        let state_data = StateAccount {
            index,
            authority: Pubkey::new_unique(),
        };
        state.init_state_data(&state_data).unwrap();

        // Create prediction with user's deposit and equalisation results
        let prediction_data = TestAccountData::create_equalised_test_prediction(user.key);

        // Write discriminator and serialise prediction data
        prediction.data[..8].copy_from_slice(&PredictionAccount::discriminator());
        let serialised_prediction = prediction_data.try_to_vec().unwrap();
        prediction.data[8..8 + serialised_prediction.len()].copy_from_slice(&serialised_prediction);

        // Get account infos
        let state_info = state.to_account_info(false);
        let prediction_info = prediction.to_account_info(false);
        let user_info = user.to_account_info(true);
        let user_anti_info = user_anti.to_account_info(false);
        let user_pro_info = user_pro.to_account_info(false);
        let prediction_anti_info = prediction_anti.to_account_info(false);
        let prediction_pro_info = prediction_pro.to_account_info(false);
        let token_program_info = token_program.to_account_info(false);
        let system_program_info = system_program.to_account_info(false);
        let vault_info = vault.to_account_info(false);

        // Derive PDAs and bumps
        let (_state_pda, state_bump) = Pubkey::find_program_address(&[b"state"], &program_id);
        let (_prediction_pda, prediction_bump) = Pubkey::find_program_address(
            &[b"prediction", index.to_le_bytes().as_ref()],
            &program_id,
        );
        let (_anti_token_pda, anti_token_bump) = Pubkey::find_program_address(
            &[b"anti_token", prediction_data.index.to_le_bytes().as_ref()],
            &program_id,
        );
        let (_pro_token_pda, pro_token_bump) = Pubkey::find_program_address(
            &[b"pro_token", prediction_data.index.to_le_bytes().as_ref()],
            &program_id,
        );

        let mut accounts = UserWithdrawTokens {
            state: Account::try_from(&state_info).unwrap(),
            prediction: Account::try_from(&prediction_info).unwrap(),
            authority: Signer::try_from(&user_info).unwrap(),
            user_anti_token: Account::try_from(&user_anti_info).unwrap(),
            user_pro_token: Account::try_from(&user_pro_info).unwrap(),
            prediction_anti_token: Account::try_from(&prediction_anti_info).unwrap(),
            prediction_pro_token: Account::try_from(&prediction_pro_info).unwrap(),
            token_program: Program::try_from(&token_program_info).unwrap(),
            system_program: Program::try_from(&system_program_info).unwrap(),
            vault: vault_info,
        };

        let bumps = UserWithdrawTokensBumps {
            state: state_bump,
            prediction: prediction_bump,
            prediction_anti_token: anti_token_bump,
            prediction_pro_token: pro_token_bump,
        };

        let _ = user_withdraw(Context::new(&program_id, &mut accounts, &[], bumps), 0);
    }
}

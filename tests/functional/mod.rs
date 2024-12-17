impl TestContext {
    // Add these methods to the existing impl block

    pub async fn initialize_collider(
        &mut self,
        baryon_mint: &Pubkey,
        photon_mint: &Pubkey,
        vault_anti: &Pubkey,
        vault_pro: &Pubkey,
    ) -> Result<Pubkey, BanksClientError> {
        use crate::instruction::CollisionInstruction;
        use borsh::BorshSerialize;

        let state = self
            .create_collision_state(baryon_mint, photon_mint, vault_anti, vault_pro)
            .await?;

        let init_ix = solana_program::instruction::Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(state, false),
                AccountMeta::new(*baryon_mint, false),
                AccountMeta::new(*photon_mint, false),
                AccountMeta::new(*vault_anti, false),
                AccountMeta::new(*vault_pro, false),
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(spl_token_2022::id(), false),
                AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
            ],
            data: CollisionInstruction::Initialise.try_to_vec().unwrap(),
        };

        let transaction = Transaction::new_signed_with_payer(
            &[init_ix],
            Some(&self.payer.pubkey()),
            &[&self.payer],
            self.last_blockhash,
        );

        self.banks_client.process_transaction(transaction).await?;

        Ok(state)
    }

    pub async fn collide(
        &mut self,
        state: &Pubkey,
        anti_token_account: &Pubkey,
        pro_token_account: &Pubkey,
        baryon_token_account: &Pubkey,
        photon_token_account: &Pubkey,
        baryon_mint: &Pubkey,
        photon_mint: &Pubkey,
        vault_anti: &Pubkey,
        vault_pro: &Pubkey,
        anti_mint: &Pubkey,
        pro_mint: &Pubkey,
        anti_amount: u64,
        pro_amount: u64,
    ) -> Result<(), BanksClientError> {
        use crate::instruction::CollisionInstruction;
        use borsh::BorshSerialize;

        let collide_ix = solana_program::instruction::Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(*state, false),
                AccountMeta::new(*anti_token_account, false),
                AccountMeta::new(*pro_token_account, false),
                AccountMeta::new(*baryon_token_account, false),
                AccountMeta::new(*photon_token_account, false),
                AccountMeta::new(*baryon_mint, false),
                AccountMeta::new(*photon_mint, false),
                AccountMeta::new(*vault_anti, false),
                AccountMeta::new(*vault_pro, false),
                AccountMeta::new_readonly(*anti_mint, false),
                AccountMeta::new_readonly(*pro_mint, false),
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(spl_token_2022::id(), false),
                AccountMeta::new_readonly(self.authority, false),
            ],
            data: CollisionInstruction::Collide {
                anti_amount,
                pro_amount,
            }
            .try_to_vec()
            .unwrap(),
        };

        let transaction = Transaction::new_signed_with_payer(
            &[collide_ix],
            Some(&self.payer.pubkey()),
            &[&self.payer],
            self.last_blockhash,
        );

        self.banks_client.process_transaction(transaction).await
    }

    pub async fn setup_collision_test(
        &mut self,
    ) -> Result<
        (
            Pubkey, // state
            Pubkey, // anti_mint
            Pubkey, // pro_mint
            Pubkey, // baryon_mint
            Pubkey, // photon_mint
            Pubkey, // anti_account
            Pubkey, // pro_account
            Pubkey, // baryon_account
            Pubkey, // photon_account
            Pubkey, // vault_anti
            Pubkey, // vault_pro
        ),
        BanksClientError,
    > {
        // Create all mints
        let anti_mint = self.create_token_mint(None).await?;
        let pro_mint = self.create_token_mint(None).await?;
        let baryon_mint = self.create_token_mint(Some(&self.authority)).await?;
        let photon_mint = self.create_token_mint(Some(&self.authority)).await?;

        // Create vault accounts
        let vault_anti = self.create_vault_account(&anti_mint).await?;
        let vault_pro = self.create_vault_account(&pro_mint).await?;

        // Initialize collider
        let state = self
            .initialize_collider(&baryon_mint, &photon_mint, &vault_anti, &vault_pro)
            .await?;

        // Create user token accounts
        let anti_account = self
            .create_token_account(&anti_mint, &self.payer.pubkey())
            .await?;
        let pro_account = self
            .create_token_account(&pro_mint, &self.payer.pubkey())
            .await?;
        let baryon_account = self
            .create_token_account(&baryon_mint, &self.payer.pubkey())
            .await?;
        let photon_account = self
            .create_token_account(&photon_mint, &self.payer.pubkey())
            .await?;

        Ok((
            state,
            anti_mint,
            pro_mint,
            baryon_mint,
            photon_mint,
            anti_account,
            pro_account,
            baryon_account,
            photon_account,
            vault_anti,
            vault_pro,
        ))
    }
}

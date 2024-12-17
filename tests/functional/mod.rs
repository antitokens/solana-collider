use super::common::TestContext;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    system_program,
};
use solana_sdk::{
    transport::TransportError,
    signature::{Keypair, Signer},
    rent::Rent,
};

mod test_initialisation {
    use super::*;

    #[tokio::test]
    async fn test_initialise_success() {
        let mut context = TestContext::new().await;
        
        // Create BARYON and PHOTON token mints
        let baryon_mint = context.create_token_mint().await.unwrap();
        let photon_mint = context.create_token_mint().await.unwrap();
        
        // Test initialisation
        let result = initialise_collision(&mut context, &baryon_mint, &photon_mint).await;
        assert!(result.is_ok());

        // Verify the program state
        let state_data = context.get_state_account().await.unwrap();
        assert_eq!(state_data.baryon_mint, baryon_mint);
        assert_eq!(state_data.photon_mint, photon_mint);
        assert_eq!(state_data.authority, context.payer.pubkey());
    }

    #[tokio::test]
    async fn test_initialise_already_initialised() {
        let mut context = TestContext::new().await;
        
        // Create mints
        let baryon_mint = context.create_token_mint().await.unwrap();
        let photon_mint = context.create_token_mint().await.unwrap();
        
        // Initialise once
        initialise_collision(&mut context, &baryon_mint, &photon_mint).await.unwrap();
        
        // Try to initialise again
        let result = initialise_collision(&mut context, &baryon_mint, &photon_mint).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_initialise_invalid_mint() {
        let mut context = TestContext::new().await;
        
        // Create invalid mint (not initialised)
        let invalid_mint = Keypair::new().pubkey();
        let photon_mint = context.create_token_mint().await.unwrap();
        
        let result = initialise_collision(&mut context, &invalid_mint, &photon_mint).await;
        assert!(result.is_err());
    }
}

mod test_collision {
    use super::*;

    #[tokio::test]
    async fn test_collide_success() {
        let mut context = TestContext::new().await;
        
        // Setup complete test environment
        let (
            user,
            anti_mint,
            pro_mint,
            baryon_mint,
            photon_mint,
            anti_account,
            pro_account,
            baryon_account,
            photon_account,
        ) = context.setup_collision_test().await.unwrap();
        
        // Initialise program
        initialise_collision(&mut context, &baryon_mint, &photon_mint).await.unwrap();
        
        // Mint initial tokens
        context.mint_tokens(&anti_mint, &anti_account, 100).await.unwrap();
        context.mint_tokens(&pro_mint, &pro_account, 100).await.unwrap();
        
        // Perform collision
        let result = perform_collision(
            &mut context,
            &user,
            &anti_account,
            &pro_account,
            &baryon_account,
            &photon_account,
            100,
            100,
        ).await;
        
        assert!(result.is_ok());
        
        // Verify results
        let baryon_balance = context.get_token_balance(&baryon_account).await.unwrap();
        let photon_balance = context.get_token_balance(&photon_account).await.unwrap();
        let anti_balance = context.get_token_balance(&anti_account).await.unwrap();
        let pro_balance = context.get_token_balance(&pro_account).await.unwrap();
        
        assert_eq!(anti_balance, 0);
        assert_eq!(pro_balance, 0);
        assert_eq!(baryon_balance, 100); // Expected based on calculation
        assert_eq!(photon_balance, 100); // Expected based on calculation
    }

    #[tokio::test]
    async fn test_collide_uneven_amounts() {
        let mut context = TestContext::new().await;
        
        let (
            user,
            anti_mint,
            pro_mint,
            baryon_mint,
            photon_mint,
            anti_account,
            pro_account,
            baryon_account,
            photon_account,
        ) = context.setup_collision_test().await.unwrap();
        
        initialise_collision(&mut context, &baryon_mint, &photon_mint).await.unwrap();
        
        // Mint uneven amounts
        context.mint_tokens(&anti_mint, &anti_account, 150).await.unwrap();
        context.mint_tokens(&pro_mint, &pro_account, 50).await.unwrap();
        
        let result = perform_collision(
            &mut context,
            &user,
            &anti_account,
            &pro_account,
            &baryon_account,
            &photon_account,
            150,
            50,
        ).await;
        
        assert!(result.is_ok());
        
        // Verify results reflect uneven distribution
        let baryon_balance = context.get_token_balance(&baryon_account).await.unwrap();
        let photon_balance = context.get_token_balance(&photon_account).await.unwrap();
        
        assert!(baryon_balance > 0);
        assert!(photon_balance > 0);
        assert!(baryon_balance != photon_balance); // Uneven amounts should produce different results
    }

    #[tokio::test]
    async fn test_collide_zero_amounts() {
        let mut context = TestContext::new().await;
        
        let (
            user,
            _,
            _,
            baryon_mint,
            photon_mint,
            anti_account,
            pro_account,
            baryon_account,
            photon_account,
        ) = context.setup_collision_test().await.unwrap();
        
        initialise_collision(&mut context, &baryon_mint, &photon_mint).await.unwrap();
        
        // Try to collide with zero amounts
        let result = perform_collision(
            &mut context,
            &user,
            &anti_account,
            &pro_account,
            &baryon_account,
            &photon_account,
            0,
            0,
        ).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_collide_insufficient_balance() {
        let mut context = TestContext::new().await;
        
        let (
            user,
            anti_mint,
            pro_mint,
            baryon_mint,
            photon_mint,
            anti_account,
            pro_account,
            baryon_account,
            photon_account,
        ) = context.setup_collision_test().await.unwrap();
        
        initialise_collision(&mut context, &baryon_mint, &photon_mint).await.unwrap();
        
        // Mint less than we'll try to collide
        context.mint_tokens(&anti_mint, &anti_account, 50).await.unwrap();
        context.mint_tokens(&pro_mint, &pro_account, 50).await.unwrap();
        
        // Try to collide more than available
        let result = perform_collision(
            &mut context,
            &user,
            &anti_account,
            &pro_account,
            &baryon_account,
            &photon_account,
            100,
            100,
        ).await;
        
        assert!(result.is_err());
        
        // Verify balances remained unchanged
        assert_eq!(context.get_token_balance(&anti_account).await.unwrap(), 50);
        assert_eq!(context.get_token_balance(&pro_account).await.unwrap(), 50);
    }

    #[tokio::test]
    async fn test_collide_wrong_owner() {
        let mut context = TestContext::new().await;
        
        let (
            _,
            anti_mint,
            pro_mint,
            baryon_mint,
            photon_mint,
            anti_account,
            pro_account,
            baryon_account,
            photon_account,
        ) = context.setup_collision_test().await.unwrap();
        
        initialise_collision(&mut context, &baryon_mint, &photon_mint).await.unwrap();
        
        // Create a different user
        let wrong_user = Keypair::new();
        
        // Try to collide with wrong user
        let result = perform_collision(
            &mut context,
            &wrong_user,
            &anti_account,
            &pro_account,
            &baryon_account,
            &photon_account,
            100,
            100,
        ).await;
        
        assert!(result.is_err());
    }
}
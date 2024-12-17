#[cfg(test)]
mod tests {
    use super::*;
    use test::PROGRAM_ID;
    use solana_program::{
        account_info::AccountInfo,
        program_pack::Pack,
        sysvar::{clock::Clock, rent::Rent, Sysvar},
    };
    use solana_program_test::*;
    use solana_sdk::{account::Account, signature::Signer, transaction::Transaction};

    #[tokio::test]
    async fn test_collide() {
        // Use the test program ID
        let program_test = ProgramTest::new(
            "solana_collider",
            PROGRAM_ID,
            processor!(process_instruction),
        );

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        // Your test code here
        // You can use PROGRAM_ID whenever you need the program ID
    }

    #[test]
    fn test_error_conditions() {
        // For non-async tests, you can also use PROGRAM_ID
        let program_id = PROGRAM_ID;
        // Test code here
    }
}
//! Program Description: Integration tests for the Collider programme
//! Version: 0.0.1
//! License: MIT
//! Created: 20 Jan 2025
//! Last Modified: 20 Jan 2025
//! Repository: https://github.com/antitokens/solana-collider
//! Contact: dev@antitoken.pro

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::solana_program::clock::Clock;

    #[test]
    fn test_calculate_metrics() {
        // Test flag = false case
        let (u, s) = calculate_metrics(5000, 3000, false).unwrap();
        assert_eq!(u, 2000); // |5000 - 3000|
        assert_eq!(s, 8000); // 5000 + 3000
        
        // Test edge cases
        let (u, s) = calculate_metrics(0, 0, false).unwrap();
        assert_eq!(u, 0);
        assert_eq!(s, 0);
    }

    #[test]
    fn test_calculate_equalisation() {
        let deposits = vec![
            UserDeposit {
                user: Pubkey::new_unique(),
                anti_amount: 1000,
                pro_amount: 2000,
                u_value: 1000,
                s_value: 3000,
                withdrawn: false,
            },
            UserDeposit {
                user: Pubkey::new_unique(),
                anti_amount: 3000,
                pro_amount: 1000,
                u_value: 2000,
                s_value: 4000,
                withdrawn: false,
            },
        ];

        let truth = vec![6000, 4000]; // 60% vs 40%
        let (anti_returns, pro_returns) = calculate_equalisation(
            &deposits,
            4000, // total anti
            3000, // total pro
            &truth,
        ).unwrap();

        // Verify returns sum up to totals
        assert_eq!(anti_returns.iter().sum::<u64>(), 4000);
        assert_eq!(pro_returns.iter().sum::<u64>(), 3000);
    }

    #[test]
    fn test_parse_iso_timestamp() {
        // Test valid timestamp
        let result = parse_iso_timestamp("2025-01-20T00:00:00Z").unwrap();
        assert!(result > 0);

        // Test invalid formats
        assert!(parse_iso_timestamp("2025-01-20").is_err());
        assert!(parse_iso_timestamp("2025-13-20T00:00:00Z").is_err());
    }

    #[test]
    #[should_panic(expected = "InvalidTimeRange")]
    fn test_invalid_time_range() {
        let mut lamports = 10000;
        let mut data = vec![0; 1000];
        let owner = Pubkey::new_unique();
        
        let mut poll_account = AccountInfo::new(
            &owner,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );

        let ctx = Context::new(
            &crate::ID,
            &mut PollAccount {
                index: 0,
                title: "Test".to_string(),
                description: "Test".to_string(),
                start_time: "2025-01-20T00:00:00Z".to_string(),
                end_time: "2025-01-19T00:00:00Z".to_string(), // End before start
                etc: None,
                total_anti: 0,
                total_pro: 0,
                deposits: vec![],
            },
            &[&mut poll_account],
            &[],
            CpiContext::new(poll_account, ()),
        );

        create_poll(ctx, "Test".to_string(), "Test".to_string(),
            "2025-01-20T00:00:00Z".to_string(),
            "2025-01-19T00:00:00Z".to_string(),
            None).unwrap();
    }
}

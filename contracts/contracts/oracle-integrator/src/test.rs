#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env, Map};

#[test]
fn test_contract_initialization() {
    let env = Env::default();
    let contract_id = env.register(OracleIntegrator, ());
    let client = OracleIntegratorClient::new(&env, &contract_id);
    let config_manager = Address::generate(&env);

    client.initialize(&config_manager);
}

#[test]
fn test_test_mode_enables_simulation() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup
    let contract_id = env.register(OracleIntegrator, ());
    let client = OracleIntegratorClient::new(&env, &contract_id);
    let config_manager = Address::generate(&env);
    let admin = Address::generate(&env);

    client.initialize(&config_manager);

    // Enable test mode with base prices
    let mut base_prices = Map::new(&env);
    base_prices.set(0, 1_200_000); // XLM: $0.12
    base_prices.set(1, 600_000_000_000); // BTC: $60,000
    base_prices.set(2, 30_000_000_000); // ETH: $3,000

    client.set_test_mode(&admin, &true, &base_prices);

    // Verify test mode is enabled
    assert_eq!(client.get_test_mode(), true);

    // Verify prices are simulated (not fixed 100_000_000)
    let price_xlm = client.get_price(&0);
    // Price should be near base price (±10%)
    assert!(price_xlm > 1_080_000 && price_xlm < 1_320_000);
}

#[test]
fn test_simulated_prices_vary_over_time() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| li.timestamp = 1000);

    let contract_id = env.register(OracleIntegrator, ());
    let client = OracleIntegratorClient::new(&env, &contract_id);
    let config_manager = Address::generate(&env);
    let admin = Address::generate(&env);

    client.initialize(&config_manager);

    let mut base_prices = Map::new(&env);
    base_prices.set(0, 100_000_000); // $1.00
    client.set_test_mode(&admin, &true, &base_prices);

    let price_t1 = client.get_price(&0);

    // Advance time by 1800 seconds (30 minutes)
    env.ledger().with_mut(|li| li.timestamp = 2800);
    let price_t2 = client.get_price(&0);

    // Prices should differ
    assert_ne!(price_t1, price_t2);
}

#[test]
fn test_median_calculation() {
    let env = Env::default();
    let contract_id = env.register(OracleIntegrator, ());
    let client = OracleIntegratorClient::new(&env, &contract_id);
    let config_manager = Address::generate(&env);

    client.initialize(&config_manager);

    // Test median of two prices
    let median = client.calculate_median(&100_000_000, &110_000_000);
    assert_eq!(median, 105_000_000);
}

#[test]
fn test_multiple_markets_independent_prices() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(OracleIntegrator, ());
    let client = OracleIntegratorClient::new(&env, &contract_id);
    let config_manager = Address::generate(&env);
    let admin = Address::generate(&env);

    client.initialize(&config_manager);

    // Set different base prices
    let mut base_prices = Map::new(&env);
    base_prices.set(0, 1_000_000); // XLM: $0.10
    base_prices.set(1, 500_000_000_000); // BTC: $50,000
    base_prices.set(2, 30_000_000_000); // ETH: $3,000

    client.set_test_mode(&admin, &true, &base_prices);

    let price_xlm = client.get_price(&0);
    let price_btc = client.get_price(&1);
    let price_eth = client.get_price(&2);

    // All prices should be different and near their base prices
    assert!(price_xlm < 2_000_000); // < $0.20
    assert!(price_btc > 400_000_000_000); // > $40,000
    assert!(price_eth > 20_000_000_000); // > $2,000
}

#[test]
fn test_test_mode_disabled_by_default() {
    let env = Env::default();
    let contract_id = env.register(OracleIntegrator, ());
    let client = OracleIntegratorClient::new(&env, &contract_id);
    let config_manager = Address::generate(&env);

    client.initialize(&config_manager);

    // Test mode should be disabled by default
    assert_eq!(client.get_test_mode(), false);
}

#[test]
fn test_price_oscillation_pattern() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(OracleIntegrator, ());
    let client = OracleIntegratorClient::new(&env, &contract_id);
    let config_manager = Address::generate(&env);
    let admin = Address::generate(&env);

    client.initialize(&config_manager);

    let mut base_prices = Map::new(&env);
    base_prices.set(0, 100_000_000); // $1.00
    client.set_test_mode(&admin, &true, &base_prices);

    // Test at different times to verify oscillation
    env.ledger().with_mut(|li| li.timestamp = 0);
    let price_0 = client.get_price(&0);

    env.ledger().with_mut(|li| li.timestamp = 900); // 15 minutes
    let price_900 = client.get_price(&0);

    env.ledger().with_mut(|li| li.timestamp = 1800); // 30 minutes (peak of first oscillation)
    let price_1800 = client.get_price(&0);

    env.ledger().with_mut(|li| li.timestamp = 2700); // 45 minutes (descending)
    let price_2700 = client.get_price(&0);

    env.ledger().with_mut(|li| li.timestamp = 3600); // 60 minutes (back to base, new cycle)
    let price_3600 = client.get_price(&0);

    // Verify oscillation pattern
    // Oscillation: goes UP first (0-1800s), then DOWN (1800-3600s)
    assert_eq!(price_0, 100_000_000); // At start, base price
    assert!(price_900 > price_0); // Increasing in first period
    assert_eq!(price_1800, 95_000_000); // At 1800s, flips direction (-5%)
    assert!(price_2700 < price_0); // Continuing down
    assert_eq!(price_3600, 100_000_000); // Back to base at 1 hour (new cycle)
}

#[test]
fn test_median_with_equal_prices() {
    let env = Env::default();
    let contract_id = env.register(OracleIntegrator, ());
    let client = OracleIntegratorClient::new(&env, &contract_id);
    let config_manager = Address::generate(&env);

    client.initialize(&config_manager);

    let median = client.calculate_median(&100_000_000, &100_000_000);
    assert_eq!(median, 100_000_000);
}

#[test]
fn test_median_with_large_difference() {
    let env = Env::default();
    let contract_id = env.register(OracleIntegrator, ());
    let client = OracleIntegratorClient::new(&env, &contract_id);
    let config_manager = Address::generate(&env);

    client.initialize(&config_manager);

    let median = client.calculate_median(&50_000_000, &150_000_000);
    assert_eq!(median, 100_000_000);
}

mod fuzz {
    use super::*;
    use proptest::prelude::*;

    fn setup_oracle(env: &Env, base_price: i128) -> OracleIntegratorClient {
        env.mock_all_auths();
        let contract_id = env.register(OracleIntegrator, ());
        let client = OracleIntegratorClient::new(env, &contract_id);
        let config_manager = Address::generate(env);
        let admin = Address::generate(env);

        client.initialize(&config_manager);

        let mut base_prices = Map::new(env);
        base_prices.set(0, base_price);
        client.set_test_mode(&admin, &true, &base_prices);

        client
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        #[test]
        fn fuzz_price_always_positive(
            base_price in 1_000i128..1_000_000_000_000i128,
            timestamp in 0u64..100_000u64,
        ) {
            let env = Env::default();
            env.ledger().with_mut(|li| li.timestamp = timestamp);
            let client = setup_oracle(&env, base_price);

            let price = client.get_price(&0);
            prop_assert!(price > 0, "Price must be positive, got {}", price);
        }

        #[test]
        fn fuzz_price_within_bounds(
            base_price in 10_000i128..1_000_000_000_000i128,
            timestamp in 0u64..100_000u64,
        ) {
            let env = Env::default();
            env.ledger().with_mut(|li| li.timestamp = timestamp);
            let client = setup_oracle(&env, base_price);

            let price = client.get_price(&0);
            let lower = base_price * 90 / 100;
            let upper = base_price * 110 / 100;
            prop_assert!(
                price >= lower && price <= upper,
                "Price {} outside ±10% of base {} (range {}-{})",
                price, base_price, lower, upper
            );
        }

        #[test]
        fn fuzz_fixed_price_determinism(
            base_price in 1_000i128..1_000_000_000_000i128,
            timestamp in 0u64..100_000u64,
        ) {
            let env = Env::default();
            env.mock_all_auths();
            env.ledger().with_mut(|li| li.timestamp = timestamp);

            let contract_id = env.register(OracleIntegrator, ());
            let client = OracleIntegratorClient::new(&env, &contract_id);
            let config_manager = Address::generate(&env);
            let admin = Address::generate(&env);

            client.initialize(&config_manager);

            let mut base_prices = Map::new(&env);
            base_prices.set(0, base_price);
            client.set_test_mode(&admin, &true, &base_prices);
            client.set_fixed_price_mode(&admin, &true);

            let price = client.get_price(&0);
            prop_assert_eq!(price, base_price, "Fixed price mode should return exact base price");
        }

        #[test]
        fn fuzz_median_commutative(
            a in 1i128..1_000_000_000_000i128,
            b in 1i128..1_000_000_000_000i128,
        ) {
            let env = Env::default();
            let contract_id = env.register(OracleIntegrator, ());
            let client = OracleIntegratorClient::new(&env, &contract_id);
            let config_manager = Address::generate(&env);
            client.initialize(&config_manager);

            let median_ab = client.calculate_median(&a, &b);
            let median_ba = client.calculate_median(&b, &a);
            prop_assert_eq!(median_ab, median_ba, "Median must be commutative");
        }

        #[test]
        fn fuzz_median_bounded(
            a in 1i128..1_000_000_000_000i128,
            b in 1i128..1_000_000_000_000i128,
        ) {
            let env = Env::default();
            let contract_id = env.register(OracleIntegrator, ());
            let client = OracleIntegratorClient::new(&env, &contract_id);
            let config_manager = Address::generate(&env);
            client.initialize(&config_manager);

            let median = client.calculate_median(&a, &b);
            let min = a.min(b);
            let max = a.max(b);
            prop_assert!(
                median >= min && median <= max,
                "Median {} must be between {} and {}",
                median, min, max
            );
        }
    }
}

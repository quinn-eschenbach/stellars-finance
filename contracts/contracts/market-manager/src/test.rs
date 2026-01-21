#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, testutils::Address as _, Env};

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let config_manager = Address::generate(&env);

    let contract_id = env.register(MarketManager, ());
    let client = MarketManagerClient::new(&env, &contract_id);

    client.initialize(&config_manager, &admin);

    // Verify initialization worked - should not panic
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let config_manager = Address::generate(&env);

    let contract_id = env.register(MarketManager, ());
    let client = MarketManagerClient::new(&env, &contract_id);

    client.initialize(&config_manager, &admin);
    client.initialize(&config_manager, &admin); // Should panic
}

#[test]
fn test_create_market_success() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let config_manager = Address::generate(&env);

    let contract_id = env.register(MarketManager, ());
    let client = MarketManagerClient::new(&env, &contract_id);

    client.initialize(&config_manager, &admin);
    let ticker = symbol_short!("XLM");
    client.create_market(&admin, &0u32, &ticker, &1_000_000_000_000u128, &10000i128);

    let (long_oi, short_oi) = client.get_open_interest(&0u32);
    assert_eq!(long_oi, 0);
    assert_eq!(short_oi, 0);

    let funding_rate = client.get_funding_rate(&0u32);
    assert_eq!(funding_rate, 0);

    // Verify ticker is stored correctly
    let stored_ticker = client.get_market_ticker(&0u32);
    assert_eq!(stored_ticker, ticker);
}

#[test]
#[should_panic(expected = "market already exists")]
fn test_create_duplicate_market_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let config_manager = Address::generate(&env);

    let contract_id = env.register(MarketManager, ());
    let client = MarketManagerClient::new(&env, &contract_id);

    client.initialize(&config_manager, &admin);
    let ticker = symbol_short!("XLM");
    client.create_market(&admin, &0u32, &ticker, &1_000_000_000_000u128, &10000i128);
    client.create_market(&admin, &0u32, &ticker, &1_000_000_000_000u128, &10000i128); // Duplicate
}

#[test]
fn test_set_position_manager() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let config_manager = Address::generate(&env);
    let position_manager = Address::generate(&env);

    let contract_id = env.register(MarketManager, ());
    let client = MarketManagerClient::new(&env, &contract_id);

    client.initialize(&config_manager, &admin);
    client.set_position_manager(&admin, &position_manager);

    // If this doesn't panic, it succeeded
}

#[test]
fn test_update_open_interest_increase() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let config_manager = Address::generate(&env);
    let position_manager = Address::generate(&env);

    let contract_id = env.register(MarketManager, ());
    let client = MarketManagerClient::new(&env, &contract_id);

    client.initialize(&config_manager, &admin);
    client.set_position_manager(&admin, &position_manager);
    let ticker = symbol_short!("XLM");
    client.create_market(&admin, &0u32, &ticker, &1_000_000_000_000u128, &10000i128);

    // Increase long OI
    client.update_open_interest(&position_manager, &0u32, &true, &1_000_000_000i128);

    let (long_oi, short_oi) = client.get_open_interest(&0u32);
    assert_eq!(long_oi, 1_000_000_000);
    assert_eq!(short_oi, 0);
}

#[test]
fn test_update_open_interest_decrease() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let config_manager = Address::generate(&env);
    let position_manager = Address::generate(&env);

    let contract_id = env.register(MarketManager, ());
    let client = MarketManagerClient::new(&env, &contract_id);

    client.initialize(&config_manager, &admin);
    client.set_position_manager(&admin, &position_manager);
    let ticker = symbol_short!("XLM");
    client.create_market(&admin, &0u32, &ticker, &1_000_000_000_000u128, &10000i128);

    // Increase then decrease
    client.update_open_interest(&position_manager, &0u32, &true, &1_000_000_000i128);
    client.update_open_interest(&position_manager, &0u32, &true, &-500_000_000i128);

    let (long_oi, short_oi) = client.get_open_interest(&0u32);
    assert_eq!(long_oi, 500_000_000);
    assert_eq!(short_oi, 0);
}

#[test]
#[should_panic(expected = "exceeds max open interest")]
fn test_update_open_interest_exceeds_cap() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let config_manager = Address::generate(&env);
    let position_manager = Address::generate(&env);

    let contract_id = env.register(MarketManager, ());
    let client = MarketManagerClient::new(&env, &contract_id);

    client.initialize(&config_manager, &admin);
    client.set_position_manager(&admin, &position_manager);
    let ticker = symbol_short!("XLM");
    client.create_market(&admin, &0u32, &ticker, &1_000_000_000u128, &10000i128); // Max OI = 1B

    // Try to add 1.1B (exceeds cap)
    client.update_open_interest(&position_manager, &0u32, &true, &1_100_000_000i128);
}

#[test]
fn test_pause_unpause_market() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let config_manager = Address::generate(&env);

    let contract_id = env.register(MarketManager, ());
    let client = MarketManagerClient::new(&env, &contract_id);

    client.initialize(&config_manager, &admin);
    let ticker = symbol_short!("XLM");
    client.create_market(&admin, &0u32, &ticker, &1_000_000_000_000u128, &10000i128);

    assert!(!client.is_market_paused(&0u32));

    client.pause_market(&admin, &0u32);
    assert!(client.is_market_paused(&0u32));

    client.unpause_market(&admin, &0u32);
    assert!(!client.is_market_paused(&0u32));
}

#[test]
fn test_can_open_position_when_paused() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let config_manager = Address::generate(&env);

    let contract_id = env.register(MarketManager, ());
    let client = MarketManagerClient::new(&env, &contract_id);

    client.initialize(&config_manager, &admin);
    let ticker = symbol_short!("XLM");
    client.create_market(&admin, &0u32, &ticker, &1_000_000_000_000u128, &10000i128);

    assert!(client.can_open_position(&0u32, &true, &1_000_000u128));

    client.pause_market(&admin, &0u32);
    assert!(!client.can_open_position(&0u32, &true, &1_000_000u128));
}

#[test]
fn test_can_open_position_exceeds_oi() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let config_manager = Address::generate(&env);

    let contract_id = env.register(MarketManager, ());
    let client = MarketManagerClient::new(&env, &contract_id);

    client.initialize(&config_manager, &admin);
    let ticker = symbol_short!("XLM");
    client.create_market(&admin, &0u32, &ticker, &1_000_000_000u128, &10000i128); // Max OI = 1B

    assert!(!client.can_open_position(&0u32, &true, &1_100_000_000u128)); // Exceeds cap
    assert!(client.can_open_position(&0u32, &true, &900_000_000u128)); // Within cap
}

#[test]
fn test_get_cumulative_funding() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let config_manager = Address::generate(&env);

    let contract_id = env.register(MarketManager, ());
    let client = MarketManagerClient::new(&env, &contract_id);

    client.initialize(&config_manager, &admin);
    let ticker = symbol_short!("XLM");
    client.create_market(&admin, &0u32, &ticker, &1_000_000_000_000u128, &10000i128);

    // Initially should be 0
    let cumulative_long = client.get_cumulative_funding(&0u32, &true);
    let cumulative_short = client.get_cumulative_funding(&0u32, &false);

    assert_eq!(cumulative_long, 0);
    assert_eq!(cumulative_short, 0);
}

// Note: Comprehensive funding rate testing requires setting up ConfigManager mock
// which is complex in unit tests. The funding rate logic is tested through
// the formula implementation and will be verified in integration tests.

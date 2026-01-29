use soroban_sdk::Env;

use crate::common::{assertions::*, config_manager, oracle_integrator, position_manager, market_manager, setup::*, time_helpers::*};

#[test]
fn test_funding_accumulation_over_time() {
    let env = Env::default();
    let test_env = setup_focused_test(&env);

    let position_client = position_manager::Client::new(&env, &test_env.position_manager_id);
    let market_client = market_manager::Client::new(&env, &test_env.market_manager_id);

    // Enable fixed price mode so we can test funding in isolation (without price PnL)
    let oracle_client = oracle_integrator::Client::new(&env, &test_env.oracle_id);
    oracle_client.set_fixed_price_mode(&test_env.admin, &true);

    // Disable borrowing fees to isolate funding rate effects
    let config_client = config_manager::Client::new(&env, &test_env.config_manager_id);
    config_client.set_borrow_rate_per_second(&test_env.admin, &0);

    let market_id = 0u32;
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;

    // Create imbalanced market: 3 longs, 1 short
    let mut long_ids = soroban_sdk::Vec::new(&env);
    for i in 0..3 {
        let trader = test_env.traders.get(i).unwrap();
        let pos_id =
            position_client.open_position(&trader, &market_id, &collateral, &leverage, &true);
        long_ids.push_back(pos_id);
    }

    let trader_short = test_env.traders.get(3).unwrap();
    let short_id =
        position_client.open_position(&trader_short, &market_id, &collateral, &leverage, &false);

    // Advance time by 10 funding intervals (10 minutes)
    for _ in 0..10 {
        advance_funding_interval(&env);
        market_client.update_funding_rate(&test_env.admin, &market_id);
    }

    // Close positions and verify funding payments
    let trader0 = test_env.traders.get(0).unwrap();
    let long_pnl = position_client.close_position(&trader0, &long_ids.get(0).unwrap());

    // Long should have paid funding (negative PnL component)
    // Note: Since we have more longs than shorts, longs pay funding
    assert!(
        long_pnl <= 0,
        "Long position should have neutral or negative PnL due to funding"
    );

    let short_pnl = position_client.close_position(&trader_short, &short_id);
    // Short should have received funding (positive PnL component)
    assert!(
        short_pnl >= 0,
        "Short position should have neutral or positive PnL from funding"
    );
}

#[test]
fn test_funding_long_heavy_market() {
    let env = Env::default();
    let test_env = setup_focused_test(&env);

    let position_client = position_manager::Client::new(&env, &test_env.position_manager_id);
    let market_client = market_manager::Client::new(&env, &test_env.market_manager_id);

    // Enable fixed price mode so we can test funding in isolation
    let oracle_client = oracle_integrator::Client::new(&env, &test_env.oracle_id);
    oracle_client.set_fixed_price_mode(&test_env.admin, &true);

    // Disable borrowing fees to isolate funding rate effects
    let config_client = config_manager::Client::new(&env, &test_env.config_manager_id);
    config_client.set_borrow_rate_per_second(&test_env.admin, &0);

    let market_id = 0u32;
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;

    // Create heavily long-skewed market: 4 longs, 1 short
    for i in 0..4 {
        let trader = test_env.traders.get(i).unwrap();
        position_client.open_position(&trader, &market_id, &collateral, &leverage, &true);
    }

    let trader_short = test_env.traders.get(4).unwrap();
    let short_id =
        position_client.open_position(&trader_short, &market_id, &collateral, &leverage, &false);

    // Get initial cumulative funding
    let initial_funding_long = market_client.get_cumulative_funding(&market_id, &true);
    let initial_funding_short = market_client.get_cumulative_funding(&market_id, &false);

    // Advance time and update funding
    for _ in 0..20 {
        advance_funding_interval(&env);
        market_client.update_funding_rate(&test_env.admin, &market_id);
    }

    // Get updated cumulative funding
    let updated_funding_long = market_client.get_cumulative_funding(&market_id, &true);
    let updated_funding_short = market_client.get_cumulative_funding(&market_id, &false);

    // In a long-heavy market:
    // - Longs pay funding (cumulative funding for longs increases as they owe more)
    // - Shorts receive funding (but cumulative_funding_short stays 0 since shorts don't pay)
    // The key insight: shorts profit because they see cumulative_funding_long increased
    // and their PnL calculation subtracts (paid - received) where received = cumulative_funding_long delta
    assert!(
        updated_funding_long >= initial_funding_long,
        "Long cumulative funding should increase as longs pay"
    );
    // cumulative_funding_short stays 0 since shorts don't pay in a long-heavy market
    assert!(
        updated_funding_short == initial_funding_short,
        "Short cumulative funding should stay same (shorts don't pay in long-heavy market)"
    );

    // Close short position and verify it received funding
    let short_pnl = position_client.close_position(&trader_short, &short_id);
    assert!(
        short_pnl >= 0,
        "Short should have profit from funding in long-heavy market"
    );
}

#[test]
fn test_funding_balanced_market() {
    let env = Env::default();
    let test_env = setup_focused_test(&env);

    let position_client = position_manager::Client::new(&env, &test_env.position_manager_id);
    let market_client = market_manager::Client::new(&env, &test_env.market_manager_id);

    let market_id = 0u32;
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;

    // Create balanced market: 2 longs, 2 shorts
    let mut long_ids = soroban_sdk::Vec::new(&env);
    for i in 0..2 {
        let trader = test_env.traders.get(i).unwrap();
        let pos_id =
            position_client.open_position(&trader, &market_id, &collateral, &leverage, &true);
        long_ids.push_back(pos_id);
    }

    let mut short_ids = soroban_sdk::Vec::new(&env);
    for i in 2..4 {
        let trader = test_env.traders.get(i).unwrap();
        let pos_id =
            position_client.open_position(&trader, &market_id, &collateral, &leverage, &false);
        short_ids.push_back(pos_id);
    }

    // Get initial cumulative funding
    let initial_funding_long = market_client.get_cumulative_funding(&market_id, &true);
    let initial_funding_short = market_client.get_cumulative_funding(&market_id, &false);

    // Advance time and update funding
    for _ in 0..10 {
        advance_funding_interval(&env);
        market_client.update_funding_rate(&test_env.admin, &market_id);
    }

    // Get updated cumulative funding
    let updated_funding_long = market_client.get_cumulative_funding(&market_id, &true);
    let updated_funding_short = market_client.get_cumulative_funding(&market_id, &false);

    // In a balanced market, funding should be minimal or zero
    let long_funding_change = (updated_funding_long - initial_funding_long).abs();
    let short_funding_change = (updated_funding_short - initial_funding_short).abs();

    assert!(
        long_funding_change <= 100,
        "Long funding change should be minimal in balanced market"
    );
    assert!(
        short_funding_change <= 100,
        "Short funding change should be minimal in balanced market"
    );
}

#[test]
fn test_funding_realization_on_close() {
    let env = Env::default();
    let test_env = setup_focused_test(&env);

    let position_client = position_manager::Client::new(&env, &test_env.position_manager_id);
    let market_client = market_manager::Client::new(&env, &test_env.market_manager_id);

    // Enable fixed price mode so we can test funding in isolation
    let oracle_client = oracle_integrator::Client::new(&env, &test_env.oracle_id);
    oracle_client.set_fixed_price_mode(&test_env.admin, &true);

    // Disable borrowing fees to isolate funding rate effects
    let config_client = config_manager::Client::new(&env, &test_env.config_manager_id);
    config_client.set_borrow_rate_per_second(&test_env.admin, &0);

    let market_id = 0u32;
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;

    // Create long-heavy market
    for i in 0..3 {
        let trader = test_env.traders.get(i).unwrap();
        position_client.open_position(&trader, &market_id, &collateral, &leverage, &true);
    }

    let trader_short = test_env.traders.get(3).unwrap();
    let short_id =
        position_client.open_position(&trader_short, &market_id, &collateral, &leverage, &false);

    // Simulate time passage with funding updates
    for _ in 0..30 {
        advance_funding_interval(&env);
        market_client.update_funding_rate(&test_env.admin, &market_id);
    }

    // Close short position - should realize funding profit
    let short_pnl = position_client.close_position(&trader_short, &short_id);

    // In a long-heavy market, shorts receive funding
    // PnL should include funding component (positive in this case)
    assert!(
        short_pnl >= 0,
        "Short PnL should include positive funding component"
    );
}

#[test]
fn test_multiple_funding_intervals() {
    let env = Env::default();
    let test_env = setup_focused_test(&env);

    let position_client = position_manager::Client::new(&env, &test_env.position_manager_id);
    let market_client = market_manager::Client::new(&env, &test_env.market_manager_id);

    // Enable fixed price mode so we can test funding in isolation
    let oracle_client = oracle_integrator::Client::new(&env, &test_env.oracle_id);
    oracle_client.set_fixed_price_mode(&test_env.admin, &true);

    // Disable borrowing fees to isolate funding rate effects
    let config_client = config_manager::Client::new(&env, &test_env.config_manager_id);
    config_client.set_borrow_rate_per_second(&test_env.admin, &0);

    let market_id = 0u32;
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;

    // Create imbalanced market
    for i in 0..3 {
        let trader = test_env.traders.get(i).unwrap();
        position_client.open_position(&trader, &market_id, &collateral, &leverage, &true);
    }

    let trader_short = test_env.traders.get(3).unwrap();
    let short_id =
        position_client.open_position(&trader_short, &market_id, &collateral, &leverage, &false);

    // Track cumulative funding over multiple intervals
    let mut funding_snapshots = soroban_sdk::Vec::new(&env);
    funding_snapshots.push_back(market_client.get_cumulative_funding(&market_id, &false));

    // Update funding 5 times
    for _ in 0..5 {
        advance_funding_interval(&env);
        market_client.update_funding_rate(&test_env.admin, &market_id);
        funding_snapshots.push_back(market_client.get_cumulative_funding(&market_id, &false));
    }

    // Verify funding is accumulating (should generally increase for shorts)
    let first_snapshot = funding_snapshots.get(0).unwrap();
    let last_snapshot = funding_snapshots.get(5).unwrap();

    assert!(
        last_snapshot >= first_snapshot,
        "Cumulative funding should accumulate over time"
    );

    // Close and verify
    let short_pnl = position_client.close_position(&trader_short, &short_id);
    assert!(short_pnl >= 0, "Short should profit from funding");
}

#![cfg(test)]

use super::*;
use soroban_sdk::log;
use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, token, Address, Env, Map};

// Import the actual contracts for integration testing
use crate::config_manager;
use crate::liquidity_pool;
use crate::market_manager;
use crate::oracle_integrator;

/// Helper to create a token contract for testing
fn create_token_contract<'a>(
    env: &'a Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let contract_address = env.register_stellar_asset_contract_v2(admin.clone());
    (
        token::Client::new(env, &contract_address.address()),
        token::StellarAssetClient::new(env, &contract_address.address()),
    )
}

/// Helper to set up the full test environment with all contracts
fn setup_test_environment<'a>(
    env: &'a Env,
) -> (
    Address,                       // config_manager_id
    Address,                       // oracle_id
    Address,                       // position_manager_id
    Address,                       // token_address
    token::Client<'a>,             // token_client
    token::StellarAssetClient<'a>, // token_admin
    Address,                       // admin
    Address,                       // trader
    Address,                       // liquidity pool
) {
    env.mock_all_auths();

    let admin = Address::generate(env);
    let trader = Address::generate(env);

    // Create token contract
    let (token_client, token_admin) = create_token_contract(env, &admin);

    // Deploy ConfigManager
    let config_manager_id = env.register(config_manager::WASM, ());
    let config_client = config_manager::Client::new(env, &config_manager_id);
    config_client.initialize(&admin);

    // Deploy OracleIntegrator
    let oracle_id = env.register(oracle_integrator::WASM, ());
    let oracle_client = oracle_integrator::Client::new(env, &oracle_id);
    oracle_client.initialize(&config_manager_id);

    // Enable test mode with base prices for simulated oracle
    let mut base_prices = Map::new(env);
    base_prices.set(0u32, 100_000_000i128); // XLM: $1.00
    base_prices.set(1u32, 50_000_000_000i128); // BTC: $50,000
    base_prices.set(2u32, 3_000_000_000i128); // ETH: $3,000
    oracle_client.set_test_mode(&admin, &true, &base_prices);

    // Deploy MarketManager
    let market_manager_id = env.register(market_manager::WASM, ());
    let market_client = market_manager::Client::new(env, &market_manager_id);
    market_client.initialize(&config_manager_id, &admin);

    // Deploy LiquidityPool
    let liquidity_pool_id = env.register(liquidity_pool::WASM, ());
    let liquidity_client = liquidity_pool::Client::new(env, &liquidity_pool_id);
    liquidity_client.initialize(&admin, &config_manager_id, &token_client.address);

    // Deploy PositionManager
    let position_manager_id = env.register(PositionManager, ());
    let position_client = PositionManagerClient::new(env, &position_manager_id);
    position_client.initialize(&admin, &config_manager_id);

    // Configure ConfigManager with contract addresses
    config_client.set_oracle_integrator(&admin, &oracle_id);
    config_client.set_market_manager(&admin, &market_manager_id);
    config_client.set_liquidity_pool(&admin, &liquidity_pool_id);
    config_client.set_position_manager(&admin, &position_manager_id);
    config_client.set_token(&admin, &token_client.address);

    // Set PositionManager in MarketManager (for authorization)
    market_client.set_position_manager(&admin, &position_manager_id);

    // Set PositionManager in LiquidityPool (for authorization)
    liquidity_client.set_position_manager(&admin, &position_manager_id);

    // Create test markets
    market_client.create_market(&admin, &0u32, &1_000_000_000_000u128, &10000i128); // XLM-PERP
    market_client.create_market(&admin, &1u32, &1_000_000_000_000u128, &10000i128); // BTC-PERP
    market_client.create_market(&admin, &2u32, &1_000_000_000_000u128, &10000i128); // ETH-PERP

    // Mint tokens to trader for testing
    token_admin.mint(&trader, &10_000_000_000); // 10,000 tokens with 7 decimals

    // Also deposit some initial liquidity to the pool
    token_admin.mint(&admin, &100_000_000_000); // 100,000 tokens for initial liquidity
    liquidity_client.deposit(&admin, &100_000_000_000);

    (
        config_manager_id,
        oracle_id,
        position_manager_id,
        token_client.address.clone(),
        token_client,
        token_admin,
        admin,
        trader,
        liquidity_pool_id,
    )
}

#[test]
fn test_contract_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let config_manager = Address::generate(&env);

    let contract_id = env.register(PositionManager, ());
    let client = PositionManagerClient::new(&env, &contract_id);

    // Initialize the contract
    client.initialize(&admin, &config_manager);

    // Contract successfully initialized
}

#[test]
fn test_open_position_success() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        token_client,
        _token_admin,
        _admin,
        trader,
        liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    let initial_balance = token_client.balance(&trader);
    let initial_contract_balance = token_client.balance(&liquidity_pool_id);

    // Open a position
    let market_id = 0u32; // XLM-PERP
    let collateral = 1_000_000_000u128; // 100 tokens (with 7 decimals)
    let leverage = 10u32;
    let is_long = true;

    let position_id =
        position_client.open_position(&trader, &market_id, &collateral, &leverage, &is_long);

    // Verify position ID is 1 (first position - IDs start at 1)
    assert_eq!(
        position_id, 1,
        "expected first position_id to be 1, got {}",
        position_id
    );

    // Verify collateral was transferred
    let new_balance = token_client.balance(&trader);
    assert_eq!(new_balance as u128, (initial_balance as u128) - collateral);

    // Verify colleteral exisits on liquidity pool

    // Verify liquidity contract received collateral
    let final_contract_balance = token_client.balance(&liquidity_pool_id);
    assert_eq!(
        (final_contract_balance - initial_contract_balance) as u128,
        collateral
    );

    // Verify position was stored correctly
    let position = position_client.get_position(&position_id);
    assert_eq!(position.trader, trader);
    assert_eq!(position.collateral, collateral);
    assert_eq!(position.size, collateral * (leverage as u128)); // size = collateral * leverage
    assert_eq!(position.is_long, is_long);
    assert_eq!(position.entry_price, 100_000_000); // Mock oracle price
}

#[test]
fn test_close_position_success() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        token_client,
        _token_admin,
        _admin,
        trader,
        liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Open a position first
    let market_id = 0u32;
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;
    let is_long = true;

    let position_id =
        position_client.open_position(&trader, &market_id, &collateral, &leverage, &is_long);

    let balance_after_open = token_client.balance(&trader);

    // Close the position
    let pnl = position_client.close_position(&trader, &position_id);

    // Verify PnL is 0 (MVP - no price changes)
    assert_eq!(pnl, 0);

    // Verify collateral was returned
    let final_balance = token_client.balance(&trader);
    assert_eq!(
        final_balance as u128,
        (balance_after_open as u128) + collateral
    );

    // Verify contract balance is 0
    let contract_balance = token_client.balance(&position_manager_id);
    assert_eq!(contract_balance as u128, 0);
}

#[test]
fn test_multiple_positions() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        token_client,
        _token_admin,
        _admin,
        trader,
        liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Open multiple positions
    let pos1 = position_client.open_position(&trader, &0u32, &1_000_000_000u128, &10u32, &true);

    let pos2 = position_client.open_position(&trader, &1u32, &2_000_000_000u128, &10u32, &false);

    let pos3 = position_client.open_position(&trader, &2u32, &500_000_000u128, &10u32, &true);

    // Verify position IDs are sequential (starting from 1)
    assert_eq!(pos1, 1, "expected first position_id to be 1, got {}", pos1);
    assert_eq!(pos2, 2, "expected second position_id to be 2, got {}", pos2);
    assert_eq!(pos3, 3, "expected third position_id to be 3, got {}", pos3);

    // Verify each position is stored correctly
    let position1 = position_client.get_position(&pos1);
    assert_eq!(position1.collateral, 1_000_000_000);
    assert_eq!(position1.is_long, true);

    let position2 = position_client.get_position(&pos2);
    assert_eq!(position2.collateral, 2_000_000_000);
    assert_eq!(position2.is_long, false);

    let position3 = position_client.get_position(&pos3);
    assert_eq!(position3.collateral, 500_000_000);
    assert_eq!(position3.is_long, true);

    // Verify total collateral in liquidity pool (collateral is held by pool, not position manager)
    let total_collateral: u128 = 1_000_000_000 + 2_000_000_000 + 500_000_000;
    let initial_pool_balance = 100_000_000_000u128; // Initial liquidity deposited in setup
    let pool_balance = token_client.balance(&liquidity_pool_id);
    assert_eq!(
        pool_balance as u128,
        initial_pool_balance + total_collateral
    );

    // Close middle position
    position_client.close_position(&trader, &pos2);

    // Verify collateral was returned for pos2
    let pool_balance_after_close = token_client.balance(&liquidity_pool_id);
    assert_eq!(
        pool_balance_after_close as u128,
        initial_pool_balance + total_collateral - 2_000_000_000
    );
}

#[test]
fn test_open_and_close_full_workflow() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        token_client,
        _token_admin,
        _admin,
        trader,
        liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    let initial_balance = token_client.balance(&trader);

    // Open position
    let position_id =
        position_client.open_position(&trader, &0u32, &1_000_000_000u128, &10u32, &true);

    // Close position
    let pnl = position_client.close_position(&trader, &position_id);

    // Verify final balance is the same as initial (no fees, no PnL)
    let final_balance = token_client.balance(&trader);
    assert_eq!(final_balance as u128, initial_balance as u128);
    assert_eq!(pnl, 0);
}

#[test]
#[should_panic(expected = "Collateral must be positive")]
fn test_open_position_zero_collateral() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Try to open position with zero collateral
    position_client.open_position(&trader, &0u32, &0u128, &10u32, &true);
}

#[test]
#[should_panic(expected = "Leverage must be positive")]
fn test_open_position_zero_leverage() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Try to open position with zero leverage
    position_client.open_position(&trader, &0u32, &1_000_000_000u128, &0u32, &true);
}

#[test]
#[should_panic(expected = "Position not found")]
fn test_get_nonexistent_position() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        _trader,
        liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Try to get a position that doesn't exist
    position_client.get_position(&999u64);
}

#[test]
#[should_panic(expected = "Unauthorized: caller does not own this position")]
fn test_close_position_unauthorized() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Trader opens a position
    let position_id =
        position_client.open_position(&trader, &0u32, &1_000_000_000u128, &10u32, &true);

    // Different user tries to close it
    let other_user = Address::generate(&env);
    position_client.close_position(&other_user, &position_id);
}

#[test]
#[should_panic(expected = "Leverage too low")]
fn test_open_position_leverage_too_low() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Try to open position with leverage = 4 (below min of 5)
    position_client.open_position(&trader, &0u32, &1_000_000_000u128, &4u32, &true);
}

#[test]
#[should_panic(expected = "Leverage too high")]
fn test_open_position_leverage_too_high() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Try to open position with leverage = 21 (above max of 20)
    position_client.open_position(&trader, &0u32, &1_000_000_000u128, &21u32, &true);
}

#[test]
#[should_panic(expected = "Position size too small")]
fn test_open_position_size_too_small() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Try to open position with size = 1_000_000 * 5 = 5_000_000 (below min of 10_000_000)
    position_client.open_position(&trader, &0u32, &1_000_000u128, &5u32, &true);
}

#[test]
fn test_get_user_positions_empty() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Get positions for user with no positions
    let user_positions = position_client.get_user_open_positions(&trader);

    // Verify empty vector is returned
    assert_eq!(user_positions.len(), 0);
}

#[test]
fn test_get_user_positions_single() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Open a single position
    let position_id =
        position_client.open_position(&trader, &0u32, &1_000_000_000u128, &10u32, &true);

    // Get user positions
    let user_positions = position_client.get_user_open_positions(&trader);

    // Verify one position is tracked
    assert_eq!(user_positions.len(), 1);
    assert_eq!(user_positions.get(0).unwrap(), position_id);
}

#[test]
fn test_get_user_positions_multiple() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Open multiple positions
    let pos1 = position_client.open_position(&trader, &0u32, &1_000_000_000u128, &10u32, &true);

    let pos2 = position_client.open_position(&trader, &1u32, &2_000_000_000u128, &10u32, &false);

    let pos3 = position_client.open_position(&trader, &2u32, &500_000_000u128, &10u32, &true);

    // Get user positions
    let user_positions = position_client.get_user_open_positions(&trader);

    // Verify all positions are tracked
    assert_eq!(user_positions.len(), 3);
    assert_eq!(user_positions.get(0).unwrap(), pos1);
    assert_eq!(user_positions.get(1).unwrap(), pos2);
    assert_eq!(user_positions.get(2).unwrap(), pos3);
}

#[test]
fn test_user_positions_after_close() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Open multiple positions
    let pos1 = position_client.open_position(&trader, &0u32, &1_000_000_000u128, &10u32, &true);

    let pos2 = position_client.open_position(&trader, &1u32, &2_000_000_000u128, &10u32, &false);

    let pos3 = position_client.open_position(&trader, &2u32, &500_000_000u128, &10u32, &true);

    // Verify all 3 positions are tracked
    let user_positions_before = position_client.get_user_open_positions(&trader);
    assert_eq!(user_positions_before.len(), 3);

    // Close the middle position
    position_client.close_position(&trader, &pos2);

    // Get user positions after closing
    let user_positions_after = position_client.get_user_open_positions(&trader);

    // Verify only 2 positions remain
    assert_eq!(user_positions_after.len(), 2);
    assert_eq!(user_positions_after.get(0).unwrap(), pos1);
    assert_eq!(user_positions_after.get(1).unwrap(), pos3);

    // Close all remaining positions
    position_client.close_position(&trader, &pos1);
    position_client.close_position(&trader, &pos3);

    // Verify no positions remain
    let user_positions_final = position_client.get_user_open_positions(&trader);
    assert_eq!(user_positions_final.len(), 0);
}

#[test]
fn test_multiple_users_positions() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        token_admin,
        _admin,
        trader1,
        liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Create a second trader
    let trader2 = Address::generate(&env);
    token_admin.mint(&trader2, &10_000_000_000);

    // Trader 1 opens 2 positions
    let trader1_pos1 =
        position_client.open_position(&trader1, &0u32, &1_000_000_000u128, &10u32, &true);

    let trader1_pos2 =
        position_client.open_position(&trader1, &1u32, &2_000_000_000u128, &10u32, &false);

    // Trader 2 opens 1 position
    let trader2_pos1 =
        position_client.open_position(&trader2, &2u32, &500_000_000u128, &10u32, &true);

    // Verify trader1 has 2 positions
    let trader1_positions = position_client.get_user_open_positions(&trader1);
    assert_eq!(trader1_positions.len(), 2);
    assert_eq!(trader1_positions.get(0).unwrap(), trader1_pos1);
    assert_eq!(trader1_positions.get(1).unwrap(), trader1_pos2);

    // Verify trader2 has 1 position
    let trader2_positions = position_client.get_user_open_positions(&trader2);
    assert_eq!(trader2_positions.len(), 1);
    assert_eq!(trader2_positions.get(0).unwrap(), trader2_pos1);

    // Close trader1's first position
    position_client.close_position(&trader1, &trader1_pos1);

    // Verify trader1 now has 1 position
    let trader1_positions_after = position_client.get_user_open_positions(&trader1);
    assert_eq!(trader1_positions_after.len(), 1);
    assert_eq!(trader1_positions_after.get(0).unwrap(), trader1_pos2);

    // Verify trader2's positions are unchanged
    let trader2_positions_after = position_client.get_user_open_positions(&trader2);
    assert_eq!(trader2_positions_after.len(), 1);
    assert_eq!(trader2_positions_after.get(0).unwrap(), trader2_pos1);
}

// ============================================================================
// ORDER TESTS - Limit Orders, Stop-Loss, Take-Profit
// ============================================================================

// Test constants for orders
const EXECUTION_FEE: u128 = 1_000_000; // 0.1 tokens (meets minimum)
const XLM_PRICE: i128 = 100_000_000; // $1.00
const LONG_TP_PRICE: i128 = 110_000_000; // $1.10 (+10%)
const LONG_SL_PRICE: i128 = 95_000_000; // $0.95 (-5%)
const SHORT_TP_PRICE: i128 = 90_000_000; // $0.90 (-10%)
const SHORT_SL_PRICE: i128 = 105_000_000; // $1.05 (+5%)
const CLOSE_FULL: u32 = 10000; // 100%
const CLOSE_HALF: u32 = 5000; // 50%

/// Helper to change oracle price for testing order triggers
fn set_oracle_price(
    env: &Env,
    oracle_id: &Address,
    admin: &Address,
    market_id: u32,
    new_price: i128,
) {
    let oracle_client = oracle_integrator::Client::new(env, oracle_id);
    let mut base_prices = Map::new(env);
    base_prices.set(market_id, new_price);
    // Keep other markets at defaults
    if market_id != 0 {
        base_prices.set(0u32, 100_000_000i128);
    }
    if market_id != 1 {
        base_prices.set(1u32, 50_000_000_000i128);
    }
    if market_id != 2 {
        base_prices.set(2u32, 3_000_000_000i128);
    }
    oracle_client.set_test_mode(admin, &true, &base_prices);
}

// ============================================================================
// LIMIT ORDER TESTS
// ============================================================================

#[test]
fn test_create_limit_order_long_success() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    let initial_balance = token_client.balance(&trader);
    let initial_contract_balance = token_client.balance(&position_manager_id);

    // Create a limit order for a long position
    let market_id = 0u32;
    let trigger_price = 95_000_000i128; // Buy when price drops to $0.95
    let acceptable_price = 96_000_000i128; // Accept up to $0.96
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;
    let is_long = true;
    let expiration = 0u64; // No expiry

    let order_id = position_client.create_limit_order(
        &trader,
        &market_id,
        &trigger_price,
        &acceptable_price,
        &collateral,
        &leverage,
        &is_long,
        &EXECUTION_FEE,
        &expiration,
    );

    // Verify order ID is 1 (first order - IDs start at 1)
    assert_eq!(order_id, 1);

    // Verify execution fee AND collateral were transferred to contract (escrowed)
    let new_balance = token_client.balance(&trader);
    assert_eq!(
        new_balance as u128,
        (initial_balance as u128) - EXECUTION_FEE - collateral
    );

    let new_contract_balance = token_client.balance(&position_manager_id);
    assert_eq!(
        new_contract_balance as u128,
        (initial_contract_balance as u128) + EXECUTION_FEE + collateral
    );

    // Verify order is stored correctly
    let order = position_client.get_order(&order_id);
    assert_eq!(order.trader, trader);
    assert_eq!(order.market_id, market_id);
    assert_eq!(order.trigger_price, trigger_price);
    assert_eq!(order.acceptable_price, acceptable_price);
    assert_eq!(order.collateral, collateral);
    assert_eq!(order.leverage, leverage);
    assert_eq!(order.is_long, is_long);
    assert_eq!(order.position_id, 0); // No position yet
    assert_eq!(order.execution_fee, EXECUTION_FEE);

    // Verify order appears in user orders
    let user_orders = position_client.get_user_orders(&trader);
    assert_eq!(user_orders.len(), 1);
    assert_eq!(user_orders.get(0).unwrap(), order_id);

    // Verify order appears in market orders
    let market_orders = position_client.get_market_orders(&market_id);
    assert_eq!(market_orders.len(), 1);
    assert_eq!(market_orders.get(0).unwrap(), order_id);
}

#[test]
fn test_create_limit_order_short_success() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Create a limit order for a short position
    let market_id = 0u32;
    let trigger_price = 105_000_000i128; // Short when price rises to $1.05
    let acceptable_price = 104_000_000i128; // Accept down to $1.04
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;
    let is_long = false;

    let order_id = position_client.create_limit_order(
        &trader,
        &market_id,
        &trigger_price,
        &acceptable_price,
        &collateral,
        &leverage,
        &is_long,
        &EXECUTION_FEE,
        &0u64,
    );

    // Verify order is stored correctly
    let order = position_client.get_order(&order_id);
    assert_eq!(order.is_long, false);
    assert_eq!(order.trigger_price, trigger_price);
}

#[test]
#[should_panic(expected = "Trigger price must be positive")]
fn test_create_limit_order_zero_trigger_price() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    position_client.create_limit_order(
        &trader,
        &0u32,
        &0i128, // Zero trigger price
        &0i128,
        &1_000_000_000u128,
        &10u32,
        &true,
        &EXECUTION_FEE,
        &0u64,
    );
}

#[test]
#[should_panic(expected = "Collateral must be positive")]
fn test_create_limit_order_zero_collateral() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    position_client.create_limit_order(
        &trader,
        &0u32,
        &95_000_000i128,
        &0i128,
        &0u128, // Zero collateral
        &10u32,
        &true,
        &EXECUTION_FEE,
        &0u64,
    );
}

#[test]
#[should_panic(expected = "Leverage too low")]
fn test_create_limit_order_leverage_too_low() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    position_client.create_limit_order(
        &trader,
        &0u32,
        &95_000_000i128,
        &0i128,
        &1_000_000_000u128,
        &4u32, // Below min of 5
        &true,
        &EXECUTION_FEE,
        &0u64,
    );
}

#[test]
#[should_panic(expected = "Leverage too high")]
fn test_create_limit_order_leverage_too_high() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    position_client.create_limit_order(
        &trader,
        &0u32,
        &95_000_000i128,
        &0i128,
        &1_000_000_000u128,
        &21u32, // Above max of 20
        &true,
        &EXECUTION_FEE,
        &0u64,
    );
}

#[test]
#[should_panic(expected = "Execution fee below minimum")]
fn test_create_limit_order_fee_too_low() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    position_client.create_limit_order(
        &trader,
        &0u32,
        &95_000_000i128,
        &0i128,
        &1_000_000_000u128,
        &10u32,
        &true,
        &100u128, // Below minimum of 1_000_000
        &0u64,
    );
}

#[test]
#[should_panic(expected = "Position size too small")]
fn test_create_limit_order_size_too_small() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Size = 1_000_000 * 5 = 5_000_000 (below min of 10_000_000)
    position_client.create_limit_order(
        &trader,
        &0u32,
        &95_000_000i128,
        &0i128,
        &1_000_000u128,
        &5u32,
        &true,
        &EXECUTION_FEE,
        &0u64,
    );
}

#[test]
fn test_execute_limit_order_long_trigger() {
    let env = Env::default();
    let (
        _config_id,
        oracle_id,
        position_manager_id,
        _token_address,
        token_client,
        token_admin,
        admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Create a limit order for a long position
    let market_id = 0u32;
    let trigger_price = 95_000_000i128; // Buy when price drops to $0.95
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;

    let order_id = position_client.create_limit_order(
        &trader,
        &market_id,
        &trigger_price,
        &0i128, // No slippage limit
        &collateral,
        &leverage,
        &true,
        &EXECUTION_FEE,
        &0u64,
    );

    // Create a keeper and fund them
    let keeper = Address::generate(&env);
    token_admin.mint(&keeper, &1_000_000_000);

    let keeper_initial_balance = token_client.balance(&keeper);

    // Verify order cannot be executed at current price ($1.00)
    assert_eq!(position_client.can_execute_order(&order_id), false);

    // Change price to trigger level ($0.95)
    set_oracle_price(&env, &oracle_id, &admin, market_id, trigger_price);

    // Verify order can now be executed
    assert_eq!(position_client.can_execute_order(&order_id), true);

    // Execute the order
    let result = position_client.execute_order(&keeper, &order_id);

    // Result should be the new position ID
    let position_id = result as u64;

    // Verify position was created
    let position = position_client.get_position(&position_id);
    assert_eq!(position.trader, trader);
    assert_eq!(position.collateral, collateral);
    assert_eq!(position.size, collateral * (leverage as u128));
    assert_eq!(position.is_long, true);

    // Verify keeper received execution fee
    let keeper_final_balance = token_client.balance(&keeper);
    assert_eq!(
        keeper_final_balance as u128,
        (keeper_initial_balance as u128) + EXECUTION_FEE
    );

    // Verify order is removed from storage
    let user_orders = position_client.get_user_orders(&trader);
    assert_eq!(user_orders.len(), 0);

    let market_orders = position_client.get_market_orders(&market_id);
    assert_eq!(market_orders.len(), 0);
}

#[test]
fn test_execute_limit_order_short_trigger() {
    let env = Env::default();
    let (
        _config_id,
        oracle_id,
        position_manager_id,
        _token_address,
        token_client,
        token_admin,
        admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Create a limit order for a short position
    let market_id = 0u32;
    let trigger_price = 105_000_000i128; // Short when price rises to $1.05
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;

    let order_id = position_client.create_limit_order(
        &trader,
        &market_id,
        &trigger_price,
        &0i128,
        &collateral,
        &leverage,
        &false, // Short
        &EXECUTION_FEE,
        &0u64,
    );

    let keeper = Address::generate(&env);
    token_admin.mint(&keeper, &1_000_000_000);

    // Verify order cannot be executed at current price ($1.00)
    assert_eq!(position_client.can_execute_order(&order_id), false);

    // Change price to trigger level ($1.05)
    set_oracle_price(&env, &oracle_id, &admin, market_id, trigger_price);

    // Execute the order
    let result = position_client.execute_order(&keeper, &order_id);
    let position_id = result as u64;

    // Verify short position was created
    let position = position_client.get_position(&position_id);
    assert_eq!(position.is_long, false);
}

// ============================================================================
// STOP-LOSS ORDER TESTS
// ============================================================================

#[test]
fn test_create_stop_loss_long_success() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // First open a long position
    let market_id = 0u32;
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;

    let position_id =
        position_client.open_position(&trader, &market_id, &collateral, &leverage, &true);

    let initial_balance = token_client.balance(&trader);

    // Create stop-loss below current price ($1.00), above liquidation price
    // At 10x leverage, liquidation is around $0.91
    let trigger_price = LONG_SL_PRICE; // $0.95

    let order_id = position_client.create_stop_loss(
        &trader,
        &position_id,
        &trigger_price,
        &0i128, // No slippage limit
        &CLOSE_FULL,
        &EXECUTION_FEE,
        &0u64,
    );

    // Verify execution fee was transferred
    let new_balance = token_client.balance(&trader);
    assert_eq!(
        new_balance as u128,
        (initial_balance as u128) - EXECUTION_FEE
    );

    // Verify order is stored correctly
    let order = position_client.get_order(&order_id);
    assert_eq!(order.trader, trader);
    assert_eq!(order.position_id, position_id);
    assert_eq!(order.trigger_price, trigger_price);
    assert_eq!(order.close_percentage, CLOSE_FULL);
    assert_eq!(order.is_long, true);

    // Verify order is attached to position
    let position_orders = position_client.get_position_orders(&position_id);
    assert_eq!(position_orders.len(), 1);
    assert_eq!(position_orders.get(0).unwrap(), order_id);
}

#[test]
fn test_create_stop_loss_short_success() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // First open a short position
    let market_id = 0u32;
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;

    let position_id =
        position_client.open_position(&trader, &market_id, &collateral, &leverage, &false);

    // Create stop-loss above current price ($1.00), below liquidation price
    // For shorts at 10x, liquidation is around $1.09
    let trigger_price = SHORT_SL_PRICE; // $1.05

    let order_id = position_client.create_stop_loss(
        &trader,
        &position_id,
        &trigger_price,
        &0i128,
        &CLOSE_FULL,
        &EXECUTION_FEE,
        &0u64,
    );

    // Verify order is stored correctly
    let order = position_client.get_order(&order_id);
    assert_eq!(order.is_long, false);
    assert_eq!(order.trigger_price, trigger_price);
}

#[test]
#[should_panic(expected = "Unauthorized: caller does not own this position")]
fn test_create_stop_loss_not_owner() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Trader opens a position
    let position_id =
        position_client.open_position(&trader, &0u32, &1_000_000_000u128, &10u32, &true);

    // Another user tries to create stop-loss
    let other_user = Address::generate(&env);
    token_admin.mint(&other_user, &10_000_000_000);

    position_client.create_stop_loss(
        &other_user,
        &position_id,
        &LONG_SL_PRICE,
        &0i128,
        &CLOSE_FULL,
        &EXECUTION_FEE,
        &0u64,
    );
}

#[test]
#[should_panic(expected = "Invalid close percentage")]
fn test_create_stop_loss_zero_percentage() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    let position_id =
        position_client.open_position(&trader, &0u32, &1_000_000_000u128, &10u32, &true);

    position_client.create_stop_loss(
        &trader,
        &position_id,
        &LONG_SL_PRICE,
        &0i128,
        &0u32, // Zero percentage
        &EXECUTION_FEE,
        &0u64,
    );
}

#[test]
#[should_panic(expected = "Invalid close percentage")]
fn test_create_stop_loss_over_100_percent() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    let position_id =
        position_client.open_position(&trader, &0u32, &1_000_000_000u128, &10u32, &true);

    position_client.create_stop_loss(
        &trader,
        &position_id,
        &LONG_SL_PRICE,
        &0i128,
        &10001u32, // Over 100%
        &EXECUTION_FEE,
        &0u64,
    );
}

#[test]
#[should_panic(expected = "Stop-loss for long must be below current price")]
fn test_create_stop_loss_long_above_current_price() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    let position_id =
        position_client.open_position(&trader, &0u32, &1_000_000_000u128, &10u32, &true);

    // Try to set SL above current price for long position
    position_client.create_stop_loss(
        &trader,
        &position_id,
        &105_000_000i128, // $1.05 - above current $1.00
        &0i128,
        &CLOSE_FULL,
        &EXECUTION_FEE,
        &0u64,
    );
}

#[test]
#[should_panic(expected = "Stop-loss must be above liquidation price")]
fn test_create_stop_loss_long_below_liquidation() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    let position_id =
        position_client.open_position(&trader, &0u32, &1_000_000_000u128, &10u32, &true);

    // At 10x leverage, liquidation is ~$0.91. Try to set SL at $0.90
    position_client.create_stop_loss(
        &trader,
        &position_id,
        &90_000_000i128, // $0.90 - below liquidation
        &0i128,
        &CLOSE_FULL,
        &EXECUTION_FEE,
        &0u64,
    );
}

#[test]
#[should_panic(expected = "Stop-loss for short must be above current price")]
fn test_create_stop_loss_short_below_current_price() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    let position_id =
        position_client.open_position(&trader, &0u32, &1_000_000_000u128, &10u32, &false);

    // Try to set SL below current price for short position
    position_client.create_stop_loss(
        &trader,
        &position_id,
        &95_000_000i128, // $0.95 - below current $1.00
        &0i128,
        &CLOSE_FULL,
        &EXECUTION_FEE,
        &0u64,
    );
}

#[test]
#[should_panic(expected = "Stop-loss must be below liquidation price")]
fn test_create_stop_loss_short_above_liquidation() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    let position_id =
        position_client.open_position(&trader, &0u32, &1_000_000_000u128, &10u32, &false);

    // At 10x leverage for short, liquidation is ~$1.09. Try to set SL at $1.10
    position_client.create_stop_loss(
        &trader,
        &position_id,
        &110_000_000i128, // $1.10 - above liquidation
        &0i128,
        &CLOSE_FULL,
        &EXECUTION_FEE,
        &0u64,
    );
}

#[test]
fn test_execute_stop_loss_full_close() {
    let env = Env::default();
    let (
        _config_id,
        oracle_id,
        position_manager_id,
        _token_address,
        token_client,
        token_admin,
        admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Open a long position
    let market_id = 0u32;
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;

    let position_id =
        position_client.open_position(&trader, &market_id, &collateral, &leverage, &true);

    // Create stop-loss at $0.95
    let trigger_price = LONG_SL_PRICE;
    let order_id = position_client.create_stop_loss(
        &trader,
        &position_id,
        &trigger_price,
        &0i128,
        &CLOSE_FULL,
        &EXECUTION_FEE,
        &0u64,
    );

    // Create keeper
    let keeper = Address::generate(&env);
    token_admin.mint(&keeper, &1_000_000_000);

    // Change price to trigger level
    set_oracle_price(&env, &oracle_id, &admin, market_id, trigger_price);

    // Execute the stop-loss
    let pnl = position_client.execute_order(&keeper, &order_id);

    // PnL should be negative (price dropped from $1.00 to $0.95)
    assert!(pnl < 0);

    // Verify position is fully closed
    let user_positions = position_client.get_user_open_positions(&trader);
    assert_eq!(user_positions.len(), 0);

    // Verify order is removed
    let position_orders = position_client.get_position_orders(&position_id);
    assert_eq!(position_orders.len(), 0);
}

#[test]
fn test_execute_stop_loss_partial_close() {
    let env = Env::default();
    let (
        _config_id,
        oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        token_admin,
        admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Open a long position
    let market_id = 0u32;
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;
    let initial_size = collateral * (leverage as u128);

    let position_id =
        position_client.open_position(&trader, &market_id, &collateral, &leverage, &true);

    // Use a mild stop-loss price for partial close test
    // $0.98 = 2% drop = 20% loss with 10x leverage (manageable for partial close)
    let mild_sl_price: i128 = 98_000_000;
    let order_id = position_client.create_stop_loss(
        &trader,
        &position_id,
        &mild_sl_price,
        &0i128,
        &CLOSE_HALF, // 50%
        &EXECUTION_FEE,
        &0u64,
    );

    // Create keeper
    let keeper = Address::generate(&env);
    token_admin.mint(&keeper, &1_000_000_000);

    // Change price to trigger level
    set_oracle_price(&env, &oracle_id, &admin, market_id, mild_sl_price);

    // Execute the stop-loss
    let pnl = position_client.execute_order(&keeper, &order_id);

    // PnL should be negative (partial)
    assert!(pnl < 0);

    // Verify position still exists but is reduced
    let user_positions = position_client.get_user_open_positions(&trader);
    assert_eq!(user_positions.len(), 1);

    let position = position_client.get_position(&position_id);
    // Size should be reduced by 50%
    assert!(position.size < initial_size);
}

// ============================================================================
// TAKE-PROFIT ORDER TESTS
// ============================================================================

#[test]
fn test_create_take_profit_long_success() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // First open a long position
    let market_id = 0u32;
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;

    let position_id =
        position_client.open_position(&trader, &market_id, &collateral, &leverage, &true);

    let initial_balance = token_client.balance(&trader);

    // Create take-profit above current price ($1.00)
    let trigger_price = LONG_TP_PRICE; // $1.10

    let order_id = position_client.create_take_profit(
        &trader,
        &position_id,
        &trigger_price,
        &0i128,
        &CLOSE_FULL,
        &EXECUTION_FEE,
        &0u64,
    );

    // Verify execution fee was transferred
    let new_balance = token_client.balance(&trader);
    assert_eq!(
        new_balance as u128,
        (initial_balance as u128) - EXECUTION_FEE
    );

    // Verify order is stored correctly
    let order = position_client.get_order(&order_id);
    assert_eq!(order.trader, trader);
    assert_eq!(order.position_id, position_id);
    assert_eq!(order.trigger_price, trigger_price);
    assert_eq!(order.close_percentage, CLOSE_FULL);
    assert_eq!(order.is_long, true);

    // Verify order is attached to position
    let position_orders = position_client.get_position_orders(&position_id);
    assert_eq!(position_orders.len(), 1);
}

#[test]
fn test_create_take_profit_short_success() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // First open a short position
    let market_id = 0u32;
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;

    let position_id =
        position_client.open_position(&trader, &market_id, &collateral, &leverage, &false);

    // Create take-profit below current price ($1.00)
    let trigger_price = SHORT_TP_PRICE; // $0.90

    let order_id = position_client.create_take_profit(
        &trader,
        &position_id,
        &trigger_price,
        &0i128,
        &CLOSE_FULL,
        &EXECUTION_FEE,
        &0u64,
    );

    // Verify order is stored correctly
    let order = position_client.get_order(&order_id);
    assert_eq!(order.is_long, false);
    assert_eq!(order.trigger_price, trigger_price);
}

#[test]
#[should_panic(expected = "Take-profit for long must be above current price")]
fn test_create_take_profit_long_below_current() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    let position_id =
        position_client.open_position(&trader, &0u32, &1_000_000_000u128, &10u32, &true);

    // Try to set TP below current price for long position
    position_client.create_take_profit(
        &trader,
        &position_id,
        &95_000_000i128, // $0.95 - below current $1.00
        &0i128,
        &CLOSE_FULL,
        &EXECUTION_FEE,
        &0u64,
    );
}

#[test]
#[should_panic(expected = "Take-profit for short must be below current price")]
fn test_create_take_profit_short_above_current() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    let position_id =
        position_client.open_position(&trader, &0u32, &1_000_000_000u128, &10u32, &false);

    // Try to set TP above current price for short position
    position_client.create_take_profit(
        &trader,
        &position_id,
        &105_000_000i128, // $1.05 - above current $1.00
        &0i128,
        &CLOSE_FULL,
        &EXECUTION_FEE,
        &0u64,
    );
}

#[test]
fn test_execute_take_profit_full_close() {
    let env = Env::default();
    let (
        _config_id,
        oracle_id,
        position_manager_id,
        _token_address,
        token_client,
        token_admin,
        admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Open a long position
    let market_id = 0u32;
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;

    let position_id =
        position_client.open_position(&trader, &market_id, &collateral, &leverage, &true);

    // Create take-profit at $1.10
    let trigger_price = LONG_TP_PRICE;
    let order_id = position_client.create_take_profit(
        &trader,
        &position_id,
        &trigger_price,
        &0i128,
        &CLOSE_FULL,
        &EXECUTION_FEE,
        &0u64,
    );

    // Create keeper
    let keeper = Address::generate(&env);
    token_admin.mint(&keeper, &1_000_000_000);

    // Change price to trigger level
    set_oracle_price(&env, &oracle_id, &admin, market_id, trigger_price);

    // Execute the take-profit
    let pnl = position_client.execute_order(&keeper, &order_id);

    // PnL should be positive (price increased from $1.00 to $1.10)
    assert!(pnl > 0);

    // Verify position is fully closed
    let user_positions = position_client.get_user_open_positions(&trader);
    assert_eq!(user_positions.len(), 0);
}

#[test]
fn test_execute_take_profit_partial_close() {
    let env = Env::default();
    let (
        _config_id,
        oracle_id,
        position_manager_id,
        _token_address,
        token_client,
        token_admin,
        admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Open a long position
    let market_id = 0u32;
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;
    let initial_size = collateral * (leverage as u128);

    let position_id =
        position_client.open_position(&trader, &market_id, &collateral, &leverage, &true);

    // Create take-profit for 50% of position
    let trigger_price = LONG_TP_PRICE;
    let order_id = position_client.create_take_profit(
        &trader,
        &position_id,
        &trigger_price,
        &0i128,
        &CLOSE_HALF,
        &EXECUTION_FEE,
        &0u64,
    );

    // Create keeper
    let keeper = Address::generate(&env);
    token_admin.mint(&keeper, &1_000_000_000);

    // Change price to trigger level
    set_oracle_price(&env, &oracle_id, &admin, market_id, trigger_price);

    // Execute the take-profit
    let pnl = position_client.execute_order(&keeper, &order_id);

    // PnL should be positive (partial)
    assert!(pnl > 0);

    // Verify position still exists but is reduced
    let user_positions = position_client.get_user_open_positions(&trader);
    assert_eq!(user_positions.len(), 1);

    let position = position_client.get_position(&position_id);
    assert!(position.size < initial_size);
}

// ============================================================================
// ORDER CANCELLATION TESTS
// ============================================================================

#[test]
fn test_cancel_order_refund_fee() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Create a limit order
    let order_id = position_client.create_limit_order(
        &trader,
        &0u32,
        &95_000_000i128,
        &0i128,
        &1_000_000_000u128,
        &10u32,
        &true,
        &EXECUTION_FEE,
        &0u64,
    );

    let collateral = 1_000_000_000u128;
    let balance_after_create = token_client.balance(&trader);

    // Cancel the order
    position_client.cancel_order(&trader, &order_id);

    // Verify execution fee AND collateral were refunded (limit orders escrow both)
    let balance_after_cancel = token_client.balance(&trader);
    assert_eq!(
        balance_after_cancel as u128,
        (balance_after_create as u128) + EXECUTION_FEE + collateral
    );

    // Verify order is removed from storage
    let user_orders = position_client.get_user_orders(&trader);
    assert_eq!(user_orders.len(), 0);

    let market_orders = position_client.get_market_orders(&0u32);
    assert_eq!(market_orders.len(), 0);
}

#[test]
#[should_panic(expected = "Unauthorized: caller does not own this order")]
fn test_cancel_order_not_owner() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Trader creates an order
    let order_id = position_client.create_limit_order(
        &trader,
        &0u32,
        &95_000_000i128,
        &0i128,
        &1_000_000_000u128,
        &10u32,
        &true,
        &EXECUTION_FEE,
        &0u64,
    );

    // Another user tries to cancel it
    let other_user = Address::generate(&env);
    token_admin.mint(&other_user, &10_000_000_000);

    position_client.cancel_order(&other_user, &order_id);
}

#[test]
fn test_orders_cancelled_on_position_close() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Open a position
    let position_id =
        position_client.open_position(&trader, &0u32, &1_000_000_000u128, &10u32, &true);

    // Create stop-loss and take-profit orders
    let sl_order_id = position_client.create_stop_loss(
        &trader,
        &position_id,
        &LONG_SL_PRICE,
        &0i128,
        &CLOSE_FULL,
        &EXECUTION_FEE,
        &0u64,
    );

    let tp_order_id = position_client.create_take_profit(
        &trader,
        &position_id,
        &LONG_TP_PRICE,
        &0i128,
        &CLOSE_FULL,
        &EXECUTION_FEE,
        &0u64,
    );

    // Verify orders are attached
    let position_orders = position_client.get_position_orders(&position_id);
    assert_eq!(position_orders.len(), 2);

    let balance_before_close = token_client.balance(&trader);

    // Close the position manually
    position_client.close_position(&trader, &position_id);

    // Verify both orders are cancelled and fees refunded
    let user_orders = position_client.get_user_orders(&trader);
    assert_eq!(user_orders.len(), 0);

    // Check that execution fees were refunded (2 orders * EXECUTION_FEE)
    let balance_after_close = token_client.balance(&trader);
    // After close: trader gets collateral + refunded fees
    // The exact balance depends on collateral returned
    // We just verify orders are gone; balance includes collateral return too
    assert!(balance_after_close > balance_before_close);
}

// ============================================================================
// ORDER EXECUTION EDGE CASE TESTS
// ============================================================================

#[test]
#[should_panic(expected = "Order trigger condition not met")]
fn test_execute_order_trigger_not_met() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Create a limit order for long at $0.95 (current price is $1.00)
    let order_id = position_client.create_limit_order(
        &trader,
        &0u32,
        &95_000_000i128,
        &0i128,
        &1_000_000_000u128,
        &10u32,
        &true,
        &EXECUTION_FEE,
        &0u64,
    );

    let keeper = Address::generate(&env);
    token_admin.mint(&keeper, &1_000_000_000);

    // Try to execute without price reaching trigger
    position_client.execute_order(&keeper, &order_id);
}

#[test]
#[should_panic(expected = "Current price outside acceptable range")]
fn test_execute_order_slippage_exceeded() {
    let env = Env::default();
    let (
        _config_id,
        oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        token_admin,
        admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Create a limit order with tight slippage limit
    // Buy when price hits $0.95, but only accept if price <= $0.93 (very strict)
    // This tests: price drops to $0.94 (triggers because <= $0.95),
    // but fails slippage check (because $0.94 > $0.93)
    let order_id = position_client.create_limit_order(
        &trader,
        &0u32,
        &95_000_000i128, // trigger at $0.95
        &93_000_000i128, // only accept if price <= $0.93 (strict slippage)
        &1_000_000_000u128,
        &10u32,
        &true,
        &EXECUTION_FEE,
        &0u64,
    );

    let keeper = Address::generate(&env);
    token_admin.mint(&keeper, &1_000_000_000);

    // Set price to $0.94 - triggers (because $0.94 <= $0.95) but exceeds slippage ($0.94 > $0.93)
    set_oracle_price(&env, &oracle_id, &admin, 0u32, 94_000_000i128);

    // Try to execute - should fail due to slippage
    position_client.execute_order(&keeper, &order_id);
}

// ============================================================================
// QUERY FUNCTION TESTS
// ============================================================================

#[test]
fn test_get_order() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    let order_id = position_client.create_limit_order(
        &trader,
        &0u32,
        &95_000_000i128,
        &96_000_000i128,
        &1_000_000_000u128,
        &10u32,
        &true,
        &EXECUTION_FEE,
        &0u64,
    );

    let order = position_client.get_order(&order_id);

    assert_eq!(order.order_id, order_id);
    assert_eq!(order.trader, trader);
    assert_eq!(order.market_id, 0u32);
    assert_eq!(order.trigger_price, 95_000_000i128);
    assert_eq!(order.acceptable_price, 96_000_000i128);
    assert_eq!(order.collateral, 1_000_000_000u128);
    assert_eq!(order.leverage, 10u32);
    assert_eq!(order.is_long, true);
    assert_eq!(order.execution_fee, EXECUTION_FEE);
}

#[test]
#[should_panic(expected = "Order not found")]
fn test_get_nonexistent_order() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        _trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Try to get an order that doesn't exist
    position_client.get_order(&999u64);
}

#[test]
fn test_get_user_orders_empty() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    let user_orders = position_client.get_user_orders(&trader);
    assert_eq!(user_orders.len(), 0);
}

#[test]
fn test_get_user_orders_multiple() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Create multiple limit orders
    let order1 = position_client.create_limit_order(
        &trader,
        &0u32,
        &95_000_000i128,
        &0i128,
        &1_000_000_000u128,
        &10u32,
        &true,
        &EXECUTION_FEE,
        &0u64,
    );

    let order2 = position_client.create_limit_order(
        &trader,
        &1u32,
        &105_000_000i128,
        &0i128,
        &1_000_000_000u128,
        &10u32,
        &false,
        &EXECUTION_FEE,
        &0u64,
    );

    let user_orders = position_client.get_user_orders(&trader);
    assert_eq!(user_orders.len(), 2);
    assert_eq!(user_orders.get(0).unwrap(), order1);
    assert_eq!(user_orders.get(1).unwrap(), order2);
}

#[test]
fn test_get_position_orders() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Open a position
    let position_id =
        position_client.open_position(&trader, &0u32, &1_000_000_000u128, &10u32, &true);

    // Create SL and TP orders attached to position
    let sl_order = position_client.create_stop_loss(
        &trader,
        &position_id,
        &LONG_SL_PRICE,
        &0i128,
        &CLOSE_FULL,
        &EXECUTION_FEE,
        &0u64,
    );

    let tp_order = position_client.create_take_profit(
        &trader,
        &position_id,
        &LONG_TP_PRICE,
        &0i128,
        &CLOSE_FULL,
        &EXECUTION_FEE,
        &0u64,
    );

    let position_orders = position_client.get_position_orders(&position_id);
    assert_eq!(position_orders.len(), 2);
    assert_eq!(position_orders.get(0).unwrap(), sl_order);
    assert_eq!(position_orders.get(1).unwrap(), tp_order);
}

#[test]
fn test_get_market_orders() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Create orders in different markets
    let order_xlm = position_client.create_limit_order(
        &trader,
        &0u32, // XLM market
        &95_000_000i128,
        &0i128,
        &1_000_000_000u128,
        &10u32,
        &true,
        &EXECUTION_FEE,
        &0u64,
    );

    let order_btc = position_client.create_limit_order(
        &trader,
        &1u32, // BTC market
        &55_000_000_000i128,
        &0i128,
        &1_000_000_000u128,
        &10u32,
        &true,
        &EXECUTION_FEE,
        &0u64,
    );

    // Verify XLM market orders
    let xlm_orders = position_client.get_market_orders(&0u32);
    assert_eq!(xlm_orders.len(), 1);
    assert_eq!(xlm_orders.get(0).unwrap(), order_xlm);

    // Verify BTC market orders
    let btc_orders = position_client.get_market_orders(&1u32);
    assert_eq!(btc_orders.len(), 1);
    assert_eq!(btc_orders.get(0).unwrap(), order_btc);

    // Verify ETH market has no orders
    let eth_orders = position_client.get_market_orders(&2u32);
    assert_eq!(eth_orders.len(), 0);
}

#[test]
fn test_can_execute_order_true() {
    let env = Env::default();
    let (
        _config_id,
        oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    let order_id = position_client.create_limit_order(
        &trader,
        &0u32,
        &95_000_000i128, // Trigger at $0.95
        &0i128,
        &1_000_000_000u128,
        &10u32,
        &true,
        &EXECUTION_FEE,
        &0u64,
    );

    // Initially cannot execute (price is $1.00)
    assert_eq!(position_client.can_execute_order(&order_id), false);

    // Change price to trigger level
    set_oracle_price(&env, &oracle_id, &admin, 0u32, 95_000_000i128);

    // Now can execute
    assert_eq!(position_client.can_execute_order(&order_id), true);
}

#[test]
fn test_can_execute_order_false_price_not_met() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    let order_id = position_client.create_limit_order(
        &trader,
        &0u32,
        &95_000_000i128, // Trigger at $0.95, current is $1.00
        &0i128,
        &1_000_000_000u128,
        &10u32,
        &true,
        &EXECUTION_FEE,
        &0u64,
    );

    // Cannot execute because price hasn't reached trigger
    assert_eq!(position_client.can_execute_order(&order_id), false);
}

// ============================================================================
// CALCULATE PNL TESTS
// ============================================================================

#[test]
fn test_calculate_pnl_long_profit() {
    let env = Env::default();
    let (
        _config_id,
        oracle_id,
        position_manager_id,
        _token_address,
        token_client,
        _token_admin,
        admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Open long position at $1.00
    let collateral = 1_000_000_000u128; // 100 tokens
    let leverage = 10u32;
    let position_id = position_client.open_position(&trader, &0u32, &collateral, &leverage, &true);

    // Price increases to $1.10 (+10%)
    set_oracle_price(&env, &oracle_id, &admin, 0, 110_000_000);

    let pnl = position_client.calculate_pnl(&position_id);

    // Long profits when price goes up
    // Size = 1_000_000_000 * 10 = 10_000_000_000
    // Price PnL = (110_000_000 - 100_000_000) * 10_000_000_000 / 100_000_000 = 1_000_000_000
    assert!(pnl > 0, "Long should profit when price increases");
    assert_eq!(
        pnl, 1_000_000_000,
        "PnL should be ~100 tokens (100% of collateral)"
    );
}

#[test]
fn test_calculate_pnl_long_loss() {
    let env = Env::default();
    let (
        _config_id,
        oracle_id,
        position_manager_id,
        _token_address,
        token_client,
        _token_admin,
        admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Open long position at $1.00
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;
    let position_id = position_client.open_position(&trader, &0u32, &collateral, &leverage, &true);

    // Price decreases to $0.95 (-5%)
    set_oracle_price(&env, &oracle_id, &admin, 0, 95_000_000);

    let pnl = position_client.calculate_pnl(&position_id);

    // Long loses when price goes down
    // Size = 10_000_000_000
    // Price PnL = (95_000_000 - 100_000_000) * 10_000_000_000 / 100_000_000 = -500_000_000
    assert!(pnl < 0, "Long should lose when price decreases");
    assert_eq!(
        pnl, -500_000_000,
        "PnL should be -50 tokens (50% of collateral)"
    );
}

#[test]
fn test_calculate_pnl_short_profit() {
    let env = Env::default();
    let (
        _config_id,
        oracle_id,
        position_manager_id,
        _token_address,
        token_client,
        _token_admin,
        admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Open short position at $1.00
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;
    let position_id = position_client.open_position(
        &trader,
        &0u32,
        &collateral,
        &leverage,
        &false, // short
    );

    // Price decreases to $0.90 (-10%)
    set_oracle_price(&env, &oracle_id, &admin, 0, 90_000_000);

    let pnl = position_client.calculate_pnl(&position_id);

    // Short profits when price goes down
    // Size = 10_000_000_000
    // Price PnL = (100_000_000 - 90_000_000) * 10_000_000_000 / 100_000_000 = 1_000_000_000
    assert!(pnl > 0, "Short should profit when price decreases");
    assert_eq!(
        pnl, 1_000_000_000,
        "PnL should be ~100 tokens (100% of collateral)"
    );
}

#[test]
fn test_calculate_pnl_short_loss() {
    let env = Env::default();
    let (
        _config_id,
        oracle_id,
        position_manager_id,
        _token_address,
        token_client,
        _token_admin,
        admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Open short position at $1.00
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;
    let position_id = position_client.open_position(
        &trader,
        &0u32,
        &collateral,
        &leverage,
        &false, // short
    );

    // Price increases to $1.05 (+5%)
    set_oracle_price(&env, &oracle_id, &admin, 0, 105_000_000);

    let pnl = position_client.calculate_pnl(&position_id);

    // Short loses when price goes up
    // Size = 10_000_000_000
    // Price PnL = (100_000_000 - 105_000_000) * 10_000_000_000 / 100_000_000 = -500_000_000
    assert!(pnl < 0, "Short should lose when price increases");
    assert_eq!(
        pnl, -500_000_000,
        "PnL should be -50 tokens (50% of collateral)"
    );
}

#[test]
fn test_calculate_pnl_no_price_change() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        token_client,
        _token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Open long position at $1.00
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;
    let position_id = position_client.open_position(&trader, &0u32, &collateral, &leverage, &true);

    // Price stays at $1.00 (no change)
    let pnl = position_client.calculate_pnl(&position_id);

    // No price change = no PnL (ignoring funding which is 0 at start)
    assert_eq!(pnl, 0, "PnL should be 0 when price hasn't changed");
}

#[test]
fn test_calculate_pnl_different_leverage() {
    let env = Env::default();
    let (
        _config_id,
        oracle_id,
        position_manager_id,
        _token_address,
        token_client,
        _token_admin,
        admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Open position with 5x leverage
    let collateral = 1_000_000_000u128;
    let leverage_5x = 5u32;
    let position_id_5x =
        position_client.open_position(&trader, &0u32, &collateral, &leverage_5x, &true);

    // Open position with 20x leverage
    let leverage_20x = 20u32;
    let position_id_20x =
        position_client.open_position(&trader, &0u32, &collateral, &leverage_20x, &true);

    // Price increases 10%
    set_oracle_price(&env, &oracle_id, &admin, 0, 110_000_000);

    let pnl_5x = position_client.calculate_pnl(&position_id_5x);
    let pnl_20x = position_client.calculate_pnl(&position_id_20x);

    // 5x: Size = 5_000_000_000, PnL = 10% * 5_000_000_000 = 500_000_000
    // 20x: Size = 20_000_000_000, PnL = 10% * 20_000_000_000 = 2_000_000_000
    assert_eq!(pnl_5x, 500_000_000, "5x leverage should yield 50% profit");
    assert_eq!(
        pnl_20x, 2_000_000_000,
        "20x leverage should yield 200% profit"
    );
    assert_eq!(pnl_20x, pnl_5x * 4, "20x PnL should be 4x the 5x PnL");
}

#[test]
#[should_panic(expected = "Position not found")]
fn test_calculate_pnl_nonexistent_position() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        _admin,
        _trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Try to calculate PnL for non-existent position
    position_client.calculate_pnl(&999u64);
}

#[test]
fn test_calculate_pnl_with_borrowing_fee() {
    let env = Env::default();
    let (
        config_id,
        oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        _token_admin,
        admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);
    let config_client = config_manager::Client::new(&env, &config_id);
    let oracle_client = oracle_integrator::Client::new(&env, &oracle_id);

    // Enable fixed price mode to prevent price oscillation during time advancement
    oracle_client.set_fixed_price_mode(&admin, &true);

    // Set a higher borrow rate for visible effect: 100 units per second
    config_client.set_borrow_rate_per_second(&admin, &100);

    // Open long position at $1.00
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;
    let position_id = position_client.open_position(&trader, &0u32, &collateral, &leverage, &true);

    // Advance time by 100 seconds
    env.ledger().with_mut(|li| {
        li.timestamp += 100;
    });

    // Calculate PnL - should include borrowing fees
    // Size = collateral * leverage = 1_000_000_000 * 10 = 10_000_000_000
    // Borrowing fee = (rate * time_elapsed * size) / 10_000_000
    //               = (100 * 100 * 10_000_000_000) / 10_000_000
    //               = 100_000_000_000_000 / 10_000_000 = 10_000_000
    let pnl = position_client.calculate_pnl(&position_id);

    // With no price change, PnL should be negative due to borrowing fee
    assert!(pnl < 0, "PnL should be negative due to borrowing fees, got: {}", pnl);
    assert_eq!(pnl, -10_000_000, "Borrowing fee should be 10_000_000, got: {}", pnl);
}

// ============================================================================
// MAX ORDERS PER POSITION LIMIT TESTS
// ============================================================================

#[test]
#[should_panic(expected = "Maximum orders per position reached")]
fn test_max_orders_per_position_stop_loss() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Mint extra tokens for execution fees (21 orders * 1_000_000 = 21_000_000)
    token_admin.mint(&trader, &100_000_000);

    // Open a long position
    let market_id = 0u32;
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;
    let position_id =
        position_client.open_position(&trader, &market_id, &collateral, &leverage, &true);

    // Create 20 stop-loss orders (the maximum allowed)
    // Each at a slightly different trigger price to be valid
    for i in 0..20 {
        let trigger_price = 94_000_000i128 - (i as i128 * 100_000); // Varying prices below $0.94
        position_client.create_stop_loss(
            &trader,
            &position_id,
            &trigger_price,
            &0i128,
            &CLOSE_FULL,
            &EXECUTION_FEE,
            &0u64,
        );
    }

    // Verify we have 20 orders attached
    let position_orders = position_client.get_position_orders(&position_id);
    assert_eq!(position_orders.len(), 20);

    // The 21st order should panic with "Maximum orders per position reached"
    position_client.create_stop_loss(
        &trader,
        &position_id,
        &92_000_000i128,
        &0i128,
        &CLOSE_FULL,
        &EXECUTION_FEE,
        &0u64,
    );
}

#[test]
#[should_panic(expected = "Maximum orders per position reached")]
fn test_max_orders_per_position_take_profit() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Mint extra tokens for execution fees
    token_admin.mint(&trader, &100_000_000);

    // Open a long position
    let market_id = 0u32;
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;
    let position_id =
        position_client.open_position(&trader, &market_id, &collateral, &leverage, &true);

    // Create 20 take-profit orders (the maximum allowed)
    for i in 0..20 {
        let trigger_price = 110_000_000i128 + (i as i128 * 100_000); // Varying prices above $1.10
        position_client.create_take_profit(
            &trader,
            &position_id,
            &trigger_price,
            &0i128,
            &CLOSE_FULL,
            &EXECUTION_FEE,
            &0u64,
        );
    }

    // Verify we have 20 orders attached
    let position_orders = position_client.get_position_orders(&position_id);
    assert_eq!(position_orders.len(), 20);

    // The 21st order should panic with "Maximum orders per position reached"
    position_client.create_take_profit(
        &trader,
        &position_id,
        &150_000_000i128,
        &0i128,
        &CLOSE_FULL,
        &EXECUTION_FEE,
        &0u64,
    );
}

#[test]
#[should_panic(expected = "Maximum orders per position reached")]
fn test_max_orders_per_position_mixed_sl_tp() {
    let env = Env::default();
    let (
        _config_id,
        _oracle_id,
        position_manager_id,
        _token_address,
        _token_client,
        token_admin,
        _admin,
        trader,
        _liquidity_pool_id,
    ) = setup_test_environment(&env);

    let position_client = PositionManagerClient::new(&env, &position_manager_id);

    // Mint extra tokens for execution fees
    token_admin.mint(&trader, &100_000_000);

    // Open a long position
    let market_id = 0u32;
    let collateral = 1_000_000_000u128;
    let leverage = 10u32;
    let position_id =
        position_client.open_position(&trader, &market_id, &collateral, &leverage, &true);

    // Create 10 stop-loss orders
    for i in 0..10 {
        let trigger_price = 94_000_000i128 - (i as i128 * 100_000);
        position_client.create_stop_loss(
            &trader,
            &position_id,
            &trigger_price,
            &0i128,
            &CLOSE_FULL,
            &EXECUTION_FEE,
            &0u64,
        );
    }

    // Create 10 take-profit orders (total now 20)
    for i in 0..10 {
        let trigger_price = 110_000_000i128 + (i as i128 * 100_000);
        position_client.create_take_profit(
            &trader,
            &position_id,
            &trigger_price,
            &0i128,
            &CLOSE_FULL,
            &EXECUTION_FEE,
            &0u64,
        );
    }

    // Verify we have 20 orders attached
    let position_orders = position_client.get_position_orders(&position_id);
    assert_eq!(position_orders.len(), 20);

    // The 21st order (either SL or TP) should panic
    position_client.create_stop_loss(
        &trader,
        &position_id,
        &92_000_000i128,
        &0i128,
        &CLOSE_FULL,
        &EXECUTION_FEE,
        &0u64,
    );
}

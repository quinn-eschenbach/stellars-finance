use soroban_sdk::{symbol_short, token, Address, Env, Vec};
use soroban_sdk::testutils::Address as _;

// Import contract WASMs for integration testing
pub mod config_manager {
    soroban_sdk::contractimport!(
        file = "../target/wasm32v1-none/release/config_manager.wasm"
    );
}

pub mod oracle_integrator {
    soroban_sdk::contractimport!(
        file = "../target/wasm32v1-none/release/oracle_integrator.wasm"
    );
}

pub mod liquidity_pool {
    soroban_sdk::contractimport!(
        file = "../target/wasm32v1-none/release/liquidity_pool.wasm"
    );
}

pub mod market_manager {
    soroban_sdk::contractimport!(
        file = "../target/wasm32v1-none/release/market_manager.wasm"
    );
}

pub mod position_manager {
    soroban_sdk::contractimport!(
        file = "../target/wasm32v1-none/release/position_manager.wasm"
    );
}

pub mod custom_oracle {
    soroban_sdk::contractimport!(
        file = "../target/wasm32v1-none/release/custom_oracle.wasm"
    );
}

/// Enhanced test environment with multi-user support
pub struct TestEnvironment<'a> {
    pub env: &'a Env,
    pub config_manager_id: Address,
    pub oracle_id: Address,
    pub custom_oracle_id: Address,
    pub position_manager_id: Address,
    pub market_manager_id: Address,
    pub liquidity_pool_id: Address,
    pub token_address: Address,
    pub token_client: token::Client<'a>,
    pub token_admin: token::StellarAssetClient<'a>,
    pub admin: Address,
    pub traders: Vec<Address>,
    pub lps: Vec<Address>,
}

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

/// Setup test environment with configurable number of users
pub fn setup_test_environment<'a>(
    env: &'a Env,
    num_traders: u32,
    num_lps: u32,
    initial_trader_balance: i128,
    initial_lp_balance: i128,
    initial_pool_liquidity: i128,
) -> TestEnvironment<'a> {
    env.mock_all_auths();

    let admin = Address::generate(env);

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

    // Enable test mode with simulated prices
    let mut base_prices = soroban_sdk::Map::new(env);
    base_prices.set(0u32, 100_000_000i128);        // XLM: $0.10
    base_prices.set(1u32, 50_000_000_000_000i128); // BTC: $50,000
    base_prices.set(2u32, 3_000_000_000_000i128);  // ETH: $3,000
    oracle_client.set_test_mode(&admin, &true, &base_prices);

    // Deploy CustomOracle
    let custom_oracle_id = env.register(custom_oracle::WASM, ());
    let custom_oracle_client = custom_oracle::Client::new(env, &custom_oracle_id);
    custom_oracle_client.initialize(&admin);
    custom_oracle_client.add_pusher(&admin, &admin);

    // Push initial prices to custom oracle
    let mut custom_prices = soroban_sdk::Map::new(env);
    custom_prices.set(0u32, 100_000_000i128);        // XLM: $0.10
    custom_prices.set(1u32, 50_000_000_000_000i128); // BTC: $50,000
    custom_prices.set(2u32, 3_000_000_000_000i128);  // ETH: $3,000
    custom_oracle_client.push_prices(&admin, &custom_prices);

    // Deploy MarketManager
    let market_manager_id = env.register(market_manager::WASM, ());
    let market_client = market_manager::Client::new(env, &market_manager_id);
    market_client.initialize(&config_manager_id, &admin);

    // Deploy LiquidityPool
    let liquidity_pool_id = env.register(liquidity_pool::WASM, ());
    let liquidity_client = liquidity_pool::Client::new(env, &liquidity_pool_id);
    liquidity_client.initialize(&admin, &config_manager_id, &token_client.address);

    // Deploy PositionManager
    let position_manager_id = env.register(position_manager::WASM, ());
    let position_client = position_manager::Client::new(env, &position_manager_id);
    position_client.initialize(&admin, &config_manager_id);

    // Configure ConfigManager with contract addresses
    config_client.set_oracle_integrator(&admin, &oracle_id);
    config_client.set_custom_oracle(&admin, &custom_oracle_id);
    config_client.set_market_manager(&admin, &market_manager_id);
    config_client.set_liquidity_pool(&admin, &liquidity_pool_id);
    config_client.set_position_manager(&admin, &position_manager_id);
    config_client.set_token(&admin, &token_client.address);

    // Set PositionManager in MarketManager (for authorization)
    market_client.set_position_manager(&admin, &position_manager_id);

    // Set PositionManager in LiquidityPool (for authorization)
    liquidity_client.set_position_manager(&admin, &position_manager_id);

    // Create test markets
    market_client.create_market(&admin, &0u32, &symbol_short!("XLM"), &1_000_000_000_000u128, &10000i128); // XLM-PERP
    market_client.create_market(&admin, &1u32, &symbol_short!("BTC"), &1_000_000_000_000u128, &10000i128); // BTC-PERP
    market_client.create_market(&admin, &2u32, &symbol_short!("ETH"), &1_000_000_000_000u128, &10000i128); // ETH-PERP

    // Create multiple traders
    let mut traders = Vec::new(env);
    for _ in 0..num_traders {
        let trader = Address::generate(env);
        token_admin.mint(&trader, &initial_trader_balance);
        traders.push_back(trader);
    }

    // Create multiple LPs
    let mut lps = Vec::new(env);
    for _ in 0..num_lps {
        let lp = Address::generate(env);
        token_admin.mint(&lp, &initial_lp_balance);
        lps.push_back(lp);
    }

    // Initial pool liquidity deposit from admin
    token_admin.mint(&admin, &initial_pool_liquidity);
    liquidity_client.deposit(&admin, &initial_pool_liquidity);

    TestEnvironment {
        env,
        config_manager_id,
        oracle_id,
        custom_oracle_id,
        position_manager_id,
        market_manager_id,
        liquidity_pool_id,
        token_address: token_client.address.clone(),
        token_client,
        token_admin,
        admin,
        traders,
        lps,
    }
}

/// Quick setup for focused tests: 5 traders, 2 LPs
pub fn setup_focused_test<'a>(env: &'a Env) -> TestEnvironment<'a> {
    setup_test_environment(
        env,
        5,                   // num_traders
        2,                   // num_lps
        10_000_000_000,      // 10,000 tokens per trader
        100_000_000_000,     // 100,000 tokens per LP
        1_000_000_000_000,   // 1M tokens initial pool liquidity
    )
}

/// Setup for stress tests: 20 traders, 5 LPs
pub fn setup_stress_test<'a>(env: &'a Env) -> TestEnvironment<'a> {
    setup_test_environment(
        env,
        20,                   // num_traders
        5,                    // num_lps
        10_000_000_000,       // 10,000 tokens per trader
        100_000_000_000,      // 100,000 tokens per LP
        10_000_000_000_000,   // 10M tokens initial pool liquidity
    )
}

// ============================================================================
// ORDER TEST HELPERS
// ============================================================================

/// Standard execution fee for order tests
pub const ORDER_EXECUTION_FEE: u128 = 1_000_000; // 0.1 tokens

/// Set oracle price for a specific market (used to trigger orders)
pub fn set_oracle_price(
    env: &Env,
    oracle_id: &Address,
    admin: &Address,
    market_id: u32,
    new_price: i128,
) {
    let oracle_client = oracle_integrator::Client::new(env, oracle_id);
    let mut base_prices = soroban_sdk::Map::new(env);

    // Set all markets to their defaults, then override the target market
    base_prices.set(0u32, 100_000_000i128);        // XLM: $1.00
    base_prices.set(1u32, 50_000_000_000_000i128); // BTC: $50,000
    base_prices.set(2u32, 3_000_000_000_000i128);  // ETH: $3,000

    // Override target market
    base_prices.set(market_id, new_price);

    oracle_client.set_test_mode(admin, &true, &base_prices);
}

/// Create a limit order with standard parameters
pub fn create_test_limit_order(
    env: &Env,
    position_client: &position_manager::Client,
    trader: &Address,
    market_id: u32,
    trigger_price: i128,
    collateral: u128,
    leverage: u32,
    is_long: bool,
) -> u64 {
    position_client.create_limit_order(
        trader,
        &market_id,
        &trigger_price,
        &0i128, // No slippage limit
        &collateral,
        &leverage,
        &is_long,
        &ORDER_EXECUTION_FEE,
        &0u64, // No expiry
    )
}

/// Create a stop-loss order with standard parameters
pub fn create_test_stop_loss(
    env: &Env,
    position_client: &position_manager::Client,
    trader: &Address,
    position_id: u64,
    trigger_price: i128,
    close_percentage: u32,
) -> u64 {
    position_client.create_stop_loss(
        trader,
        &position_id,
        &trigger_price,
        &0i128, // No slippage limit
        &close_percentage,
        &ORDER_EXECUTION_FEE,
        &0u64, // No expiry
    )
}

/// Create a take-profit order with standard parameters
pub fn create_test_take_profit(
    env: &Env,
    position_client: &position_manager::Client,
    trader: &Address,
    position_id: u64,
    trigger_price: i128,
    close_percentage: u32,
) -> u64 {
    position_client.create_take_profit(
        trader,
        &position_id,
        &trigger_price,
        &0i128, // No slippage limit
        &close_percentage,
        &ORDER_EXECUTION_FEE,
        &0u64, // No expiry
    )
}

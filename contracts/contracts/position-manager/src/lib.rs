#![no_std]

//! # Position Manager Contract
//!
//! Core position lifecycle management for the Stellars Finance perpetual trading protocol.
//! Handles opening, closing, modifying, and liquidating leveraged positions, plus advanced
//! order types (limit orders, stop-loss, take-profit).
//!
//! ## Key Features
//! - **Position Lifecycle**: Open, close, increase, and decrease leveraged positions
//! - **Liquidations**: Force-close undercollateralized positions with keeper incentives
//! - **Advanced Orders**: Limit orders to open at target price, SL/TP to manage risk
//! - **PnL Calculation**: Comprehensive PnL including price movement, funding, and fees
//!
//! ## Position Structure
//! Each position tracks:
//! - Trader address and market ID
//! - Collateral and size (notional value = collateral × leverage)
//! - Direction (long/short) and entry price
//! - Funding rate snapshots for accurate funding payment calculation
//! - Liquidation price (automatically calculated)
//!
//! ## Order Types
//! - **Limit Order**: Opens a new position when price reaches trigger level
//! - **Stop-Loss**: Closes position to limit losses when price moves against you
//! - **Take-Profit**: Closes position to secure gains when price target is reached
//!
//! ## PnL Components
//! 1. **Price PnL**: Profit/loss from price movement
//! 2. **Funding Payments**: Periodic payments based on market imbalance
//! 3. **Borrowing Fees**: Time-based fees for leverage (not yet implemented)
//!
//! ## Liquidation
//! Positions are liquidatable when collateral ratio falls below maintenance margin.
//! Keepers receive 60% of liquidation fee as incentive, 40% goes to the pool.
//!
//! ## Usage
//! - Traders call position functions directly
//! - Keeper bots call `execute_order()` and `liquidate_position()`

use soroban_sdk::{contract, contractevent, contractimpl, contracttype, log, token, Address, Env};

mod config_manager {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/config_manager.wasm");
}

mod oracle_integrator {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32v1-none/release/oracle_integrator.wasm"
    );
}

mod liquidity_pool {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/liquidity_pool.wasm");
}

mod market_manager {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/market_manager.wasm");
}

#[contract]
pub struct PositionManager;

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Position {
    pub trader: Address,
    pub market_id: u32, // NEW: which market (0=XLM, 1=BTC, 2=ETH)
    pub collateral: u128,
    pub size: u128,
    pub is_long: bool,
    pub entry_price: i128,         // Changed to i128
    pub entry_funding_long: i128,  // NEW: cumulative funding snapshot (long side)
    pub entry_funding_short: i128, // NEW: cumulative funding snapshot (short side)
    pub last_interaction: u64,     // NEW: timestamp for borrowing fee calculation
    pub liquidation_price: i128,   // NEW: price at which position is liquidatable
}

// Events
#[contractevent]
pub struct PositionOpenedEvent {
    pub position_id: u64,
    pub trader: Address,
    pub market_id: u32,
    pub collateral: u128,
    pub size: u128,
    pub leverage: u32,
    pub is_long: bool,
    pub entry_price: u128,
}

#[contractevent]
pub struct PositionClosedEvent {
    pub position_id: u64,
    pub trader: Address,
    pub pnl: i128,
}

#[contractevent]
pub struct PositionModifiedEvent {
    pub position_id: u64,
    pub trader: Address,
    pub new_collateral: u128,
    pub new_size: u128,
    pub new_liquidation_price: i128,
}

#[contractevent]
pub struct PositionLiquidatedEvent {
    pub position_id: u64,
    pub trader: Address,
    pub liquidator: Address,
    pub liquidation_price: i128,
    pub liquidation_reward: u128,
}

// ============================================================================
// ORDER TYPES - Limit, Stop-Loss, Take-Profit
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum OrderType {
    Limit,      // Open new position when price reaches target
    StopLoss,   // Close existing position to limit losses
    TakeProfit, // Close existing position to secure gains
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum OrderCancelReason {
    UserCancelled,
    PositionClosed,
    PositionLiquidated,
    Expired,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Order {
    pub order_id: u64,
    pub order_type: OrderType,
    pub trader: Address,
    pub market_id: u32,
    pub position_id: u64,       // 0 for Limit orders, position_id for SL/TP
    pub trigger_price: i128,    // Price that triggers execution (1e7 scaled)
    pub acceptable_price: i128, // Slippage protection (0 = no limit)
    pub collateral: u128,       // For Limit orders only
    pub size: u128,             // Position size (Limit) or size to close (SL/TP)
    pub leverage: u32,          // For Limit orders only
    pub is_long: bool,
    pub close_percentage: u32, // For SL/TP: 10000 = 100%
    pub execution_fee: u128,   // Fee paid to keeper
    pub expiration: u64,       // 0 = no expiry
    pub created_at: u64,
}

// Order Events
#[contractevent]
pub struct OrderCreatedEvent {
    pub order_id: u64,
    pub order_type: OrderType,
    pub trader: Address,
    pub market_id: u32,
    pub position_id: u64,
    pub trigger_price: i128,
    pub size: u128,
    pub is_long: bool,
    pub expiration: u64,
}

#[contractevent]
pub struct OrderExecutedEvent {
    pub order_id: u64,
    pub order_type: OrderType,
    pub trader: Address,
    pub keeper: Address,
    pub execution_price: i128,
    pub position_id: u64,
    pub pnl: i128,
    pub execution_fee: u128,
}

#[contractevent]
pub struct OrderCancelledEvent {
    pub order_id: u64,
    pub order_type: OrderType,
    pub trader: Address,
    pub reason: OrderCancelReason,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Position(u64),
    NextPositionId,
    ConfigManager,
    UserPositions(Address), // Maps user address to Vec<u64> of their open position IDs
    // Order-related keys
    Order(u64),                // Individual order storage
    NextOrderId,               // Auto-increment counter for order IDs
    UserOrders(Address),       // User -> Vec<order_ids>
    PositionOrders(u64),       // Position -> Vec<attached SL/TP order_ids>
    ActiveOrdersByMarket(u32), // Market -> Vec<order_ids> for keeper queries
    MinExecutionFee,           // Minimum fee for keepers
}

// Helper functions for storage

/// Get the ConfigManager address from storage
fn get_config_manager(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::ConfigManager)
        .expect("ConfigManager not initialized")
}

/// Get the token contract address from ConfigManager
fn get_token(env: &Env) -> Address {
    let config_manager = get_config_manager(env);
    let config_client = config_manager::Client::new(env, &config_manager);
    config_client.token()
}

/// Get the OracleIntegrator address from ConfigManager
fn get_oracle(env: &Env) -> Address {
    let config_manager = get_config_manager(env);
    let config_client = config_manager::Client::new(env, &config_manager);
    config_client.oracle_integrator()
}

/// Get the LiquidityPool address from ConfigManager
fn get_liquidity_pool(env: &Env) -> Address {
    let config_manager = get_config_manager(env);
    let config_client = config_manager::Client::new(env, &config_manager);
    config_client.liquidity_pool()
}

/// Get the MarketManager address from ConfigManager
fn get_market_manager(env: &Env) -> Address {
    let config_manager = get_config_manager(env);
    let config_client = config_manager::Client::new(env, &config_manager);
    config_client.market_manager()
}

/// Get a position from storage
fn get_position(env: &Env, position_id: u64) -> Position {
    env.storage()
        .persistent()
        .get(&DataKey::Position(position_id))
        .expect("Position not found")
}

/// Store a position in persistent storage with TTL extension
fn set_position(env: &Env, position_id: u64, position: &Position) {
    env.storage()
        .persistent()
        .set(&DataKey::Position(position_id), position);
    env.storage().persistent().extend_ttl(
        &DataKey::Position(position_id),
        TTL_LEDGERS,
        TTL_LEDGERS,
    );
}

/// Delete a position from storage
fn remove_position(env: &Env, position_id: u64) {
    env.storage()
        .persistent()
        .remove(&DataKey::Position(position_id));
}

/// Get the next position ID (starts at 1 since 0 means "no position" for orders)
fn get_next_position_id(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::NextPositionId)
        .unwrap_or(1)
}

/// Increment and return the next position ID
fn increment_position_id(env: &Env) -> u64 {
    let next_id = get_next_position_id(env);
    env.storage()
        .instance()
        .set(&DataKey::NextPositionId, &(next_id + 1));
    next_id
}

/// Get all open position IDs for a user
fn get_user_positions(env: &Env, trader: &Address) -> soroban_sdk::Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::UserPositions(trader.clone()))
        .unwrap_or(soroban_sdk::Vec::new(env))
}

/// Add a position ID to a user's list of open positions
fn add_user_position(env: &Env, trader: &Address, position_id: u64) {
    let mut user_positions = get_user_positions(env, trader);
    user_positions.push_back(position_id);
    let key = DataKey::UserPositions(trader.clone());
    env.storage().persistent().set(&key, &user_positions);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_LEDGERS, TTL_LEDGERS);
}

/// Remove a position ID from a user's list of open positions
fn remove_user_position(env: &Env, trader: &Address, position_id: u64) {
    let user_positions = get_user_positions(env, trader);

    // Filter out the position_id we want to remove
    let mut new_positions = soroban_sdk::Vec::new(env);
    for i in 0..user_positions.len() {
        let id = user_positions.get(i).unwrap();
        if id != position_id {
            new_positions.push_back(id);
        }
    }

    let key = DataKey::UserPositions(trader.clone());
    env.storage().persistent().set(&key, &new_positions);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_LEDGERS, TTL_LEDGERS);
}

// ============================================================================
// ORDER STORAGE HELPERS
// ============================================================================

const TTL_LEDGERS: u32 = 100_000; // ~14 days for positions and orders

/// Maximum number of SL/TP orders that can be attached to a single position.
/// This prevents gas exhaustion attacks where a trader creates many orders to block liquidation.
const MAX_ORDERS_PER_POSITION: u32 = 20;

/// Get an order from storage
fn get_order_from_storage(env: &Env, order_id: u64) -> Order {
    env.storage()
        .persistent()
        .get(&DataKey::Order(order_id))
        .expect("Order not found")
}

/// Check if an order exists
fn order_exists(env: &Env, order_id: u64) -> bool {
    env.storage().persistent().has(&DataKey::Order(order_id))
}

/// Store an order in persistent storage with TTL extension
fn set_order(env: &Env, order_id: u64, order: &Order) {
    env.storage()
        .persistent()
        .set(&DataKey::Order(order_id), order);
    env.storage().persistent().extend_ttl(
        &DataKey::Order(order_id),
        TTL_LEDGERS,
        TTL_LEDGERS,
    );
}

/// Delete an order from storage
fn remove_order(env: &Env, order_id: u64) {
    env.storage().persistent().remove(&DataKey::Order(order_id));
}

/// Get the next order ID (starts at 1 for consistency with position IDs)
fn get_next_order_id(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::NextOrderId)
        .unwrap_or(1)
}

/// Increment and return the next order ID
fn increment_order_id(env: &Env) -> u64 {
    let next_id = get_next_order_id(env);
    env.storage()
        .instance()
        .set(&DataKey::NextOrderId, &(next_id + 1));
    next_id
}

/// Get all order IDs for a user
fn get_user_orders_list(env: &Env, trader: &Address) -> soroban_sdk::Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::UserOrders(trader.clone()))
        .unwrap_or(soroban_sdk::Vec::new(env))
}

/// Add an order ID to a user's list of orders
fn add_user_order(env: &Env, trader: &Address, order_id: u64) {
    let mut orders = get_user_orders_list(env, trader);
    orders.push_back(order_id);
    let key = DataKey::UserOrders(trader.clone());
    env.storage().persistent().set(&key, &orders);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_LEDGERS, TTL_LEDGERS);
}

/// Remove an order ID from a user's list of orders
fn remove_user_order(env: &Env, trader: &Address, order_id: u64) {
    let orders = get_user_orders_list(env, trader);
    let mut new_orders = soroban_sdk::Vec::new(env);
    for i in 0..orders.len() {
        let id = orders.get(i).unwrap();
        if id != order_id {
            new_orders.push_back(id);
        }
    }
    let key = DataKey::UserOrders(trader.clone());
    env.storage().persistent().set(&key, &new_orders);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_LEDGERS, TTL_LEDGERS);
}

/// Get all order IDs attached to a position (SL/TP orders)
fn get_position_orders_list(env: &Env, position_id: u64) -> soroban_sdk::Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::PositionOrders(position_id))
        .unwrap_or(soroban_sdk::Vec::new(env))
}

/// Add an order ID to a position's attached orders
fn add_position_order(env: &Env, position_id: u64, order_id: u64) {
    let mut orders = get_position_orders_list(env, position_id);
    orders.push_back(order_id);
    let key = DataKey::PositionOrders(position_id);
    env.storage().persistent().set(&key, &orders);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_LEDGERS, TTL_LEDGERS);
}

/// Remove an order ID from a position's attached orders
fn remove_position_order(env: &Env, position_id: u64, order_id: u64) {
    let orders = get_position_orders_list(env, position_id);
    let mut new_orders = soroban_sdk::Vec::new(env);
    for i in 0..orders.len() {
        let id = orders.get(i).unwrap();
        if id != order_id {
            new_orders.push_back(id);
        }
    }
    let key = DataKey::PositionOrders(position_id);
    env.storage().persistent().set(&key, &new_orders);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_LEDGERS, TTL_LEDGERS);
}

/// Clear all orders attached to a position
fn clear_position_orders(env: &Env, position_id: u64) {
    env.storage()
        .persistent()
        .remove(&DataKey::PositionOrders(position_id));
}

/// Get all active order IDs for a market (for keeper queries)
fn get_market_orders_list(env: &Env, market_id: u32) -> soroban_sdk::Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::ActiveOrdersByMarket(market_id))
        .unwrap_or(soroban_sdk::Vec::new(env))
}

/// Add an order ID to a market's active orders
fn add_market_order(env: &Env, market_id: u32, order_id: u64) {
    let mut orders = get_market_orders_list(env, market_id);
    orders.push_back(order_id);
    let key = DataKey::ActiveOrdersByMarket(market_id);
    env.storage().persistent().set(&key, &orders);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_LEDGERS, TTL_LEDGERS);
}

/// Remove an order ID from a market's active orders
fn remove_market_order(env: &Env, market_id: u32, order_id: u64) {
    let orders = get_market_orders_list(env, market_id);
    let mut new_orders = soroban_sdk::Vec::new(env);
    for i in 0..orders.len() {
        let id = orders.get(i).unwrap();
        if id != order_id {
            new_orders.push_back(id);
        }
    }
    let key = DataKey::ActiveOrdersByMarket(market_id);
    env.storage().persistent().set(&key, &new_orders);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_LEDGERS, TTL_LEDGERS);
}

/// Get minimum execution fee
fn get_min_execution_fee(env: &Env) -> u128 {
    env.storage()
        .instance()
        .get(&DataKey::MinExecutionFee)
        .unwrap_or(1_000_000) // Default: 0.1 tokens (assuming 7 decimals)
}

/// Validate execution fee meets minimum
fn validate_execution_fee(env: &Env, fee: u128) {
    let min_fee = get_min_execution_fee(env);
    if fee < min_fee {
        panic!("Execution fee below minimum");
    }
}

/// Check if order trigger condition is met
fn check_order_trigger(order: &Order, current_price: i128) -> bool {
    match order.order_type {
        OrderType::Limit => {
            if order.is_long {
                // Buy limit: trigger when price falls to or below trigger
                current_price <= order.trigger_price
            } else {
                // Sell limit: trigger when price rises to or above trigger
                current_price >= order.trigger_price
            }
        }
        OrderType::StopLoss => {
            if order.is_long {
                // Long SL: trigger when price falls to or below trigger
                current_price <= order.trigger_price
            } else {
                // Short SL: trigger when price rises to or above trigger
                current_price >= order.trigger_price
            }
        }
        OrderType::TakeProfit => {
            if order.is_long {
                // Long TP: trigger when price rises to or above trigger
                current_price >= order.trigger_price
            } else {
                // Short TP: trigger when price falls to or below trigger
                current_price <= order.trigger_price
            }
        }
    }
}

/// Check if current price is within acceptable slippage
fn check_acceptable_price(order: &Order, current_price: i128) -> bool {
    if order.acceptable_price == 0 {
        return true; // No slippage limit
    }
    match order.order_type {
        OrderType::Limit => {
            if order.is_long {
                // Buying: current price should not exceed acceptable
                current_price <= order.acceptable_price
            } else {
                // Selling: current price should not be below acceptable
                current_price >= order.acceptable_price
            }
        }
        OrderType::StopLoss | OrderType::TakeProfit => {
            if order.is_long {
                // Closing long: receiving price should not be below acceptable
                current_price >= order.acceptable_price
            } else {
                // Closing short: price should not exceed acceptable
                current_price <= order.acceptable_price
            }
        }
    }
}

/// Clean up order from all storage locations and emit cancel event
fn cleanup_order(env: &Env, order: &Order, reason: OrderCancelReason) {
    remove_order(env, order.order_id);
    remove_user_order(env, &order.trader, order.order_id);
    remove_market_order(env, order.market_id, order.order_id);

    // Remove from position orders if SL/TP
    if order.position_id > 0 {
        remove_position_order(env, order.position_id, order.order_id);
    }

    // Emit event
    OrderCancelledEvent {
        order_id: order.order_id,
        order_type: order.order_type.clone(),
        trader: order.trader.clone(),
        reason,
    }
    .publish(env);
}

/// Cancel all orders attached to a position (used when position closes)
fn cancel_position_attached_orders(env: &Env, position_id: u64, reason: OrderCancelReason) {
    let order_ids = get_position_orders_list(env, position_id);

    for i in 0..order_ids.len() {
        let order_id = order_ids.get(i).unwrap();
        if order_exists(env, order_id) {
            let order = get_order_from_storage(env, order_id);

            // Refund execution fee to trader
            let token = get_token(env);
            let token_client = token::Client::new(env, &token);
            token_client.transfer(
                &env.current_contract_address(),
                &order.trader,
                &(order.execution_fee as i128),
            );

            // Clean up order storage
            remove_order(env, order_id);
            remove_user_order(env, &order.trader, order_id);
            remove_market_order(env, order.market_id, order_id);

            // Emit cancel event
            OrderCancelledEvent {
                order_id,
                order_type: order.order_type.clone(),
                trader: order.trader.clone(),
                reason: reason.clone(),
            }
            .publish(env);
        }
    }

    // Clear position orders mapping
    clear_position_orders(env, position_id);
}

/// Execute a limit order - opens a new position
fn execute_limit_order(env: &Env, order: &Order, _current_price: i128) -> i128 {
    let pool_address = get_liquidity_pool(env);

    // Transfer escrowed collateral from contract to pool
    let token = get_token(env);
    let token_client = token::Client::new(env, &token);
    token_client.transfer(
        &env.current_contract_address(),
        &pool_address,
        &(order.collateral as i128),
    );

    // Get oracle for entry price
    let oracle_address = get_oracle(env);
    let oracle_client = oracle_integrator::Client::new(env, &oracle_address);
    let entry_price = oracle_client.get_price(&order.market_id);

    // Check market can accept position
    let market_manager = get_market_manager(env);
    let market_client = market_manager::Client::new(env, &market_manager);

    if !market_client.can_open_position(&order.market_id, &order.is_long, &order.size) {
        panic!("Cannot open position - market paused or OI limit reached");
    }

    // Get funding snapshots
    let entry_funding_long = market_client.get_cumulative_funding(&order.market_id, &true);
    let entry_funding_short = market_client.get_cumulative_funding(&order.market_id, &false);

    // Generate position ID
    let position_id = increment_position_id(env);

    // Check pool utilization
    let pool_client = liquidity_pool::Client::new(env, &pool_address);
    let available = pool_client.get_available_liquidity();
    let reserved = pool_client.get_reserved_liquidity();

    let config_manager = get_config_manager(env);
    let config_client = config_manager::Client::new(env, &config_manager);
    let max_utilization = config_client.max_utilization_ratio();

    if available <= 0 {
        panic!("no available liquidity");
    }

    let total_balance = available as u128 + reserved;
    let reserved_after = reserved + order.size;

    if total_balance > 0 {
        let utilization_after = ((reserved_after * 10000) / total_balance) as i128;
        if utilization_after > max_utilization {
            panic!("Position would exceed max pool utilization");
        }
    }

    // Record collateral in pool (already transferred above) and reserve liquidity
    pool_client.record_position_collateral(
        &env.current_contract_address(),
        &position_id,
        &order.collateral,
    );
    pool_client.reserve_liquidity(
        &env.current_contract_address(),
        &position_id,
        &order.size,
        &order.collateral,
    );

    // Calculate liquidation price
    let liquidation_price =
        calculate_liquidation_price(entry_price, order.collateral, order.size, order.is_long);

    // Create position
    let position = Position {
        trader: order.trader.clone(),
        market_id: order.market_id,
        collateral: order.collateral,
        size: order.size,
        is_long: order.is_long,
        entry_price,
        entry_funding_long,
        entry_funding_short,
        last_interaction: env.ledger().timestamp(),
        liquidation_price,
    };

    // Store position
    set_position(env, position_id, &position);
    add_user_position(env, &order.trader, position_id);

    // Update market open interest
    market_client.update_open_interest(
        &env.current_contract_address(),
        &order.market_id,
        &order.is_long,
        &(order.size as i128),
    );

    // Emit position opened event
    PositionOpenedEvent {
        position_id,
        trader: order.trader.clone(),
        market_id: order.market_id,
        collateral: order.collateral,
        size: order.size,
        leverage: order.leverage,
        is_long: order.is_long,
        entry_price: entry_price as u128,
    }
    .publish(env);

    position_id as i128
}

/// Execute a stop-loss or take-profit order - closes (partially or fully) an existing position
fn execute_sl_tp_order(env: &Env, order: &Order, current_price: i128) -> i128 {
    // Check position still exists
    if !env
        .storage()
        .persistent()
        .has(&DataKey::Position(order.position_id))
    {
        panic!("Position no longer exists");
    }

    let position = get_position(env, order.position_id);

    // Verify position ownership hasn't changed
    if position.trader != order.trader {
        panic!("Position ownership changed");
    }

    // Calculate actual size to close based on current position (may have changed)
    let size_to_close = if order.close_percentage == 10000 {
        position.size
    } else {
        let calculated = (position.size * order.close_percentage as u128) / 10000;
        // Don't close more than position size
        if calculated > position.size {
            position.size
        } else {
            calculated
        }
    };

    // Close position (partial or full)
    // Pass the executing order_id so we don't refund its fee (keeper gets it instead)
    if size_to_close >= position.size {
        // Full close - use close_position logic
        execute_full_close(
            env,
            order.position_id,
            &position,
            current_price,
            Some(order.order_id),
        )
    } else {
        // Partial close - use decrease_position logic
        execute_partial_close(
            env,
            order.position_id,
            &position,
            size_to_close,
            current_price,
            Some(order.order_id),
        )
    }
}

/// Execute a full position close (internal, for order execution)
/// `executing_order_id` is the order currently being executed - skip refunding its fee
fn execute_full_close(
    env: &Env,
    position_id: u64,
    position: &Position,
    current_price: i128,
    executing_order_id: Option<u64>,
) -> i128 {
    // Calculate comprehensive PnL
    let pnl = calculate_pnl(env, position, current_price);

    // Get liquidity pool
    let pool_address = get_liquidity_pool(env);
    let pool_client = liquidity_pool::Client::new(env, &pool_address);

    // Release reserved liquidity
    pool_client.release_liquidity(
        &env.current_contract_address(),
        &position_id,
        &position.size,
    );

    // Settle PnL with pool and withdraw collateral to trader
    let collateral_i128 = position.collateral as i128;
    let final_amount = collateral_i128 + pnl;

    if pnl >= 0 {
        pool_client.withdraw_position_collateral(
            &env.current_contract_address(),
            &position_id,
            &position.trader,
            &position.collateral,
        );
        if pnl > 0 {
            pool_client.settle_trader_pnl(&env.current_contract_address(), &position.trader, &pnl);
        }
    } else {
        let withdrawal_amount = if final_amount > 0 {
            final_amount as u128
        } else {
            0u128
        };
        pool_client.withdraw_position_collateral(
            &env.current_contract_address(),
            &position_id,
            &position.trader,
            &withdrawal_amount,
        );
    }

    // Update open interest in MarketManager
    let market_manager = get_market_manager(env);
    let market_client = market_manager::Client::new(env, &market_manager);
    let size_decrease = -(position.size as i128);
    market_client.update_open_interest(
        &env.current_contract_address(),
        &position.market_id,
        &position.is_long,
        &size_decrease,
    );

    // Cancel any other attached orders (except the one being executed)
    // The executing order is cleaned up by the caller and its fee goes to keeper
    let order_ids = get_position_orders_list(env, position_id);
    for i in 0..order_ids.len() {
        let other_order_id = order_ids.get(i).unwrap();
        // Skip the currently executing order - its fee goes to the keeper
        if let Some(exec_id) = executing_order_id {
            if other_order_id == exec_id {
                continue;
            }
        }
        if order_exists(env, other_order_id) {
            let other_order = get_order_from_storage(env, other_order_id);

            // Refund execution fee
            let token = get_token(env);
            let token_client = token::Client::new(env, &token);
            token_client.transfer(
                &env.current_contract_address(),
                &other_order.trader,
                &(other_order.execution_fee as i128),
            );

            // Clean up
            remove_order(env, other_order_id);
            remove_user_order(env, &other_order.trader, other_order_id);
            remove_market_order(env, other_order.market_id, other_order_id);

            OrderCancelledEvent {
                order_id: other_order_id,
                order_type: other_order.order_type,
                trader: other_order.trader,
                reason: OrderCancelReason::PositionClosed,
            }
            .publish(env);
        }
    }
    clear_position_orders(env, position_id);

    // Delete position from storage
    remove_position(env, position_id);
    remove_user_position(env, &position.trader, position_id);

    // Emit position closed event
    PositionClosedEvent {
        position_id,
        trader: position.trader.clone(),
        pnl,
    }
    .publish(env);

    pnl
}

/// Execute a partial position close (internal, for order execution)
/// `_executing_order_id` is unused for partial close but kept for API consistency
fn execute_partial_close(
    env: &Env,
    position_id: u64,
    position: &Position,
    size_to_reduce: u128,
    current_price: i128,
    _executing_order_id: Option<u64>,
) -> i128 {
    let pool_address = get_liquidity_pool(env);
    let pool_client = liquidity_pool::Client::new(env, &pool_address);

    // Calculate proportional PnL for the size being closed
    let total_pnl = calculate_pnl(env, position, current_price);
    let proportion = (size_to_reduce as i128 * 10000) / (position.size as i128);
    let realized_pnl = (total_pnl * proportion) / 10000;

    // Realize PnL: adjust collateral
    let collateral_i128 = position.collateral as i128;
    let new_collateral_i128 = collateral_i128 + realized_pnl;

    if new_collateral_i128 <= 0 {
        panic!("Position underwater - would fully close");
    }

    // Settle realized PnL with trader
    if realized_pnl > 0 {
        pool_client.settle_trader_pnl(
            &env.current_contract_address(),
            &position.trader,
            &realized_pnl,
        );
    } else if realized_pnl < 0 {
        let loss_amount = (-realized_pnl) as u128;
        pool_client.withdraw_position_collateral(
            &env.current_contract_address(),
            &position_id,
            &pool_address,
            &loss_amount,
        );
    }

    // Release reserved liquidity
    pool_client.release_liquidity(
        &env.current_contract_address(),
        &position_id,
        &size_to_reduce,
    );

    // Update MarketManager open interest
    let market_manager = get_market_manager(env);
    let market_client = market_manager::Client::new(env, &market_manager);
    let size_decrease = -(size_to_reduce as i128);
    market_client.update_open_interest(
        &env.current_contract_address(),
        &position.market_id,
        &position.is_long,
        &size_decrease,
    );

    // Update position
    let mut updated_position = position.clone();
    updated_position.collateral = new_collateral_i128 as u128;
    updated_position.size = position.size - size_to_reduce;
    updated_position.entry_funding_long =
        market_client.get_cumulative_funding(&position.market_id, &true);
    updated_position.entry_funding_short =
        market_client.get_cumulative_funding(&position.market_id, &false);
    updated_position.liquidation_price = calculate_liquidation_price(
        position.entry_price,
        updated_position.collateral,
        updated_position.size,
        position.is_long,
    );
    updated_position.last_interaction = env.ledger().timestamp();

    set_position(env, position_id, &updated_position);

    // Update attached order sizes based on new position size
    let order_ids = get_position_orders_list(env, position_id);
    for i in 0..order_ids.len() {
        let order_id = order_ids.get(i).unwrap();
        if order_exists(env, order_id) {
            let mut order = get_order_from_storage(env, order_id);
            // Recalculate size based on percentage and new position size
            order.size = (updated_position.size * order.close_percentage as u128) / 10000;
            set_order(env, order_id, &order);
        }
    }

    // Emit position modified event
    PositionModifiedEvent {
        position_id,
        trader: position.trader.clone(),
        new_collateral: updated_position.collateral,
        new_size: updated_position.size,
        new_liquidation_price: updated_position.liquidation_price,
    }
    .publish(env);

    realized_pnl
}

/// Validate leverage is within configured limits
fn validate_leverage(env: &Env, leverage: u32) {
    let config_manager = get_config_manager(env);
    let config_client = config_manager::Client::new(env, &config_manager);

    let min_leverage = config_client.min_leverage() as u32;
    let max_leverage = config_client.max_leverage() as u32;

    if leverage < min_leverage {
        panic!("Leverage too low");
    }
    if leverage > max_leverage {
        panic!("Leverage too high");
    }
}

/// Validate position size meets minimum requirement
fn validate_position_size(env: &Env, size: u128) {
    let config_manager = get_config_manager(env);
    let config_client = config_manager::Client::new(env, &config_manager);

    let min_position_size = config_client.min_position_size() as u128;
    if size < min_position_size {
        panic!("Position size too small");
    }
}

/// Calculate liquidation price for a position
///
/// # Formula
/// - For longs: liquidation_price = entry_price * (1 - (collateral / size) + maintenance_margin)
/// - For shorts: liquidation_price = entry_price * (1 + (collateral / size) - maintenance_margin)
///
/// Where maintenance_margin = 0.01 (1%)
///
/// # Arguments
/// * `entry_price` - Entry price of the position (scaled by 1e7)
/// * `collateral` - Collateral amount (scaled by 1e7)
/// * `size` - Position size (scaled by 1e7)
/// * `is_long` - True for long positions, false for short positions
///
/// # Returns
/// Liquidation price (scaled by 1e7)
fn calculate_liquidation_price(
    entry_price: i128,
    collateral: u128,
    size: u128,
    is_long: bool,
) -> i128 {
    // Maintenance margin = 1% = 100 bps
    const MAINTENANCE_MARGIN_BPS: i128 = 100;
    const BPS_DIVISOR: i128 = 10000;

    let collateral_i128 = collateral as i128;
    let size_i128 = size as i128;

    if size_i128 == 0 {
        panic!("Cannot calculate liquidation price for zero size");
    }

    // Calculate collateral ratio in basis points: (collateral / size) * 10000
    let collateral_ratio_bps = (collateral_i128 * BPS_DIVISOR) / size_i128;

    if is_long {
        // For longs: liquidation_price = entry_price * (1 - collateral_ratio + maintenance_margin)
        // = entry_price * (10000 - collateral_ratio_bps + maintenance_margin_bps) / 10000
        let multiplier_bps = BPS_DIVISOR - collateral_ratio_bps + MAINTENANCE_MARGIN_BPS;
        (entry_price * multiplier_bps) / BPS_DIVISOR
    } else {
        // For shorts: liquidation_price = entry_price * (1 + collateral_ratio - maintenance_margin)
        // = entry_price * (10000 + collateral_ratio_bps - maintenance_margin_bps) / 10000
        let multiplier_bps = BPS_DIVISOR + collateral_ratio_bps - MAINTENANCE_MARGIN_BPS;
        (entry_price * multiplier_bps) / BPS_DIVISOR
    }
}

/// Calculate comprehensive PnL for a position
///
/// # PnL Components
/// 1. **Price PnL**: Profit/loss from price movement
///    - Long: (current_price - entry_price) * size / 1e7
///    - Short: (entry_price - current_price) * size / 1e7
///
/// 2. **Funding Payments**: Accumulated funding rate payments
///    - Long pays funding when rate is positive (long > short OI)
///    - Short pays funding when rate is negative (short > long OI)
///    - Payment = (cumulative_now - cumulative_entry) * size / 1e7
///
/// 3. **Borrowing Fees**: Time-based fees for leverage
///    - Fee = borrow_rate_per_second * time_elapsed * size / 1e7
///
/// # Arguments
/// * `env` - Soroban environment
/// * `position` - Position struct containing all position data
/// * `current_price` - Current market price (scaled by 1e7)
///
/// # Returns
/// Net PnL (can be negative) scaled by 1e7
fn calculate_pnl(env: &Env, position: &Position, current_price: i128) -> i128 {
    let size_i128 = position.size as i128;

    // 1. Calculate Price PnL
    // Size is in notional token units (collateral * leverage)
    // To get asset units: size / entry_price
    // PnL = price_diff * (size / entry_price)
    // Reordering to avoid precision loss: (price_diff * size) / entry_price
    let price_diff = if position.is_long {
        current_price - position.entry_price
    } else {
        position.entry_price - current_price
    };
    let price_pnl = (price_diff * size_i128) / position.entry_price;

    // 2. Calculate Funding Payments
    let market_manager = get_market_manager(env);
    let market_client = market_manager::Client::new(env, &market_manager);

    let cumulative_funding_long = market_client.get_cumulative_funding(&position.market_id, &true);
    let cumulative_funding_short =
        market_client.get_cumulative_funding(&position.market_id, &false);

    let funding_payment = if position.is_long {
        // Longs pay based on long-side cumulative funding
        // Note: cumulative funding is stored as (funding_rate_bps * seconds) to avoid precision loss
        // Formula: (bps·seconds * size) / (seconds_per_hour * price_scaling)
        let funding_accrued = cumulative_funding_long - position.entry_funding_long;
        // To avoid overflow, we break down the calculation:
        // Instead of (funding_accrued * size) / (3600 * 1e7)
        // We do: (funding_accrued / 3600) * size / 1e7
        // This divides first to keep intermediate values small
        let funding_per_second = funding_accrued / 3600;
        (funding_per_second * size_i128) / 10_000_000
    } else {
        // Shorts: pay when cumulative_funding_short increases, receive when cumulative_funding_long increases
        // Net funding cost = what they paid - what they received
        let funding_received = cumulative_funding_long - position.entry_funding_long;
        let funding_paid = cumulative_funding_short - position.entry_funding_short;
        let net_funding = funding_paid - funding_received;
        let funding_per_second = net_funding / 3600;
        (funding_per_second * size_i128) / 10_000_000
    };

    // 3. Calculate Borrowing Fees
    let config_manager = get_config_manager(env);
    let config_client = config_manager::Client::new(env, &config_manager);
    let borrow_rate_per_second = config_client.borrow_rate_per_second() as i128;
    let current_timestamp = env.ledger().timestamp();
    let time_elapsed = (current_timestamp - position.last_interaction) as i128;
    let borrowing_fee = (borrow_rate_per_second * time_elapsed * size_i128) / 10_000_000;

    // Net PnL = Price PnL - Funding Payments - Borrowing Fees
    // (funding_payment and borrowing_fee are costs, so subtract)
    price_pnl - funding_payment - borrowing_fee
}

#[contractimpl]
impl PositionManager {
    /// Initialize the PositionManager contract.
    ///
    /// # Arguments
    ///
    /// * `admin` - The administrator address (must authorize)
    /// * `config_manager` - Address of the ConfigManager contract
    pub fn initialize(env: Env, admin: Address, config_manager: Address) {
        // Prevent reinitialization
        if env.storage().instance().has(&DataKey::ConfigManager) {
            panic!("already initialized");
        }

        // Require admin to authorize initialization
        admin.require_auth();

        // Store the ConfigManager address
        env.storage()
            .instance()
            .set(&DataKey::ConfigManager, &config_manager);

        // Initialize the next position ID to 1 (0 means "no position" for orders)
        env.storage()
            .instance()
            .set(&DataKey::NextPositionId, &1u64);
    }

    /// Open a new perpetual position.
    ///
    /// # Arguments
    ///
    /// * `trader` - The address of the trader opening the position
    /// * `market_id` - The market identifier (e.g., 0 = XLM-PERP, 1 = BTC-PERP, 2 = ETH-PERP)
    /// * `collateral` - The amount of collateral to deposit (unsigned, in token base units)
    /// * `leverage` - The leverage multiplier (e.g., 5x, 10x, 20x)
    /// * `is_long` - True for long position, false for short
    ///
    /// # Returns
    ///
    /// The position ID
    ///
    /// # Implementation
    ///
    /// Position size is calculated as: `size = collateral * leverage`
    /// - Transfers collateral from trader to contract
    /// - Gets entry price from OracleIntegrator
    /// - Emits PositionOpened event
    /// - No fees applied for MVP
    pub fn open_position(
        env: Env,
        trader: Address,
        market_id: u32,
        collateral: u128,
        leverage: u32,
        is_long: bool,
    ) -> u64 {
        // Require trader authorization
        trader.require_auth();

        // Validate inputs
        if collateral == 0 {
            panic!("Collateral must be positive");
        }
        if leverage == 0 {
            panic!("Leverage must be positive");
        }

        // Validate leverage against ConfigManager limits
        validate_leverage(&env, leverage);

        // Calculate position size from collateral and leverage
        let size = collateral
            .checked_mul(leverage as u128)
            .expect("Size overflow");

        // Validate position size against ConfigManager minimum
        validate_position_size(&env, size);

        // Get entry price from OracleIntegrator
        let oracle_address = get_oracle(&env);
        let oracle_client = oracle_integrator::Client::new(&env, &oracle_address);
        let entry_price = oracle_client.get_price(&market_id);

        // Check market is not paused and can accept this position
        let market_manager = get_market_manager(&env);
        let market_client = market_manager::Client::new(&env, &market_manager);

        if !market_client.can_open_position(&market_id, &is_long, &size) {
            panic!("Cannot open position - market paused or OI limit reached");
        }

        // Get current cumulative funding rates for this position
        let entry_funding_long = market_client.get_cumulative_funding(&market_id, &true);
        let entry_funding_short = market_client.get_cumulative_funding(&market_id, &false);

        // Generate a new position ID
        let position_id = increment_position_id(&env);

        // Get liquidity pool and check utilization
        let pool_address = get_liquidity_pool(&env);
        let pool_client = liquidity_pool::Client::new(&env, &pool_address);

        // Check max utilization before opening position
        let config_manager = get_config_manager(&env);
        let config_client = config_manager::Client::new(&env, &config_manager);
        let max_utilization = config_client.max_utilization_ratio();

        // Calculate what utilization would be after this position
        let reserved_current = pool_client.get_reserved_liquidity();
        let available = pool_client.get_available_liquidity();

        if available <= 0 {
            panic!("no available liquidity");
        }

        let total_balance = available as u128 + reserved_current;
        let reserved_after = reserved_current + size;

        if total_balance > 0 {
            let utilization_after = ((reserved_after * 10000) / total_balance) as i128;
            if utilization_after > max_utilization {
                panic!("position would exceed max utilization");
            }
        }

        // Deposit collateral to liquidity pool
        pool_client.deposit_position_collateral(
            &env.current_contract_address(),
            &position_id,
            &trader,
            &collateral,
        );

        // Reserve liquidity for this position
        pool_client.reserve_liquidity(
            &env.current_contract_address(),
            &position_id,
            &size,
            &collateral,
        );

        // Calculate liquidation price
        let liquidation_price = calculate_liquidation_price(entry_price, collateral, size, is_long);

        // Create the position with all new fields
        let position = Position {
            trader: trader.clone(),
            market_id,
            collateral,
            size,
            is_long,
            entry_price,
            entry_funding_long,
            entry_funding_short,
            last_interaction: env.ledger().timestamp(),
            liquidation_price,
        };

        // Store the position
        set_position(&env, position_id, &position);

        // Add position ID to user's list of open positions
        add_user_position(&env, &trader, position_id);

        // Update open interest in MarketManager
        let size_i128 = size as i128;
        market_client.update_open_interest(
            &env.current_contract_address(),
            &market_id,
            &is_long,
            &size_i128,
        );

        // Emit position opened event
        PositionOpenedEvent {
            position_id,
            trader: trader.clone(),
            market_id,
            collateral,
            size,
            leverage,
            is_long,
            entry_price: entry_price as u128, // Convert i128 to u128 for event
        }
        .publish(&env);

        // Return the position ID
        position_id
    }

    /// Close an existing position.
    ///
    /// # Arguments
    ///
    /// * `trader` - The address of the trader closing the position
    /// * `position_id` - The unique position identifier
    ///
    /// # Returns
    ///
    /// The realized PnL (positive for profit, negative for loss)
    ///
    /// # Implementation
    ///
    /// - Gets current price from OracleIntegrator
    /// - Calculates comprehensive PnL (price PnL + funding payments + borrowing fees)
    /// - Settles PnL with LiquidityPool
    /// - Updates MarketManager open interest
    /// - Returns collateral ± PnL to trader
    /// - Emits PositionClosed event
    pub fn close_position(env: Env, trader: Address, position_id: u64) -> i128 {
        // Require trader authorization
        trader.require_auth();

        // Retrieve the position
        let position = get_position(&env, position_id);

        // Verify the trader owns the position
        if position.trader != trader {
            panic!("Unauthorized: caller does not own this position");
        }

        // Cancel all attached SL/TP orders and refund execution fees
        cancel_position_attached_orders(&env, position_id, OrderCancelReason::PositionClosed);

        // Get current price from OracleIntegrator
        let oracle_address = get_oracle(&env);
        let oracle_client = oracle_integrator::Client::new(&env, &oracle_address);
        let current_price = oracle_client.get_price(&position.market_id);

        // Calculate comprehensive PnL
        let pnl = calculate_pnl(&env, &position, current_price);

        log!(&env, "pnl", pnl);

        // Get liquidity pool
        let pool_address = get_liquidity_pool(&env);
        let pool_client = liquidity_pool::Client::new(&env, &pool_address);

        // Release reserved liquidity
        pool_client.release_liquidity(
            &env.current_contract_address(),
            &position_id,
            &position.size,
        );

        // Settle PnL with pool and withdraw collateral to trader
        let collateral_i128 = position.collateral as i128;
        let final_amount = collateral_i128 + pnl;

        log!(&env, "final", final_amount);

        // Withdraw collateral and settle PnL
        if pnl >= 0 {
            // Profit or break-even: return full collateral, then pay profit separately
            pool_client.withdraw_position_collateral(
                &env.current_contract_address(),
                &position_id,
                &trader,
                &position.collateral,
            );
            if pnl > 0 {
                pool_client.settle_trader_pnl(&env.current_contract_address(), &trader, &pnl);
            }
        } else {
            // Loss: return reduced collateral (collateral + negative pnl)
            let withdrawal_amount = if final_amount > 0 {
                final_amount as u128
            } else {
                0u128
            };
            pool_client.withdraw_position_collateral(
                &env.current_contract_address(),
                &position_id,
                &trader,
                &withdrawal_amount,
            );
        }

        // Update open interest in MarketManager (decrease)
        let market_manager = get_market_manager(&env);
        let market_client = market_manager::Client::new(&env, &market_manager);
        let size_decrease = -(position.size as i128); // Negative to decrease
        market_client.update_open_interest(
            &env.current_contract_address(),
            &position.market_id,
            &position.is_long,
            &size_decrease,
        );

        // Delete the position from storage
        remove_position(&env, position_id);

        // Remove position ID from user's list of open positions
        remove_user_position(&env, &trader, position_id);

        // Emit position closed event
        PositionClosedEvent {
            position_id,
            trader: trader.clone(),
            pnl,
        }
        .publish(&env);

        // Return PnL
        pnl
    }

    /// Increase position size or add collateral.
    ///
    /// # Arguments
    ///
    /// * `trader` - The address of the trader
    /// * `position_id` - The unique position identifier
    /// * `additional_collateral` - Additional collateral to add (0 if none)
    /// * `additional_size` - Additional position size (0 if none)
    ///
    /// # Implementation
    ///
    /// - Verifies trader owns the position
    /// - Checks leverage limits with new total size
    /// - Checks market open interest limits for additional size
    /// - Transfers additional collateral if provided
    /// - Updates position size and recalculates average entry price
    /// - Recalculates liquidation price
    /// - Updates funding rate snapshots and last_interaction timestamp
    /// - Updates MarketManager open interest
    /// - Emits PositionModified event
    pub fn increase_position(
        env: Env,
        trader: Address,
        position_id: u64,
        additional_collateral: u128,
        additional_size: u128,
    ) {
        // Require trader authorization
        trader.require_auth();

        // At least one of collateral or size must be added
        if additional_collateral == 0 && additional_size == 0 {
            panic!("Must add collateral or size");
        }

        // Retrieve the position
        let mut position = get_position(&env, position_id);

        // Verify the trader owns the position
        if position.trader != trader {
            panic!("Unauthorized: caller does not own this position");
        }

        // Get current price for entry price calculation if adding size
        let current_price = if additional_size > 0 {
            let oracle_address = get_oracle(&env);
            let oracle_client = oracle_integrator::Client::new(&env, &oracle_address);
            oracle_client.get_price(&position.market_id)
        } else {
            position.entry_price
        };

        // Update collateral if provided
        if additional_collateral > 0 {
            let pool_address = get_liquidity_pool(&env);
            let pool_client = liquidity_pool::Client::new(&env, &pool_address);

            // Transfer additional collateral from trader to pool
            pool_client.deposit_position_collateral(
                &env.current_contract_address(),
                &position_id,
                &trader,
                &additional_collateral,
            );

            position.collateral = position
                .collateral
                .checked_add(additional_collateral)
                .expect("Collateral overflow");
        }

        // Update size if provided
        if additional_size > 0 {
            // Check market can accept additional size
            let market_manager = get_market_manager(&env);
            let market_client = market_manager::Client::new(&env, &market_manager);

            if !market_client.can_open_position(
                &position.market_id,
                &position.is_long,
                &additional_size,
            ) {
                panic!("Cannot increase position - market paused or OI limit reached");
            }

            // Calculate new average entry price
            let old_size_value = position.size as i128 * position.entry_price;
            let new_size_value = additional_size as i128 * current_price;
            let total_size = position.size + additional_size;
            let avg_entry_price = (old_size_value + new_size_value) / (total_size as i128);

            // Reserve additional liquidity
            let pool_address = get_liquidity_pool(&env);
            let pool_client = liquidity_pool::Client::new(&env, &pool_address);
            pool_client.reserve_liquidity(
                &env.current_contract_address(),
                &position_id,
                &additional_size,
                &0, // No new collateral reserved (already handled above)
            );

            // Update position fields
            position.size = total_size;
            position.entry_price = avg_entry_price;

            // Update open interest in MarketManager
            let size_i128 = additional_size as i128;
            market_client.update_open_interest(
                &env.current_contract_address(),
                &position.market_id,
                &position.is_long,
                &size_i128,
            );

            // Update funding snapshots to current values
            position.entry_funding_long =
                market_client.get_cumulative_funding(&position.market_id, &true);
            position.entry_funding_short =
                market_client.get_cumulative_funding(&position.market_id, &false);
        }

        // Check leverage is still within limits
        let effective_leverage = position.size / position.collateral;
        validate_leverage(&env, effective_leverage as u32);

        // Recalculate liquidation price
        position.liquidation_price = calculate_liquidation_price(
            position.entry_price,
            position.collateral,
            position.size,
            position.is_long,
        );

        // Update last interaction timestamp
        position.last_interaction = env.ledger().timestamp();

        // Store updated position
        set_position(&env, position_id, &position);

        // Emit position modified event
        PositionModifiedEvent {
            position_id,
            trader: trader.clone(),
            new_collateral: position.collateral,
            new_size: position.size,
            new_liquidation_price: position.liquidation_price,
        }
        .publish(&env);
    }

    /// Decrease position size or remove collateral.
    ///
    /// # Arguments
    ///
    /// * `trader` - The address of the trader
    /// * `position_id` - The unique position identifier
    /// * `collateral_to_remove` - Collateral to remove (0 if none)
    /// * `size_to_reduce` - Position size to reduce (0 if none)
    ///
    /// # Implementation
    ///
    /// - Verifies trader owns the position
    /// - If reducing size, realizes proportional PnL
    /// - Releases corresponding reserved liquidity
    /// - Updates MarketManager open interest
    /// - If removing collateral, verifies position remains sufficiently collateralized
    /// - Transfers collateral to trader if removed
    /// - Recalculates liquidation price
    /// - Updates last_interaction timestamp
    /// - Emits PositionModified event
    pub fn decrease_position(
        env: Env,
        trader: Address,
        position_id: u64,
        collateral_to_remove: u128,
        size_to_reduce: u128,
    ) {
        // Require trader authorization
        trader.require_auth();

        // At least one of collateral or size must be removed
        if collateral_to_remove == 0 && size_to_reduce == 0 {
            panic!("Must remove collateral or size");
        }

        // Retrieve the position
        let mut position = get_position(&env, position_id);

        // Verify the trader owns the position
        if position.trader != trader {
            panic!("Unauthorized: caller does not own this position");
        }

        // Verify we're not removing more than exists
        if collateral_to_remove > position.collateral {
            panic!("Cannot remove more collateral than exists");
        }
        if size_to_reduce > position.size {
            panic!("Cannot reduce more size than exists");
        }

        let pool_address = get_liquidity_pool(&env);
        let pool_client = liquidity_pool::Client::new(&env, &pool_address);

        // Handle size reduction with PnL realization
        if size_to_reduce > 0 {
            // Get current price
            let oracle_address = get_oracle(&env);
            let oracle_client = oracle_integrator::Client::new(&env, &oracle_address);
            let current_price = oracle_client.get_price(&position.market_id);

            // Calculate proportional PnL for the size being closed
            let total_pnl = calculate_pnl(&env, &position, current_price);
            let proportion = (size_to_reduce as i128 * 10000) / (position.size as i128);
            let realized_pnl = (total_pnl * proportion) / 10000;

            // Realize PnL: adjust collateral by realized PnL
            let collateral_i128 = position.collateral as i128;
            let new_collateral_i128 = collateral_i128 + realized_pnl;

            if new_collateral_i128 <= 0 {
                panic!("Position underwater - use close or liquidate instead");
            }

            // Settle realized PnL with trader
            if realized_pnl > 0 {
                // Profit: pay trader from pool reserves
                pool_client.settle_trader_pnl(
                    &env.current_contract_address(),
                    &trader,
                    &realized_pnl,
                );
            } else if realized_pnl < 0 {
                // Loss: withdraw the loss amount from position collateral back to pool
                let loss_amount = (-realized_pnl) as u128;
                pool_client.withdraw_position_collateral(
                    &env.current_contract_address(),
                    &position_id,
                    &pool_address,
                    &loss_amount,
                );
            }

            position.collateral = new_collateral_i128 as u128;

            // Release reserved liquidity
            pool_client.release_liquidity(
                &env.current_contract_address(),
                &position_id,
                &size_to_reduce,
            );

            // Update MarketManager open interest (decrease)
            let market_manager = get_market_manager(&env);
            let market_client = market_manager::Client::new(&env, &market_manager);
            let size_decrease = -(size_to_reduce as i128);
            market_client.update_open_interest(
                &env.current_contract_address(),
                &position.market_id,
                &position.is_long,
                &size_decrease,
            );

            // Update position size
            position.size = position.size - size_to_reduce;

            // Update funding snapshots to current values
            position.entry_funding_long =
                market_client.get_cumulative_funding(&position.market_id, &true);
            position.entry_funding_short =
                market_client.get_cumulative_funding(&position.market_id, &false);

            // Verify remaining position meets minimum size
            validate_position_size(&env, position.size);
        }

        // Handle collateral removal
        if collateral_to_remove > 0 {
            // Verify this won't make position undercollateralized
            let remaining_collateral = position.collateral - collateral_to_remove;
            let effective_leverage = position.size / remaining_collateral;

            // Check leverage is still within limits
            validate_leverage(&env, effective_leverage as u32);

            // Check maintenance margin (1% = 100x max effective leverage)
            let margin_ratio = (remaining_collateral * 10000) / position.size;
            if margin_ratio < 100 {
                // Less than 1% margin
                panic!("Cannot remove collateral - would violate maintenance margin");
            }

            // Update collateral
            position.collateral = remaining_collateral;

            // Withdraw collateral from pool to trader
            pool_client.withdraw_position_collateral(
                &env.current_contract_address(),
                &position_id,
                &trader,
                &collateral_to_remove,
            );
        }

        // Recalculate liquidation price
        position.liquidation_price = calculate_liquidation_price(
            position.entry_price,
            position.collateral,
            position.size,
            position.is_long,
        );

        // Update last interaction timestamp
        position.last_interaction = env.ledger().timestamp();

        // Store updated position
        set_position(&env, position_id, &position);

        // Emit position modified event
        PositionModifiedEvent {
            position_id,
            trader: trader.clone(),
            new_collateral: position.collateral,
            new_size: position.size,
            new_liquidation_price: position.liquidation_price,
        }
        .publish(&env);
    }

    /// Liquidate an undercollateralized position.
    ///
    /// # Arguments
    ///
    /// * `keeper` - The address of the keeper liquidating the position
    /// * `position_id` - The unique position identifier
    ///
    /// # Returns
    ///
    /// The liquidation reward paid to the keeper
    ///
    /// # Implementation
    ///
    /// - Gets current price from OracleIntegrator
    /// - Calculates comprehensive PnL including all fees
    /// - Verifies position is liquidatable (underwater or below maintenance margin)
    /// - Calculates liquidation fees:
    ///   - 0.3% of position size goes to keeper as reward
    ///   - 0.2% of position size goes to liquidity pool
    /// - Settles with LiquidityPool (collateral minus losses and fees)
    /// - Updates MarketManager open interest
    /// - Deletes position from storage
    /// - Emits PositionLiquidated event
    pub fn liquidate_position(env: Env, keeper: Address, position_id: u64) -> u128 {
        // Keeper must authorize (they're paying gas)
        keeper.require_auth();

        // Retrieve the position
        let position = get_position(&env, position_id);

        // Cancel all attached SL/TP orders and refund execution fees
        cancel_position_attached_orders(&env, position_id, OrderCancelReason::PositionLiquidated);

        // Get current price from OracleIntegrator
        let oracle_address = get_oracle(&env);
        let oracle_client = oracle_integrator::Client::new(&env, &oracle_address);
        let current_price = oracle_client.get_price(&position.market_id);

        // Calculate comprehensive PnL
        let pnl = calculate_pnl(&env, &position, current_price);

        // Calculate remaining collateral value after PnL
        let collateral_i128 = position.collateral as i128;
        let remaining_value = collateral_i128 + pnl;

        // Calculate maintenance margin requirement (1% of position size)
        let maintenance_margin = (position.size as i128 * 100) / 10000; // 1% in basis points

        // Verify position is liquidatable
        // Position is liquidatable if:
        // 1. Remaining value <= 0 (completely underwater), OR
        // 2. Remaining value < maintenance_margin (below 1% maintenance)
        if remaining_value > maintenance_margin {
            panic!("Position not liquidatable - sufficient collateral");
        }

        // Get ConfigManager for liquidation fee parameters
        let config_manager = get_config_manager(&env);
        let config_client = config_manager::Client::new(&env, &config_manager);
        let liquidation_fee = config_client.liquidation_fee_bps(); // In basis points (e.g., 50 = 0.5%)

        // Calculate liquidation fees
        // Total liquidation fee is split: 60% to keeper, 40% to pool
        let total_liquidation_fee = (position.size as i128 * liquidation_fee as i128) / 10000;
        let keeper_reward = (total_liquidation_fee * 60) / 100; // 60% of fee
        let pool_fee = (total_liquidation_fee * 40) / 100; // 40% of fee

        // Get liquidity pool
        let pool_address = get_liquidity_pool(&env);
        let pool_client = liquidity_pool::Client::new(&env, &pool_address);

        // Release reserved liquidity
        pool_client.release_liquidity(
            &env.current_contract_address(),
            &position_id,
            &position.size,
        );

        // Settle liquidation:
        // - If position has remaining collateral value, use it to pay fees
        // - Keeper gets their reward from position collateral
        // - Pool gets their fee from position collateral
        // - Any remaining collateral (or deficit) goes to/from pool

        let mut keeper_payment = 0u128;

        // Pay keeper from actual collateral (not remaining_value)
        // The collateral physically exists in the pool; PnL is an accounting calculation
        if keeper_reward > 0 && position.collateral > 0 {
            let keeper_reward_u128 = keeper_reward as u128;
            keeper_payment = if keeper_reward_u128 > position.collateral {
                position.collateral // Pay whatever collateral is available
            } else {
                keeper_reward_u128 // Pay full reward
            };

            if keeper_payment > 0 {
                pool_client.withdraw_position_collateral(
                    &env.current_contract_address(),
                    &position_id,
                    &keeper,
                    &keeper_payment,
                );
            }
        }

        // Remaining collateral goes to pool (covers losses and pool fee)
        let remaining_collateral = position.collateral - keeper_payment;
        if remaining_collateral > 0 {
            pool_client.withdraw_position_collateral(
                &env.current_contract_address(),
                &position_id,
                &pool_address,
                &remaining_collateral,
            );
        }

        // Update open interest in MarketManager (decrease)
        let market_manager = get_market_manager(&env);
        let market_client = market_manager::Client::new(&env, &market_manager);
        let size_decrease = -(position.size as i128);
        market_client.update_open_interest(
            &env.current_contract_address(),
            &position.market_id,
            &position.is_long,
            &size_decrease,
        );

        // Delete the position from storage
        remove_position(&env, position_id);

        // Remove position ID from user's list of open positions
        remove_user_position(&env, &position.trader, position_id);

        // Emit position liquidated event
        PositionLiquidatedEvent {
            position_id,
            trader: position.trader.clone(),
            liquidator: keeper.clone(),
            liquidation_price: current_price,
            liquidation_reward: keeper_payment,
        }
        .publish(&env);

        // Return keeper reward
        keeper_payment
    }

    /// Get position details.
    ///
    /// # Arguments
    ///
    /// * `position_id` - The unique position identifier
    ///
    /// # Returns
    ///
    /// The Position struct with all position details
    pub fn get_position(env: Env, position_id: u64) -> Position {
        // Fetch position from persistent storage
        get_position(&env, position_id)
    }

    /// Calculate unrealized PnL for a position.
    ///
    /// # Arguments
    ///
    /// * `position_id` - The unique position identifier
    ///
    /// # Returns
    ///
    /// The unrealized PnL (positive for profit, negative for loss)
    pub fn calculate_pnl(env: Env, position_id: u64) -> i128 {
        let position = get_position(&env, position_id);

        let oracle = get_oracle(&env);
        let oracle_client = oracle_integrator::Client::new(&env, &oracle);
        let current_price = oracle_client.get_price(&position.market_id);

        calculate_pnl(&env, &position, current_price)
    }

    /// Get all open position IDs for a specific trader.
    ///
    /// # Arguments
    ///
    /// * `trader` - The address of the trader
    ///
    /// # Returns
    ///
    /// A vector of position IDs (u64) for all open positions owned by the trader.
    /// Returns an empty vector if the trader has no open positions.
    ///
    /// # Usage
    ///
    /// Frontend can call this function to get all position IDs for a user,
    /// then call `get_position(id)` for each ID to retrieve full position details.
    pub fn get_user_open_positions(env: Env, trader: Address) -> soroban_sdk::Vec<u64> {
        get_user_positions(&env, &trader)
    }

    // ========================================================================
    // ORDER FUNCTIONS - Limit, Stop-Loss, Take-Profit
    // ========================================================================

    /// Create a limit order to open a position when price reaches target.
    ///
    /// # Arguments
    /// * `trader` - The address creating the order
    /// * `market_id` - The market identifier (0=XLM, 1=BTC, 2=ETH)
    /// * `trigger_price` - Price at which to execute (scaled 1e7)
    /// * `acceptable_price` - Maximum slippage from trigger (0 = any price)
    /// * `collateral` - Collateral for the new position
    /// * `leverage` - Leverage for the new position
    /// * `is_long` - True for long, false for short
    /// * `execution_fee` - Fee to pay keeper on execution
    /// * `expiration` - Timestamp when order expires (0 = no expiry)
    ///
    /// # Returns
    /// The order ID
    pub fn create_limit_order(
        env: Env,
        trader: Address,
        market_id: u32,
        trigger_price: i128,
        acceptable_price: i128,
        collateral: u128,
        leverage: u32,
        is_long: bool,
        execution_fee: u128,
        expiration: u64,
    ) -> u64 {
        trader.require_auth();

        // Validate inputs
        if trigger_price <= 0 {
            panic!("Trigger price must be positive");
        }
        if collateral == 0 {
            panic!("Collateral must be positive");
        }
        validate_leverage(&env, leverage);
        validate_execution_fee(&env, execution_fee);

        // Check market is not paused
        let market_manager = get_market_manager(&env);
        let market_client = market_manager::Client::new(&env, &market_manager);
        if market_client.is_market_paused(&market_id) {
            panic!("Market is paused");
        }

        // Calculate position size
        let size = collateral
            .checked_mul(leverage as u128)
            .expect("Size overflow");
        validate_position_size(&env, size);

        // Transfer execution fee AND collateral from trader to contract (escrow)
        let token = get_token(&env);
        let token_client = token::Client::new(&env, &token);
        let total_escrow = execution_fee + collateral;
        token_client.transfer(
            &trader,
            &env.current_contract_address(),
            &(total_escrow as i128),
        );

        // Create order
        let order_id = increment_order_id(&env);
        let order = Order {
            order_id,
            order_type: OrderType::Limit,
            trader: trader.clone(),
            market_id,
            position_id: 0, // No position yet
            trigger_price,
            acceptable_price,
            collateral,
            size,
            leverage,
            is_long,
            close_percentage: 0,
            execution_fee,
            expiration,
            created_at: env.ledger().timestamp(),
        };

        // Store order
        set_order(&env, order_id, &order);
        add_user_order(&env, &trader, order_id);
        add_market_order(&env, market_id, order_id);

        // Emit event
        OrderCreatedEvent {
            order_id,
            order_type: OrderType::Limit,
            trader: trader.clone(),
            market_id,
            position_id: 0,
            trigger_price,
            size,
            is_long,
            expiration,
        }
        .publish(&env);

        order_id
    }

    /// Create a stop-loss order attached to an existing position.
    ///
    /// # Arguments
    /// * `trader` - The position owner
    /// * `position_id` - The position to protect
    /// * `trigger_price` - Price at which to close position
    /// * `acceptable_price` - Minimum acceptable price for closure (0 = any)
    /// * `close_percentage` - Percentage to close (10000 = 100%)
    /// * `execution_fee` - Fee to pay keeper
    /// * `expiration` - Order expiration (0 = no expiry)
    ///
    /// # Returns
    /// The order ID
    pub fn create_stop_loss(
        env: Env,
        trader: Address,
        position_id: u64,
        trigger_price: i128,
        acceptable_price: i128,
        close_percentage: u32,
        execution_fee: u128,
        expiration: u64,
    ) -> u64 {
        trader.require_auth();

        // Get and validate position ownership
        let position = get_position(&env, position_id);
        if position.trader != trader {
            panic!("Unauthorized: caller does not own this position");
        }

        // Validate close percentage
        if close_percentage == 0 || close_percentage > 10000 {
            panic!("Invalid close percentage");
        }

        // Validate execution fee
        validate_execution_fee(&env, execution_fee);

        // Check maximum orders per position limit
        let existing_orders = get_position_orders_list(&env, position_id);
        if existing_orders.len() >= MAX_ORDERS_PER_POSITION {
            panic!("Maximum orders per position reached");
        }

        // Validate stop-loss price
        // For longs: SL triggers when price falls below trigger (must be below current)
        // For shorts: SL triggers when price rises above trigger (must be above current)
        let oracle_address = get_oracle(&env);
        let oracle_client = oracle_integrator::Client::new(&env, &oracle_address);
        let current_price = oracle_client.get_price(&position.market_id);

        if position.is_long {
            if trigger_price >= current_price {
                panic!("Stop-loss for long must be below current price");
            }
            if trigger_price <= position.liquidation_price {
                panic!("Stop-loss must be above liquidation price");
            }
        } else {
            if trigger_price <= current_price {
                panic!("Stop-loss for short must be above current price");
            }
            if trigger_price >= position.liquidation_price {
                panic!("Stop-loss must be below liquidation price");
            }
        }

        // Transfer execution fee
        let token = get_token(&env);
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(
            &trader,
            &env.current_contract_address(),
            &(execution_fee as i128),
        );

        // Calculate size to close
        let size_to_close = (position.size * close_percentage as u128) / 10000;

        // Create order
        let order_id = increment_order_id(&env);
        let order = Order {
            order_id,
            order_type: OrderType::StopLoss,
            trader: trader.clone(),
            market_id: position.market_id,
            position_id,
            trigger_price,
            acceptable_price,
            collateral: 0,
            size: size_to_close,
            leverage: 0,
            is_long: position.is_long,
            close_percentage,
            execution_fee,
            expiration,
            created_at: env.ledger().timestamp(),
        };

        // Store order
        set_order(&env, order_id, &order);
        add_user_order(&env, &trader, order_id);
        add_position_order(&env, position_id, order_id);
        add_market_order(&env, position.market_id, order_id);

        // Emit event
        OrderCreatedEvent {
            order_id,
            order_type: OrderType::StopLoss,
            trader: trader.clone(),
            market_id: position.market_id,
            position_id,
            trigger_price,
            size: size_to_close,
            is_long: position.is_long,
            expiration,
        }
        .publish(&env);

        order_id
    }

    /// Create a take-profit order attached to an existing position.
    ///
    /// # Arguments
    /// * `trader` - The position owner
    /// * `position_id` - The position to take profit from
    /// * `trigger_price` - Price at which to close position
    /// * `acceptable_price` - Minimum acceptable price for closure (0 = any)
    /// * `close_percentage` - Percentage to close (10000 = 100%)
    /// * `execution_fee` - Fee to pay keeper
    /// * `expiration` - Order expiration (0 = no expiry)
    ///
    /// # Returns
    /// The order ID
    pub fn create_take_profit(
        env: Env,
        trader: Address,
        position_id: u64,
        trigger_price: i128,
        acceptable_price: i128,
        close_percentage: u32,
        execution_fee: u128,
        expiration: u64,
    ) -> u64 {
        trader.require_auth();

        // Get and validate position ownership
        let position = get_position(&env, position_id);
        if position.trader != trader {
            panic!("Unauthorized: caller does not own this position");
        }

        // Validate close percentage
        if close_percentage == 0 || close_percentage > 10000 {
            panic!("Invalid close percentage");
        }

        // Validate execution fee
        validate_execution_fee(&env, execution_fee);

        // Check maximum orders per position limit
        let existing_orders = get_position_orders_list(&env, position_id);
        if existing_orders.len() >= MAX_ORDERS_PER_POSITION {
            panic!("Maximum orders per position reached");
        }

        // Validate take-profit price
        // For longs: TP triggers when price rises above trigger (must be above current)
        // For shorts: TP triggers when price falls below trigger (must be below current)
        let oracle_address = get_oracle(&env);
        let oracle_client = oracle_integrator::Client::new(&env, &oracle_address);
        let current_price = oracle_client.get_price(&position.market_id);

        if position.is_long {
            if trigger_price <= current_price {
                panic!("Take-profit for long must be above current price");
            }
        } else {
            if trigger_price >= current_price {
                panic!("Take-profit for short must be below current price");
            }
        }

        // Transfer execution fee
        let token = get_token(&env);
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(
            &trader,
            &env.current_contract_address(),
            &(execution_fee as i128),
        );

        // Calculate size to close
        let size_to_close = (position.size * close_percentage as u128) / 10000;

        // Create order
        let order_id = increment_order_id(&env);
        let order = Order {
            order_id,
            order_type: OrderType::TakeProfit,
            trader: trader.clone(),
            market_id: position.market_id,
            position_id,
            trigger_price,
            acceptable_price,
            collateral: 0,
            size: size_to_close,
            leverage: 0,
            is_long: position.is_long,
            close_percentage,
            execution_fee,
            expiration,
            created_at: env.ledger().timestamp(),
        };

        // Store order
        set_order(&env, order_id, &order);
        add_user_order(&env, &trader, order_id);
        add_position_order(&env, position_id, order_id);
        add_market_order(&env, position.market_id, order_id);

        // Emit event
        OrderCreatedEvent {
            order_id,
            order_type: OrderType::TakeProfit,
            trader: trader.clone(),
            market_id: position.market_id,
            position_id,
            trigger_price,
            size: size_to_close,
            is_long: position.is_long,
            expiration,
        }
        .publish(&env);

        order_id
    }

    /// Cancel an active order.
    ///
    /// # Arguments
    /// * `trader` - The order owner
    /// * `order_id` - The order to cancel
    pub fn cancel_order(env: Env, trader: Address, order_id: u64) {
        trader.require_auth();

        let order = get_order_from_storage(&env, order_id);

        // Verify ownership
        if order.trader != trader {
            panic!("Unauthorized: caller does not own this order");
        }

        // Refund execution fee (and collateral for limit orders)
        let token = get_token(&env);
        let token_client = token::Client::new(&env, &token);

        let refund_amount = match order.order_type {
            OrderType::Limit => order.execution_fee + order.collateral, // Limit orders escrow collateral
            _ => order.execution_fee, // SL/TP only escrow execution fee
        };

        token_client.transfer(
            &env.current_contract_address(),
            &trader,
            &(refund_amount as i128),
        );

        // Clean up storage
        cleanup_order(&env, &order, OrderCancelReason::UserCancelled);
    }

    /// Execute an order when conditions are met. Called by keeper bots.
    ///
    /// # Arguments
    /// * `keeper` - The keeper executing the order
    /// * `order_id` - The order to execute
    ///
    /// # Returns
    /// For Limit: the new position_id as i128
    /// For SL/TP: the realized PnL
    pub fn execute_order(env: Env, keeper: Address, order_id: u64) -> i128 {
        keeper.require_auth();

        let order = get_order_from_storage(&env, order_id);

        // Check expiration
        if order.expiration > 0 && env.ledger().timestamp() > order.expiration {
            // Refund execution fee to trader and cancel
            let token = get_token(&env);
            let token_client = token::Client::new(&env, &token);
            token_client.transfer(
                &env.current_contract_address(),
                &order.trader,
                &(order.execution_fee as i128),
            );
            cleanup_order(&env, &order, OrderCancelReason::Expired);
            panic!("Order expired");
        }

        // Get current price
        let oracle_address = get_oracle(&env);
        let oracle_client = oracle_integrator::Client::new(&env, &oracle_address);
        let current_price = oracle_client.get_price(&order.market_id);

        // Check market is not paused
        let market_manager = get_market_manager(&env);
        let market_client = market_manager::Client::new(&env, &market_manager);
        if market_client.is_market_paused(&order.market_id) {
            panic!("Market is paused");
        }

        // Verify trigger condition is met
        if !check_order_trigger(&order, current_price) {
            panic!("Order trigger condition not met");
        }

        // Verify acceptable price
        if !check_acceptable_price(&order, current_price) {
            panic!("Current price outside acceptable range");
        }

        // Execute based on order type
        let result = match order.order_type {
            OrderType::Limit => execute_limit_order(&env, &order, current_price),
            OrderType::StopLoss | OrderType::TakeProfit => {
                execute_sl_tp_order(&env, &order, current_price)
            }
        };

        // Pay execution fee to keeper
        let token = get_token(&env);
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(
            &env.current_contract_address(),
            &keeper,
            &(order.execution_fee as i128),
        );

        // Emit execution event
        let position_id_for_event = match order.order_type {
            OrderType::Limit => result as u64,
            _ => order.position_id,
        };
        let pnl_for_event = match order.order_type {
            OrderType::Limit => 0,
            _ => result,
        };

        OrderExecutedEvent {
            order_id: order.order_id,
            order_type: order.order_type.clone(),
            trader: order.trader.clone(),
            keeper: keeper.clone(),
            execution_price: current_price,
            position_id: position_id_for_event,
            pnl: pnl_for_event,
            execution_fee: order.execution_fee,
        }
        .publish(&env);

        // Clean up order storage (don't emit cancel event since we emitted execute event)
        remove_order(&env, order.order_id);
        remove_user_order(&env, &order.trader, order.order_id);
        remove_market_order(&env, order.market_id, order.order_id);
        if order.position_id > 0 {
            remove_position_order(&env, order.position_id, order.order_id);
        }

        result
    }

    // ========================================================================
    // ORDER QUERY FUNCTIONS
    // ========================================================================

    /// Get order details by ID.
    ///
    /// # Arguments
    /// * `order_id` - The order identifier
    ///
    /// # Returns
    /// The full Order struct with all order parameters
    ///
    /// # Panics
    /// Panics if order does not exist
    pub fn get_order(env: Env, order_id: u64) -> Order {
        get_order_from_storage(&env, order_id)
    }

    /// Get all active order IDs for a user.
    ///
    /// # Arguments
    /// * `trader` - The trader address
    ///
    /// # Returns
    /// Vector of order IDs (use `get_order()` to fetch full details for each)
    pub fn get_user_orders(env: Env, trader: Address) -> soroban_sdk::Vec<u64> {
        get_user_orders_list(&env, &trader)
    }

    /// Get all orders (SL/TP) attached to a position.
    ///
    /// # Arguments
    /// * `position_id` - The position identifier
    ///
    /// # Returns
    /// Vector of order IDs for stop-loss and take-profit orders on this position
    pub fn get_position_orders(env: Env, position_id: u64) -> soroban_sdk::Vec<u64> {
        get_position_orders_list(&env, position_id)
    }

    /// Get all active orders for a market. Used by keeper bots to discover
    /// executable orders.
    ///
    /// # Arguments
    /// * `market_id` - The market identifier (0=XLM, 1=BTC, 2=ETH)
    ///
    /// # Returns
    /// Vector of all active order IDs in this market
    pub fn get_market_orders(env: Env, market_id: u32) -> soroban_sdk::Vec<u64> {
        get_market_orders_list(&env, market_id)
    }

    /// Check if an order can be executed at current price.
    /// Used by keepers to filter executable orders before calling `execute_order()`.
    ///
    /// # Arguments
    /// * `order_id` - The order identifier
    ///
    /// # Returns
    /// True if the order exists, is not expired, market is not paused,
    /// position still exists (for SL/TP), and trigger condition is met
    pub fn can_execute_order(env: Env, order_id: u64) -> bool {
        if !order_exists(&env, order_id) {
            return false;
        }

        let order = get_order_from_storage(&env, order_id);

        // Check expiration
        if order.expiration > 0 && env.ledger().timestamp() > order.expiration {
            return false;
        }

        // Check market not paused
        let market_manager = get_market_manager(&env);
        let market_client = market_manager::Client::new(&env, &market_manager);
        if market_client.is_market_paused(&order.market_id) {
            return false;
        }

        // Check position exists for SL/TP
        if order.position_id > 0 {
            if !env
                .storage()
                .persistent()
                .has(&DataKey::Position(order.position_id))
            {
                return false;
            }
        }

        // Check trigger condition
        let oracle_address = get_oracle(&env);
        let oracle_client = oracle_integrator::Client::new(&env, &oracle_address);
        let current_price = oracle_client.get_price(&order.market_id);

        check_order_trigger(&order, current_price)
    }

    /// Set minimum execution fee required for orders (admin only).
    /// The execution fee incentivizes keeper bots to execute orders.
    ///
    /// # Arguments
    /// * `admin` - The admin address (must match ConfigManager admin)
    /// * `fee` - The minimum fee in token base units (e.g., 1_000_000 = 0.1 tokens with 7 decimals)
    ///
    /// # Panics
    /// Panics if caller is not the admin
    pub fn set_min_execution_fee(env: Env, admin: Address, fee: u128) {
        admin.require_auth();

        let config_manager = get_config_manager(&env);
        let config_client = config_manager::Client::new(&env, &config_manager);
        let config_admin = config_client.admin();

        if admin != config_admin {
            panic!("Unauthorized");
        }

        env.storage()
            .instance()
            .set(&DataKey::MinExecutionFee, &fee);
    }

    /// Get the minimum execution fee required for orders.
    ///
    /// # Returns
    /// The minimum fee in token base units (default: 1_000_000 = 0.1 tokens)
    pub fn min_execution_fee(env: Env) -> u128 {
        get_min_execution_fee(&env)
    }
}

#[cfg(test)]
mod test;

#![no_std]

//! # Oracle Integrator Contract
//!
//! Price feed aggregation and validation for the Stellars Finance protocol. Provides
//! reliable price data for position entry/exit, liquidation, and funding calculations.
//!
//! ## Key Features
//! - **Multi-Oracle Aggregation**: Fetches prices from DIA and Reflector oracles (production)
//! - **Test Mode**: Simulated prices with configurable oscillation for testing
//! - **Price Validation**: Staleness checks, bounds validation, and cross-oracle deviation checks
//! - **Median Calculation**: Returns median of oracle prices to resist manipulation
//!
//! ## Supported Markets
//! - Market 0: XLM/USD
//! - Market 1: BTC/USD
//! - Market 2: ETH/USD
//!
//! ## Test Mode
//! When enabled, returns simulated prices that oscillate ±10% per hour around a base price.
//! Use `set_fixed_price_mode(true)` to disable oscillation for deterministic testing.
//!
//! ## Production Mode (Not Yet Implemented)
//! Will fetch prices from DIA and Reflector oracles, validate each price, check
//! cross-oracle deviation (max 5%), and return the median.
//!
//! ## Usage
//! - PositionManager calls `get_price()` for entry/exit prices
//! - Admin configures test mode via `set_test_mode()`

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map, String};

#[cfg(not(test))]
mod config_manager {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/config_manager.wasm");
}

#[contracttype]
pub enum DataKey {
    ConfigManager,
    TestMode,           // bool: test mode enabled/disabled
    TestBasePrice(u32), // i128: base price per market_id for simulation
    FixedPriceMode,     // bool: if true, return base price without oscillation
}

/// Get the ConfigManager address from storage
fn get_config_manager(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::ConfigManager)
        .expect("ConfigManager not initialized")
}

/// Get asset symbols for oracle queries
/// Returns (dia_symbol, reflector_symbol)
fn get_asset_symbol(env: &Env, market_id: u32) -> (String, String) {
    match market_id {
        0 => (
            String::from_str(env, "XLM/USD"),
            String::from_str(env, "XLM"),
        ),
        1 => (
            String::from_str(env, "BTC/USD"),
            String::from_str(env, "Bitcoin"),
        ),
        2 => (
            String::from_str(env, "ETH/USD"),
            String::from_str(env, "Ethereum"),
        ),
        _ => panic!("unsupported market_id: {}", market_id),
    }
}

/// Check if test mode is enabled
fn is_test_mode(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::TestMode)
        .unwrap_or(false)
}

/// Get simulated price for testing
/// Returns (price, timestamp)
fn get_simulated_price(env: &Env, market_id: u32) -> (i128, u64) {
    let base_price = env
        .storage()
        .instance()
        .get(&DataKey::TestBasePrice(market_id))
        .unwrap_or(100_000_000);

    let timestamp = env.ledger().timestamp();

    // Check if fixed price mode is enabled (no oscillation)
    let fixed_price_mode: bool = env
        .storage()
        .instance()
        .get(&DataKey::FixedPriceMode)
        .unwrap_or(false);

    if fixed_price_mode {
        return (base_price, timestamp);
    }

    // === TEST PRICE OSCILLATION ===
    // Creates predictable price movement for testing funding rates and liquidations
    // Price oscillates ±10% over each hour in a sawtooth pattern

    // time_in_hour: 0-3599 seconds within the current hour
    let time_in_hour = (timestamp % 3600) as i128;

    // variation: linear ramp from 0 to 10% of base price over the hour
    // At t=0: variation=0, at t=3599: variation≈10% of base_price
    // Formula: (base_price * seconds) / 36000 gives 0-10% range
    let variation = (base_price * time_in_hour) / 36000; // Max ±10%

    // Flip direction every 30 minutes (1800 seconds)
    // Even half-hours: price increases, Odd half-hours: price decreases
    // This creates continuous oscillation without sudden jumps
    let oscillating_multiplier = if (timestamp / 1800) % 2 == 0 { 1 } else { -1 };

    let price = base_price + (variation * oscillating_multiplier);
    (price, timestamp)
}

/// Validate oracle price for staleness and bounds
#[cfg(not(test))]
fn validate_oracle_price(env: &Env, price: i128, timestamp: u64) {
    // Staleness check
    let config_manager = get_config_manager(env);
    let config_client = config_manager::Client::new(env, &config_manager);
    let staleness_threshold = config_client.price_staleness_threshold();
    let current_time = env.ledger().timestamp();

    // Reject future timestamps (clock skew, bad data, or malicious oracle)
    if timestamp > current_time {
        panic!("invalid price timestamp: {} is in the future (current: {})", timestamp, current_time);
    }

    let age = current_time - timestamp;
    if age > staleness_threshold {
        panic!(
            "stale price: age {} seconds exceeds threshold {}",
            age,
            staleness_threshold
        );
    }

    // Bounds check
    if price <= 0 {
        panic!("invalid price: must be positive");
    }

    // Sanity check: price should be reasonable (< $1 trillion)
    if price > 1_000_000_000_000_000_000 {
        panic!("invalid price: exceeds maximum bound");
    }
}

/// Validate price deviation between oracles
#[cfg(not(test))]
fn validate_price_deviation(env: &Env, price1: i128, price2: i128) {
    let config_manager = get_config_manager(env);
    let config_client = config_manager::Client::new(env, &config_manager);
    let max_deviation_bps = config_client.max_price_deviation_bps();

    // Calculate percentage deviation
    let diff = if price1 > price2 {
        price1 - price2
    } else {
        price2 - price1
    };
    let avg = (price1 + price2) / 2;
    let deviation_bps = (diff * 10000) / avg;

    if deviation_bps > max_deviation_bps {
        panic!(
            "excessive price deviation: {}bps exceeds threshold {}bps - possible manipulation",
            deviation_bps, max_deviation_bps
        );
    }
}

#[contract]
pub struct OracleIntegrator;

#[contractimpl]
impl OracleIntegrator {
    /// Initialize the OracleIntegrator contract.
    ///
    /// # Arguments
    ///
    /// * `config_manager` - Address of the ConfigManager contract
    pub fn initialize(env: Env, config_manager: Address) {
        // Prevent reinitialization
        if env.storage().instance().has(&DataKey::ConfigManager) {
            panic!("already initialized");
        }

        // Store the ConfigManager address
        env.storage()
            .instance()
            .set(&DataKey::ConfigManager, &config_manager);
    }

    /// Enable or disable test mode with base prices.
    ///
    /// # Arguments
    ///
    /// * `admin` - The administrator address (must match ConfigManager admin)
    /// * `enabled` - Whether to enable test mode
    /// * `base_prices` - Map of market_id to base price for simulation
    ///
    /// # Panics
    ///
    /// Panics if caller is not the admin
    pub fn set_test_mode(env: Env, admin: Address, enabled: bool, base_prices: Map<u32, i128>) {
        admin.require_auth();

        // Verify admin through ConfigManager (only in non-test environments)
        #[cfg(not(test))]
        {
            let config_manager = get_config_manager(&env);
            let config_client = config_manager::Client::new(&env, &config_manager);
            let admin_addr = config_client.admin();
            if admin != admin_addr {
                panic!("unauthorized");
            }
        }

        // Set test mode flag
        env.storage().instance().set(&DataKey::TestMode, &enabled);

        // Store base prices for each market
        for (market_id, price) in base_prices.iter() {
            env.storage()
                .instance()
                .set(&DataKey::TestBasePrice(market_id), &price);
        }
    }

    /// Check if test mode is enabled.
    ///
    /// # Returns
    ///
    /// True if test mode is enabled, false otherwise
    pub fn get_test_mode(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::TestMode)
            .unwrap_or(false)
    }

    /// Enable or disable fixed price mode (no oscillation).
    /// When enabled, prices will remain at base price without time-based variation.
    /// Useful for testing funding rates in isolation.
    ///
    /// # Arguments
    ///
    /// * `admin` - The administrator address
    /// * `enabled` - Whether to enable fixed price mode
    pub fn set_fixed_price_mode(env: Env, admin: Address, enabled: bool) {
        admin.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::FixedPriceMode, &enabled);
    }

    /// Get the current price for a specific asset from all oracle sources.
    ///
    /// # Arguments
    ///
    /// * `market_id` - The market identifier (0=XLM, 1=BTC, 2=ETH)
    ///
    /// # Returns
    ///
    /// The aggregated (median) price
    ///
    /// # Implementation
    ///
    /// In test mode: Returns time-based simulated price
    /// In production mode: Fetches from DIA and Reflector, validates, returns median
    pub fn get_price(env: Env, market_id: u32) -> i128 {
        // Test mode bypass
        if is_test_mode(&env) {
            let (price, _) = get_simulated_price(&env, market_id);
            return price;
        }

        // Production mode: fetch from both oracles
        #[cfg(not(test))]
        {
            let (dia_price, dia_timestamp) = Self::fetch_dia_price(env.clone(), market_id);
            let (reflector_price, reflector_timestamp) =
                Self::fetch_reflector_price(env.clone(), market_id);

            // Validate each price
            validate_oracle_price(&env, dia_price, dia_timestamp);
            validate_oracle_price(&env, reflector_price, reflector_timestamp);

            // Check deviation between oracles
            validate_price_deviation(&env, dia_price, reflector_price);

            // Calculate median (average of 2 prices)
            let median_price = (dia_price + reflector_price) / 2;

            median_price
        }

        #[cfg(test)]
        {
            // In test builds, if not in test mode, panic with clear message
            panic!("Production oracle integration not available in test mode - use set_test_mode");
        }
    }

    /// Fetch price from Pyth Network oracle.
    ///
    /// # Arguments
    ///
    /// * `asset_id` - The asset identifier
    ///
    /// # Returns
    ///
    /// Tuple of (price, confidence, timestamp)
    pub fn fetch_pyth_price(_env: Env, _asset_id: u32) -> (i128, i128, u64) {
        // TODO: Implement Pyth price fetching
        // - Call Pyth oracle contract
        // - Parse price feed data
        // - Extract price, confidence interval, and timestamp
        // - Return price data
        (0, 0, 0)
    }

    /// Fetch price from DIA oracle.
    ///
    /// # Arguments
    ///
    /// * `market_id` - The market identifier
    ///
    /// # Returns
    ///
    /// Tuple of (price, timestamp)
    pub fn fetch_dia_price(env: Env, market_id: u32) -> (i128, u64) {
        #[cfg(not(test))]
        {
            let config_manager = get_config_manager(&env);
            let config_client = config_manager::Client::new(&env, &config_manager);
            let _dia_address = config_client.dia_oracle();

            let (dia_symbol, _) = get_asset_symbol(&env, market_id);

            // TODO: Replace with actual DIA oracle contract call
            // This requires DIA oracle WASM interface
            // For now, panic with clear error message
            panic!(
                "DIA oracle integration not yet implemented - requires DIA contract interface for symbol: {:?}",
                dia_symbol
            );
        }

        #[cfg(test)]
        {
            // Test stub - should not be called in test mode
            panic!(
                "fetch_dia_price should not be called in test builds - market_id: {}",
                market_id
            );
        }
    }

    /// Fetch price from Reflector oracle.
    ///
    /// # Arguments
    ///
    /// * `market_id` - The market identifier
    ///
    /// # Returns
    ///
    /// Tuple of (price, timestamp)
    pub fn fetch_reflector_price(env: Env, market_id: u32) -> (i128, u64) {
        #[cfg(not(test))]
        {
            let config_manager = get_config_manager(&env);
            let config_client = config_manager::Client::new(&env, &config_manager);
            let _reflector_address = config_client.reflector_oracle();

            let (_, reflector_symbol) = get_asset_symbol(&env, market_id);

            // TODO: Use sep-40-oracle crate for Reflector integration
            // This requires proper sep-40-oracle client setup
            // For now, panic with clear error message
            panic!(
                "Reflector oracle integration not yet implemented - requires sep-40-oracle setup for symbol: {:?}",
                reflector_symbol
            );
        }

        #[cfg(test)]
        {
            // Test stub - should not be called in test mode
            panic!(
                "fetch_reflector_price should not be called in test builds - market_id: {}",
                market_id
            );
        }
    }

    /// Validate a price feed for staleness and bounds.
    ///
    /// # Arguments
    ///
    /// * `price` - The price to validate
    /// * `timestamp` - The price timestamp
    /// * `min_price` - Minimum acceptable price
    /// * `max_price` - Maximum acceptable price
    ///
    /// # Returns
    ///
    /// True if price is valid, false otherwise
    pub fn validate_price(
        _env: Env,
        _price: i128,
        _timestamp: u64,
        _min_price: i128,
        _max_price: i128,
    ) -> bool {
        // TODO: Implement price validation
        // - Check timestamp is within staleness limit (60 seconds)
        // - Verify price is within min/max bounds
        // - Ensure price is positive and reasonable
        // - Return validation result
        true
    }

    /// Calculate median price from multiple oracle sources.
    ///
    /// # Arguments
    ///
    /// * `price1` - Price from first oracle
    /// * `price2` - Price from second oracle
    ///
    /// # Returns
    ///
    /// The median price (average of 2 prices)
    pub fn calculate_median(_env: Env, price1: i128, price2: i128) -> i128 {
        // With 2 oracles, median equals average
        // Use checked arithmetic to prevent overflow
        price1
            .checked_add(price2)
            .expect("price addition overflow")
            .checked_div(2)
            .expect("division error")
    }

    /// Check if price deviation between sources exceeds threshold.
    ///
    /// # Arguments
    ///
    /// * `prices` - Array of prices from different oracles
    /// * `threshold_bps` - Maximum allowed deviation in basis points
    ///
    /// # Returns
    ///
    /// True if deviation is acceptable, false if excessive
    pub fn check_price_deviation(_env: Env, _threshold_bps: u32) -> bool {
        // TODO: Implement deviation check
        // - Calculate max and min prices
        // - Calculate deviation percentage
        // - Compare against threshold
        // - Return result
        true
    }

    /// Get the health status of all oracle sources.
    ///
    /// # Returns
    ///
    /// Tuple of (pyth_healthy, dia_healthy, reflector_healthy)
    pub fn get_oracle_health(_env: Env) -> (bool, bool, bool) {
        // TODO: Implement oracle health check
        // - Check last successful update time for each oracle
        // - Verify oracles are responding
        // - Return health status for each
        (true, true, true)
    }

    /// Update the cached price for an asset.
    ///
    /// Called periodically by keeper bots to maintain fresh prices.
    ///
    /// # Arguments
    ///
    /// * `asset_id` - The asset identifier
    pub fn update_cached_price(_env: Env, _asset_id: u32) {
        // TODO: Implement price cache update
        // - Fetch fresh prices from all oracles
        // - Calculate and validate median
        // - Update temporary storage with new price
        // - Set TTL for cache entry (10-60 seconds)
        // - Emit price updated event
    }
}

#[cfg(test)]
mod test;

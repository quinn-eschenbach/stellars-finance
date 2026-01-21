#![no_std]

//! # Market Manager Contract
//!
//! Manages perpetual futures markets for the Stellars Finance protocol. Handles open interest
//! tracking and funding rate calculations for each market (XLM, BTC, ETH perpetuals).
//!
//! ## Key Features
//! - **Market Creation**: Admin can create new perpetual markets with configurable max OI
//! - **Open Interest Tracking**: Tracks long and short OI separately for each market
//! - **Funding Rate Calculation**: Calculates funding rates based on market imbalance
//! - **Market Controls**: Admin can pause/unpause markets to halt new position openings
//!
//! ## Funding Rate Mechanism
//! Funding payments balance long and short positions by transferring value from the
//! dominant side to the minority side:
//! - Positive funding rate: Longs pay shorts (when long OI > short OI)
//! - Negative funding rate: Shorts pay longs (when short OI > long OI)
//!
//! The funding rate uses a quadratic formula to increase pressure as imbalance grows:
//! `funding_rate = base_rate * (imbalance_ratio)^2`
//!
//! ## Cumulative Funding
//! Funding is tracked cumulatively (bps * seconds) to allow precise per-position
//! calculations without iterating through all positions on each update.
//!
//! ## Usage
//! - Admin creates markets via `create_market()`
//! - Keeper bot calls `update_funding_rate()` every 60 seconds
//! - PositionManager calls `update_open_interest()` when positions open/close

use soroban_sdk::{
    contract, contractevent, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

mod config_manager {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/config_manager.wasm");
}

// Data Structures

#[contracttype]
#[derive(Clone, Debug)]
pub struct Market {
    pub market_id: u32,
    pub ticker: Symbol, // Oracle ticker symbol (e.g., "XLM", "BTC", "ETH")
    pub max_open_interest: u128,
    pub long_open_interest: u128,
    pub short_open_interest: u128,
    pub funding_rate: i128, // bps per hour
    pub last_funding_update: u64,
    pub cumulative_funding_long: i128, // Total funding paid by longs
    pub cumulative_funding_short: i128, // Total funding paid by shorts
    pub is_paused: bool,
    pub base_funding_rate: i128, // Default: 100 (0.01% per hour)
    pub max_funding_rate: i128,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    ConfigManager,
    Admin,
    Market(u32),
    MarketCount,
    AuthorizedPositionManager,
}

// Events

#[contractevent]
pub struct MarketCreatedEvent {
    pub market_id: u32,
    pub max_oi: u128,
}

#[contractevent]
pub struct FundingRateUpdatedEvent {
    pub market_id: u32,
    pub funding_rate: i128,
    pub long_oi: u128,
    pub short_oi: u128,
}

#[contractevent]
pub struct OIUpdatedEvent {
    pub market_id: u32,
    pub long_oi: u128,
    pub short_oi: u128,
}

// Helper Functions

fn get_config_manager(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::ConfigManager)
        .expect("ConfigManager not initialized")
}

fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("Admin not set")
}

fn require_admin(env: &Env, admin: &Address) {
    admin.require_auth();
    let stored_admin = get_admin(env);
    if admin != &stored_admin {
        panic!("unauthorized: not admin");
    }
}

fn get_market(env: &Env, market_id: u32) -> Market {
    env.storage()
        .instance()
        .get(&DataKey::Market(market_id))
        .expect("market not found")
}

fn set_market(env: &Env, market: &Market) {
    env.storage()
        .instance()
        .set(&DataKey::Market(market.market_id), market);
}

fn require_position_manager(env: &Env, caller: &Address) {
    caller.require_auth();
    if let Some(authorized) = env
        .storage()
        .instance()
        .get::<DataKey, Address>(&DataKey::AuthorizedPositionManager)
    {
        if caller != &authorized {
            panic!("unauthorized: not position manager");
        }
    } else {
        panic!("position manager not set");
    }
}

#[contract]
pub struct MarketManager;

#[contractimpl]
impl MarketManager {
    /// Initialize the MarketManager contract.
    ///
    /// # Arguments
    ///
    /// * `config_manager` - Address of the ConfigManager contract
    /// * `admin` - Address of the admin
    pub fn initialize(env: Env, config_manager: Address, admin: Address) {
        if env.storage().instance().has(&DataKey::ConfigManager) {
            panic!("already initialized");
        }

        env.storage()
            .instance()
            .set(&DataKey::ConfigManager, &config_manager);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::MarketCount, &0u32);
    }

    /// Set the authorized PositionManager contract.
    ///
    /// # Arguments
    ///
    /// * `admin` - Address of the admin
    /// * `position_manager` - Address of the PositionManager contract
    pub fn set_position_manager(env: Env, admin: Address, position_manager: Address) {
        require_admin(&env, &admin);
        env.storage()
            .instance()
            .set(&DataKey::AuthorizedPositionManager, &position_manager);
    }

    /// Create a new perpetual market.
    ///
    /// # Arguments
    ///
    /// * `admin` - Address of the admin
    /// * `market_id` - Unique identifier for the market (e.g., 0 = XLM-PERP)
    /// * `ticker` - Oracle ticker symbol (e.g., "XLM", "BTC", "ETH")
    /// * `max_open_interest` - Maximum total open interest allowed for this market
    /// * `max_funding_rate` - Maximum funding rate per hour (in basis points)
    pub fn create_market(
        env: Env,
        admin: Address,
        market_id: u32,
        ticker: Symbol,
        max_open_interest: u128,
        max_funding_rate: i128,
    ) {
        require_admin(&env, &admin);

        // Verify market doesn't already exist
        if env.storage().instance().has(&DataKey::Market(market_id)) {
            panic!("market already exists");
        }

        // Create market with defaults
        let market = Market {
            market_id,
            ticker,
            max_open_interest,
            long_open_interest: 0,
            short_open_interest: 0,
            funding_rate: 0,
            last_funding_update: env.ledger().timestamp(),
            cumulative_funding_long: 0,
            cumulative_funding_short: 0,
            is_paused: false,
            base_funding_rate: 100, // 1% per hour = 100 basis points
            max_funding_rate,
        };

        set_market(&env, &market);

        // Increment market count
        let count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::MarketCount)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::MarketCount, &(count + 1));

        // Emit event
        MarketCreatedEvent {
            market_id,
            max_oi: max_open_interest,
        }
        .publish(&env);
    }

    /// Update the funding rate for a market.
    ///
    /// Called every 60 seconds by the keeper bot.
    /// Calculates funding rate based on market imbalance and updates cumulative funding.
    /// Funding rate is expressed in basis points per hour.
    ///
    /// # Arguments
    ///
    /// * `caller` - Address calling this function
    /// * `market_id` - The market identifier
    pub fn update_funding_rate(env: Env, caller: Address, market_id: u32) {
        caller.require_auth();

        let mut market = get_market(&env, market_id);

        // Check if market is paused - funding updates should be suspended
        if market.is_paused {
            panic!("cannot update funding rate for paused market");
        }

        // Verify funding interval has passed (60s from ConfigManager)
        let config_manager = get_config_manager(&env);
        let config_client = config_manager::Client::new(&env, &config_manager);
        let funding_interval = config_client.funding_interval();

        let now = env.ledger().timestamp();
        let time_elapsed = now - market.last_funding_update;

        if time_elapsed < funding_interval {
            panic!("funding interval not elapsed");
        }

        // Calculate total OI
        let total_oi = market
            .long_open_interest
            .checked_add(market.short_open_interest)
            .expect("OI overflow");

        if total_oi == 0 {
            // No open interest, funding rate stays at 0
            market.last_funding_update = now;
            set_market(&env, &market);
            return;
        }

        // === FUNDING RATE CALCULATION ===
        // The funding rate incentivizes balance between longs and shorts by making
        // the dominant side pay the minority side. Uses quadratic scaling to increase
        // pressure as imbalance grows.

        // Step 1: Calculate imbalance ratio as (long_oi - short_oi) / total_oi
        // Positive = longs dominate, Negative = shorts dominate
        let oi_diff = (market.long_open_interest as i128) - (market.short_open_interest as i128);

        // Convert to basis points (10000 bps = 100%)
        // Example: If long=60, short=40, total=100, then imbalance = 2000 bps (20%)
        let imbalance_ratio_bps = (oi_diff * 10000) / (total_oi as i128);

        // Step 2: Apply quadratic scaling - funding pressure grows with square of imbalance
        // This creates gentle pressure at small imbalances but strong pressure at large ones
        // Example: 20% imbalance (2000 bps) -> squared = (2000 * 2000) / 10000 = 400 bps
        let imbalance_squared = (imbalance_ratio_bps * imbalance_ratio_bps) / 10000;

        // Step 3: Scale by base funding rate (default 100 bps = 1% per hour)
        // funding_rate = base_rate * imbalance_squared / 10000
        // Example: 100 * 400 / 10000 = 4 bps per hour
        let mut funding_rate = (market.base_funding_rate * imbalance_squared) / 10000;

        // Step 4: Restore direction - squaring loses the sign, so reapply based on imbalance
        // Positive imbalance (longs > shorts) = positive rate = longs pay shorts
        // Negative imbalance (shorts > longs) = negative rate = shorts pay longs
        if imbalance_ratio_bps < 0 {
            funding_rate = -funding_rate;
        }

        // Cap at max funding rate
        if funding_rate > market.max_funding_rate {
            funding_rate = market.max_funding_rate;
        } else if funding_rate < -market.max_funding_rate {
            funding_rate = -market.max_funding_rate;
        }

        // === CUMULATIVE FUNDING ACCUMULATION ===
        // Store funding as (bps_per_hour * seconds_elapsed) to preserve precision
        // Division by 3600 (seconds per hour) happens in PositionManager's PnL calculation
        // This avoids integer truncation that would occur if we divided here
        // Example: 4 bps/hour * 60 seconds = 240 bps·seconds stored
        let total_funding = funding_rate * (time_elapsed as i128);

        // Track cumulative funding separately for longs and shorts
        // - cumulative_funding_long: total funding longs have paid (when rate > 0)
        // - cumulative_funding_short: total funding shorts have paid (when rate < 0)
        // Positions calculate their owed funding by comparing current cumulative vs entry snapshot
        if funding_rate > 0 {
            // Positive rate: longs pay shorts
            market.cumulative_funding_long += total_funding;
        } else if funding_rate < 0 {
            // Negative rate: shorts pay longs
            market.cumulative_funding_short += total_funding.abs();
        }

        // Update market state
        market.funding_rate = funding_rate;
        market.last_funding_update = now;
        set_market(&env, &market);

        // Emit event
        FundingRateUpdatedEvent {
            market_id,
            funding_rate,
            long_oi: market.long_open_interest,
            short_oi: market.short_open_interest,
        }
        .publish(&env);
    }

    /// Get the current funding rate for a market.
    ///
    /// # Arguments
    ///
    /// * `market_id` - The market identifier
    ///
    /// # Returns
    ///
    /// The current funding rate (in basis points per hour)
    pub fn get_funding_rate(env: Env, market_id: u32) -> i128 {
        let market = get_market(&env, market_id);
        market.funding_rate
    }

    /// Get cumulative funding for a position side.
    ///
    /// # Arguments
    ///
    /// * `market_id` - The market identifier
    /// * `is_long` - True if long position, false if short
    ///
    /// # Returns
    ///
    /// The cumulative funding paid by the specified side
    pub fn get_cumulative_funding(env: Env, market_id: u32, is_long: bool) -> i128 {
        let market = get_market(&env, market_id);
        if is_long {
            market.cumulative_funding_long
        } else {
            market.cumulative_funding_short
        }
    }

    /// Update open interest when positions are opened or closed.
    ///
    /// # Arguments
    ///
    /// * `position_manager` - Address of the PositionManager contract
    /// * `market_id` - The market identifier
    /// * `is_long` - True if long position, false if short
    /// * `size_delta` - Change in position size (positive = increase, negative = decrease)
    pub fn update_open_interest(
        env: Env,
        position_manager: Address,
        market_id: u32,
        is_long: bool,
        size_delta: i128,
    ) {
        require_position_manager(&env, &position_manager);

        let mut market = get_market(&env, market_id);

        if is_long {
            // Update long OI
            if size_delta > 0 {
                // Opening or increasing position
                let new_long_oi = market
                    .long_open_interest
                    .checked_add(size_delta as u128)
                    .expect("long OI overflow");

                // Check against max OI limit
                if new_long_oi > market.max_open_interest {
                    panic!("exceeds max open interest");
                }

                market.long_open_interest = new_long_oi;
            } else {
                // Closing or decreasing position
                let decrease = (-size_delta) as u128;
                if decrease > market.long_open_interest {
                    panic!("cannot reduce OI below zero");
                }
                market.long_open_interest -= decrease;
            }
        } else {
            // Update short OI
            if size_delta > 0 {
                let new_short_oi = market
                    .short_open_interest
                    .checked_add(size_delta as u128)
                    .expect("short OI overflow");

                if new_short_oi > market.max_open_interest {
                    panic!("exceeds max open interest");
                }

                market.short_open_interest = new_short_oi;
            } else {
                let decrease = (-size_delta) as u128;
                if decrease > market.short_open_interest {
                    panic!("cannot reduce OI below zero");
                }
                market.short_open_interest -= decrease;
            }
        }

        set_market(&env, &market);

        // Emit event
        OIUpdatedEvent {
            market_id,
            long_oi: market.long_open_interest,
            short_oi: market.short_open_interest,
        }
        .publish(&env);
    }

    /// Get the current open interest for a market.
    ///
    /// # Arguments
    ///
    /// * `market_id` - The market identifier
    ///
    /// # Returns
    ///
    /// Tuple of (long_open_interest, short_open_interest)
    pub fn get_open_interest(env: Env, market_id: u32) -> (u128, u128) {
        let market = get_market(&env, market_id);
        (market.long_open_interest, market.short_open_interest)
    }

    /// Get the ticker symbol for a market.
    ///
    /// # Arguments
    ///
    /// * `market_id` - The market identifier
    ///
    /// # Returns
    ///
    /// The ticker symbol (e.g., "XLM", "BTC", "ETH")
    pub fn get_market_ticker(env: Env, market_id: u32) -> Symbol {
        let market = get_market(&env, market_id);
        market.ticker
    }

    /// Pause a market to prevent new positions from being opened.
    ///
    /// # Arguments
    ///
    /// * `admin` - Address of the admin
    /// * `market_id` - The market identifier
    pub fn pause_market(env: Env, admin: Address, market_id: u32) {
        require_admin(&env, &admin);

        let mut market = get_market(&env, market_id);
        market.is_paused = true;
        set_market(&env, &market);

        env.events().publish((symbol_short!("paused"),), market_id);
    }

    /// Unpause a market to allow new positions.
    ///
    /// # Arguments
    ///
    /// * `admin` - Address of the admin
    /// * `market_id` - The market identifier
    pub fn unpause_market(env: Env, admin: Address, market_id: u32) {
        require_admin(&env, &admin);

        let mut market = get_market(&env, market_id);
        market.is_paused = false;
        set_market(&env, &market);

        env.events()
            .publish((symbol_short!("unpaused"),), market_id);
    }

    /// Check if a market is currently paused.
    ///
    /// # Arguments
    ///
    /// * `market_id` - The market identifier
    ///
    /// # Returns
    ///
    /// True if market is paused, false otherwise
    pub fn is_market_paused(env: Env, market_id: u32) -> bool {
        let market = get_market(&env, market_id);
        market.is_paused
    }

    /// Check if a new position can be opened based on OI limits.
    ///
    /// # Arguments
    ///
    /// * `market_id` - The market identifier
    /// * `is_long` - True if long position, false if short
    /// * `size` - The size of the position to open
    ///
    /// # Returns
    ///
    /// True if position can be opened, false otherwise
    pub fn can_open_position(env: Env, market_id: u32, is_long: bool, size: u128) -> bool {
        let market = match env
            .storage()
            .instance()
            .get::<DataKey, Market>(&DataKey::Market(market_id))
        {
            Some(m) => m,
            None => return false, // Market doesn't exist
        };

        // Check if market is paused
        if market.is_paused {
            return false;
        }

        // Check if adding this size would exceed max OI
        let current_oi = if is_long {
            market.long_open_interest
        } else {
            market.short_open_interest
        };

        let new_oi = current_oi + size;
        if new_oi > market.max_open_interest {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod test;

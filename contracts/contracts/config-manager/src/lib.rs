#![no_std]

//! # Config Manager Contract
//!
//! Central configuration hub for the Stellars Finance protocol. This contract stores
//! all system parameters and maintains a registry of other contract addresses.
//!
//! ## Key Features
//! - **Contract Registry**: Stores addresses of all protocol contracts (LiquidityPool,
//!   PositionManager, MarketManager, OracleIntegrator, Token, DIA/Reflector oracles)
//! - **Trading Parameters**: Min/max leverage (default 5-20x), minimum position size
//! - **Fee Parameters**: Maker fee, taker fee, liquidation fee (all in basis points)
//! - **Risk Parameters**: Liquidation threshold, maintenance margin, max price deviation
//! - **Time Parameters**: Funding interval (60s), price staleness threshold
//! - **Liquidity Parameters**: Max utilization ratio (80%), min reserve ratio (20%)
//!
//! ## Access Control
//! All configuration changes require admin authorization. The admin can be transferred
//! to a new address via `set_admin()`.
//!
//! ## Usage
//! Other contracts in the protocol import this contract to read configuration values
//! and resolve contract addresses. This creates a single source of truth for all
//! protocol settings.

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    // Contract Registry
    LiquidityPoolContract,
    PositionManagerContract,
    MarketManagerContract,
    OracleIntegratorContract,
    CustomOracleContract,
    DiaOracleContract,
    ReflectorOracleContract,
    TokenContract,
    // Trading parameters
    MinLeverage,
    MaxLeverage,
    MinPositionSize,
    // Fee parameters
    MakerFeeBps,
    TakerFeeBps,
    LiquidationFeeBps,
    // Risk parameters
    LiquidationThreshold,
    MaintenanceMargin,
    MaxPriceDeviationBps,
    // Time parameters
    FundingInterval,
    PriceStalenessThreshold,
    // Liquidity parameters
    MaxUtilizationRatio,
    MinLiquidityReserveRatio,
    // Borrowing parameters
    BorrowRatePerSecond,
}

#[contract]
pub struct ConfigManager;

// Helper functions for storage access
fn get_admin(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Admin).unwrap()
}

fn put_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin);
}

fn get_config_value(e: &Env, key: &DataKey) -> i128 {
    e.storage().instance().get(key).unwrap_or(0)
}

fn put_config_value(e: &Env, key: &DataKey, value: i128) {
    e.storage().instance().set(key, &value);
}

fn get_time_config_value(e: &Env, key: &DataKey) -> u64 {
    e.storage().instance().get(key).unwrap_or(0)
}

fn put_time_config_value(e: &Env, key: &DataKey, value: u64) {
    e.storage().instance().set(key, &value);
}

fn require_admin(e: &Env, admin: &Address) {
    admin.require_auth();
    let stored_admin = get_admin(e);
    if admin != &stored_admin {
        panic!("unauthorized");
    }
}

fn get_contract_address(e: &Env, key: &DataKey) -> Address {
    e.storage().instance().get(key).unwrap()
}

fn put_contract_address(e: &Env, key: &DataKey, address: &Address) {
    e.storage().instance().set(key, address);
}

#[contractimpl]
impl ConfigManager {
    /// Initialize the configuration contract with admin.
    ///
    /// # Arguments
    ///
    /// * `admin` - The administrator address
    ///
    /// # Panics
    ///
    /// Panics if already initialized
    pub fn initialize(env: Env, admin: Address) {
        // Verify not already initialized
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }

        // Require the admin to authorize this initialization
        admin.require_auth();

        // Set admin
        put_admin(&env, &admin);

        // Set default configuration values
        // Trading parameters
        put_config_value(&env, &DataKey::MinLeverage, 5);
        put_config_value(&env, &DataKey::MaxLeverage, 20);
        put_config_value(&env, &DataKey::MinPositionSize, 10_000_000);

        // Fee parameters (in basis points)
        put_config_value(&env, &DataKey::MakerFeeBps, 2);
        put_config_value(&env, &DataKey::TakerFeeBps, 5);
        put_config_value(&env, &DataKey::LiquidationFeeBps, 50);

        // Risk parameters
        put_config_value(&env, &DataKey::LiquidationThreshold, 9000);
        put_config_value(&env, &DataKey::MaintenanceMargin, 5000);
        put_config_value(&env, &DataKey::MaxPriceDeviationBps, 500);

        // Time parameters
        put_time_config_value(&env, &DataKey::FundingInterval, 60);
        put_time_config_value(&env, &DataKey::PriceStalenessThreshold, 60);

        // Liquidity parameters (in basis points)
        put_config_value(&env, &DataKey::MaxUtilizationRatio, 8000); // 80%
        put_config_value(&env, &DataKey::MinLiquidityReserveRatio, 2000); // 20%

        // Borrowing parameters (rate per second scaled by 1e7)
        // Default: 1 = 0.0000001% per second (~3.15% APR)
        put_config_value(&env, &DataKey::BorrowRatePerSecond, 1);
    }

    /// Update the admin address.
    ///
    /// # Arguments
    ///
    /// * `current_admin` - The current administrator address
    /// * `new_admin` - The new administrator address
    ///
    /// # Panics
    ///
    /// Panics if caller is not the current admin
    pub fn set_admin(env: Env, current_admin: Address, new_admin: Address) {
        // Verify current admin authorization
        require_admin(&env, &current_admin);

        // Set new admin
        put_admin(&env, &new_admin);
    }

    /// Get the current admin address.
    ///
    /// # Returns
    ///
    /// The administrator address
    pub fn admin(env: Env) -> Address {
        get_admin(&env)
    }

    /// Get minimum leverage limit.
    ///
    /// # Returns
    ///
    /// Minimum leverage (default: 5)
    pub fn min_leverage(env: Env) -> i128 {
        get_config_value(&env, &DataKey::MinLeverage)
    }

    /// Get maximum leverage limit.
    ///
    /// # Returns
    ///
    /// Maximum leverage (default: 20)
    pub fn max_leverage(env: Env) -> i128 {
        get_config_value(&env, &DataKey::MaxLeverage)
    }

    /// Get minimum position size.
    ///
    /// # Returns
    ///
    /// Minimum position size in base units
    pub fn min_position_size(env: Env) -> i128 {
        get_config_value(&env, &DataKey::MinPositionSize)
    }

    /// Get maker fee in basis points.
    ///
    /// # Returns
    ///
    /// Maker fee in basis points (default: 2)
    pub fn maker_fee_bps(env: Env) -> i128 {
        get_config_value(&env, &DataKey::MakerFeeBps)
    }

    /// Get taker fee in basis points.
    ///
    /// # Returns
    ///
    /// Taker fee in basis points (default: 5)
    pub fn taker_fee_bps(env: Env) -> i128 {
        get_config_value(&env, &DataKey::TakerFeeBps)
    }

    /// Get liquidation fee in basis points.
    ///
    /// # Returns
    ///
    /// Liquidation fee in basis points (default: 50)
    pub fn liquidation_fee_bps(env: Env) -> i128 {
        get_config_value(&env, &DataKey::LiquidationFeeBps)
    }

    /// Get liquidation threshold in basis points.
    ///
    /// # Returns
    ///
    /// Liquidation threshold in basis points (default: 9000)
    pub fn liquidation_threshold(env: Env) -> i128 {
        get_config_value(&env, &DataKey::LiquidationThreshold)
    }

    /// Get maintenance margin in basis points.
    ///
    /// # Returns
    ///
    /// Maintenance margin in basis points (default: 5000)
    pub fn maintenance_margin(env: Env) -> i128 {
        get_config_value(&env, &DataKey::MaintenanceMargin)
    }

    /// Get maximum price deviation in basis points.
    ///
    /// # Returns
    ///
    /// Maximum price deviation in basis points (default: 500)
    pub fn max_price_deviation_bps(env: Env) -> i128 {
        get_config_value(&env, &DataKey::MaxPriceDeviationBps)
    }

    /// Get funding interval in seconds.
    ///
    /// # Returns
    ///
    /// Funding interval in seconds (default: 60)
    pub fn funding_interval(env: Env) -> u64 {
        get_time_config_value(&env, &DataKey::FundingInterval)
    }

    /// Get price staleness threshold in seconds.
    ///
    /// # Returns
    ///
    /// Price staleness threshold in seconds (default: 60)
    pub fn price_staleness_threshold(env: Env) -> u64 {
        get_time_config_value(&env, &DataKey::PriceStalenessThreshold)
    }

    // Contract Registry Functions

    /// Set the Liquidity Pool contract address.
    ///
    /// # Arguments
    ///
    /// * `admin` - The administrator address
    /// * `contract` - The Liquidity Pool contract address
    ///
    /// # Panics
    ///
    /// Panics if caller is not the admin
    pub fn set_liquidity_pool(env: Env, admin: Address, contract: Address) {
        require_admin(&env, &admin);
        put_contract_address(&env, &DataKey::LiquidityPoolContract, &contract);
    }

    /// Get the Liquidity Pool contract address.
    ///
    /// # Returns
    ///
    /// The Liquidity Pool contract address
    pub fn liquidity_pool(env: Env) -> Address {
        get_contract_address(&env, &DataKey::LiquidityPoolContract)
    }

    /// Set the Position Manager contract address.
    ///
    /// # Arguments
    ///
    /// * `admin` - The administrator address
    /// * `contract` - The Position Manager contract address
    ///
    /// # Panics
    ///
    /// Panics if caller is not the admin
    pub fn set_position_manager(env: Env, admin: Address, contract: Address) {
        require_admin(&env, &admin);
        put_contract_address(&env, &DataKey::PositionManagerContract, &contract);
    }

    /// Get the Position Manager contract address.
    ///
    /// # Returns
    ///
    /// The Position Manager contract address
    pub fn position_manager(env: Env) -> Address {
        get_contract_address(&env, &DataKey::PositionManagerContract)
    }

    /// Set the Market Manager contract address.
    ///
    /// # Arguments
    ///
    /// * `admin` - The administrator address
    /// * `contract` - The Market Manager contract address
    ///
    /// # Panics
    ///
    /// Panics if caller is not the admin
    pub fn set_market_manager(env: Env, admin: Address, contract: Address) {
        require_admin(&env, &admin);
        put_contract_address(&env, &DataKey::MarketManagerContract, &contract);
    }

    /// Get the Market Manager contract address.
    ///
    /// # Returns
    ///
    /// The Market Manager contract address
    pub fn market_manager(env: Env) -> Address {
        get_contract_address(&env, &DataKey::MarketManagerContract)
    }

    /// Set the Oracle Integrator contract address.
    ///
    /// # Arguments
    ///
    /// * `admin` - The administrator address
    /// * `contract` - The Oracle Integrator contract address
    ///
    /// # Panics
    ///
    /// Panics if caller is not the admin
    pub fn set_oracle_integrator(env: Env, admin: Address, contract: Address) {
        require_admin(&env, &admin);
        put_contract_address(&env, &DataKey::OracleIntegratorContract, &contract);
    }

    /// Get the Oracle Integrator contract address.
    ///
    /// # Returns
    ///
    /// The Oracle Integrator contract address
    pub fn oracle_integrator(env: Env) -> Address {
        get_contract_address(&env, &DataKey::OracleIntegratorContract)
    }

    /// Set the Custom Oracle contract address.
    pub fn set_custom_oracle(env: Env, admin: Address, contract: Address) {
        require_admin(&env, &admin);
        put_contract_address(&env, &DataKey::CustomOracleContract, &contract);
    }

    /// Get the Custom Oracle contract address.
    pub fn custom_oracle(env: Env) -> Address {
        get_contract_address(&env, &DataKey::CustomOracleContract)
    }

    /// Set the Token contract address.
    ///
    /// # Arguments
    ///
    /// * `admin` - The administrator address
    /// * `contract` - The Token contract address
    ///
    /// # Panics
    ///
    /// Panics if caller is not the admin
    pub fn set_token(env: Env, admin: Address, contract: Address) {
        require_admin(&env, &admin);
        put_contract_address(&env, &DataKey::TokenContract, &contract);
    }

    /// Get the Token contract address.
    ///
    /// # Returns
    ///
    /// The Token contract address
    pub fn token(env: Env) -> Address {
        get_contract_address(&env, &DataKey::TokenContract)
    }

    /// Set the DIA Oracle contract address.
    ///
    /// # Arguments
    ///
    /// * `admin` - The administrator address
    /// * `contract` - The DIA Oracle contract address
    ///
    /// # Panics
    ///
    /// Panics if caller is not the admin
    pub fn set_dia_oracle(env: Env, admin: Address, contract: Address) {
        require_admin(&env, &admin);
        put_contract_address(&env, &DataKey::DiaOracleContract, &contract);
    }

    /// Get the DIA Oracle contract address.
    ///
    /// # Returns
    ///
    /// The DIA Oracle contract address
    pub fn dia_oracle(env: Env) -> Address {
        get_contract_address(&env, &DataKey::DiaOracleContract)
    }

    /// Set the Reflector Oracle contract address.
    ///
    /// # Arguments
    ///
    /// * `admin` - The administrator address
    /// * `contract` - The Reflector Oracle contract address
    ///
    /// # Panics
    ///
    /// Panics if caller is not the admin
    pub fn set_reflector_oracle(env: Env, admin: Address, contract: Address) {
        require_admin(&env, &admin);
        put_contract_address(&env, &DataKey::ReflectorOracleContract, &contract);
    }

    /// Get the Reflector Oracle contract address.
    ///
    /// # Returns
    ///
    /// The Reflector Oracle contract address
    pub fn reflector_oracle(env: Env) -> Address {
        get_contract_address(&env, &DataKey::ReflectorOracleContract)
    }

    /// Get maximum pool utilization ratio in basis points.
    ///
    /// # Returns
    ///
    /// Maximum utilization ratio in basis points (default: 8000 = 80%)
    pub fn max_utilization_ratio(env: Env) -> i128 {
        get_config_value(&env, &DataKey::MaxUtilizationRatio)
    }

    /// Set maximum pool utilization ratio in basis points.
    ///
    /// # Arguments
    ///
    /// * `admin` - The administrator address
    /// * `ratio` - The maximum utilization ratio in basis points (e.g., 8000 = 80%)
    ///
    /// # Panics
    ///
    /// Panics if caller is not the admin or ratio is invalid
    pub fn set_max_utilization_ratio(env: Env, admin: Address, ratio: i128) {
        require_admin(&env, &admin);
        if ratio < 0 || ratio > 10000 {
            panic!("invalid utilization ratio");
        }
        put_config_value(&env, &DataKey::MaxUtilizationRatio, ratio);
    }

    /// Get minimum liquidity reserve ratio in basis points.
    ///
    /// # Returns
    ///
    /// Minimum reserve ratio in basis points (default: 2000 = 20%)
    pub fn min_liquidity_reserve_ratio(env: Env) -> i128 {
        get_config_value(&env, &DataKey::MinLiquidityReserveRatio)
    }

    /// Set minimum liquidity reserve ratio in basis points.
    ///
    /// # Arguments
    ///
    /// * `admin` - The administrator address
    /// * `ratio` - The minimum reserve ratio in basis points (e.g., 2000 = 20%)
    ///
    /// # Panics
    ///
    /// Panics if caller is not the admin or ratio is invalid
    pub fn set_min_liquidity_reserve_ratio(env: Env, admin: Address, ratio: i128) {
        require_admin(&env, &admin);
        if ratio < 0 || ratio > 10000 {
            panic!("invalid reserve ratio");
        }
        put_config_value(&env, &DataKey::MinLiquidityReserveRatio, ratio);
    }

    /// Get borrow rate per second (scaled by 1e7).
    ///
    /// # Returns
    ///
    /// Borrow rate per second for calculating borrowing fees
    pub fn borrow_rate_per_second(env: Env) -> i128 {
        get_config_value(&env, &DataKey::BorrowRatePerSecond)
    }

    /// Set borrow rate per second.
    ///
    /// # Arguments
    ///
    /// * `admin` - The administrator address
    /// * `rate` - Borrow rate per second (scaled by 1e7, must be >= 0)
    ///
    /// # Panics
    ///
    /// Panics if caller is not the admin or rate is negative
    pub fn set_borrow_rate_per_second(env: Env, admin: Address, rate: i128) {
        require_admin(&env, &admin);
        if rate < 0 {
            panic!("borrow rate must be >= 0");
        }
        put_config_value(&env, &DataKey::BorrowRatePerSecond, rate);
    }

    /// Set leverage limits.
    ///
    /// # Arguments
    ///
    /// * `admin` - The administrator address
    /// * `min_leverage` - Minimum leverage (must be >= 1)
    /// * `max_leverage` - Maximum leverage (must be > min_leverage and <= 100)
    ///
    /// # Panics
    ///
    /// Panics if caller is not the admin or limits are invalid
    pub fn set_leverage_limits(env: Env, admin: Address, min_leverage: i128, max_leverage: i128) {
        require_admin(&env, &admin);
        if min_leverage < 1 {
            panic!("min leverage must be >= 1");
        }
        if max_leverage <= min_leverage {
            panic!("max leverage must be > min leverage");
        }
        if max_leverage > 100 {
            panic!("max leverage must be <= 100");
        }
        put_config_value(&env, &DataKey::MinLeverage, min_leverage);
        put_config_value(&env, &DataKey::MaxLeverage, max_leverage);
    }

    /// Set minimum position size.
    ///
    /// # Arguments
    ///
    /// * `admin` - The administrator address
    /// * `size` - Minimum position size in base units (must be > 0)
    ///
    /// # Panics
    ///
    /// Panics if caller is not the admin or size is invalid
    pub fn set_min_position_size(env: Env, admin: Address, size: i128) {
        require_admin(&env, &admin);
        if size <= 0 {
            panic!("min position size must be > 0");
        }
        put_config_value(&env, &DataKey::MinPositionSize, size);
    }

    /// Set fee parameters in basis points.
    ///
    /// # Arguments
    ///
    /// * `admin` - The administrator address
    /// * `maker_fee` - Maker fee in basis points (max 1000 = 10%)
    /// * `taker_fee` - Taker fee in basis points (max 1000 = 10%)
    /// * `liquidation_fee` - Liquidation fee in basis points (max 1000 = 10%)
    ///
    /// # Panics
    ///
    /// Panics if caller is not the admin or fees are invalid
    pub fn set_fees(env: Env, admin: Address, maker_fee: i128, taker_fee: i128, liquidation_fee: i128) {
        require_admin(&env, &admin);
        if maker_fee < 0 || maker_fee > 1000 {
            panic!("maker fee must be 0-1000 bps");
        }
        if taker_fee < 0 || taker_fee > 1000 {
            panic!("taker fee must be 0-1000 bps");
        }
        if liquidation_fee < 0 || liquidation_fee > 1000 {
            panic!("liquidation fee must be 0-1000 bps");
        }
        put_config_value(&env, &DataKey::MakerFeeBps, maker_fee);
        put_config_value(&env, &DataKey::TakerFeeBps, taker_fee);
        put_config_value(&env, &DataKey::LiquidationFeeBps, liquidation_fee);
    }

    /// Set risk parameters.
    ///
    /// # Arguments
    ///
    /// * `admin` - The administrator address
    /// * `liquidation_threshold` - Liquidation threshold in bps (must be > maintenance_margin)
    /// * `maintenance_margin` - Maintenance margin in bps (must be > 0)
    ///
    /// # Panics
    ///
    /// Panics if caller is not the admin or parameters are invalid
    pub fn set_risk_params(env: Env, admin: Address, liquidation_threshold: i128, maintenance_margin: i128) {
        require_admin(&env, &admin);
        if maintenance_margin <= 0 || maintenance_margin > 10000 {
            panic!("maintenance margin must be 1-10000 bps");
        }
        if liquidation_threshold <= maintenance_margin {
            panic!("liquidation threshold must be > maintenance margin");
        }
        if liquidation_threshold > 10000 {
            panic!("liquidation threshold must be <= 10000 bps");
        }
        put_config_value(&env, &DataKey::LiquidationThreshold, liquidation_threshold);
        put_config_value(&env, &DataKey::MaintenanceMargin, maintenance_margin);
    }

    /// Set maximum price deviation in basis points.
    ///
    /// # Arguments
    ///
    /// * `admin` - The administrator address
    /// * `deviation` - Max price deviation in bps (must be 1-5000)
    ///
    /// # Panics
    ///
    /// Panics if caller is not the admin or deviation is invalid
    pub fn set_max_price_deviation(env: Env, admin: Address, deviation: i128) {
        require_admin(&env, &admin);
        if deviation < 1 || deviation > 5000 {
            panic!("price deviation must be 1-5000 bps");
        }
        put_config_value(&env, &DataKey::MaxPriceDeviationBps, deviation);
    }

    /// Set time parameters.
    ///
    /// # Arguments
    ///
    /// * `admin` - The administrator address
    /// * `funding_interval` - Funding interval in seconds (must be >= 1)
    /// * `staleness_threshold` - Price staleness threshold in seconds (must be >= 1)
    ///
    /// # Panics
    ///
    /// Panics if caller is not the admin or parameters are invalid
    pub fn set_time_params(env: Env, admin: Address, funding_interval: u64, staleness_threshold: u64) {
        require_admin(&env, &admin);
        if funding_interval < 1 {
            panic!("funding interval must be >= 1");
        }
        if staleness_threshold < 1 {
            panic!("staleness threshold must be >= 1");
        }
        put_time_config_value(&env, &DataKey::FundingInterval, funding_interval);
        put_time_config_value(&env, &DataKey::PriceStalenessThreshold, staleness_threshold);
    }
}

#[cfg(test)]
mod test;

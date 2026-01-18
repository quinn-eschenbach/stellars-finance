#![no_std]

//! # Liquidity Pool Contract
//!
//! Manages liquidity provision for the Stellars Finance perpetual trading protocol.
//! This contract acts as the counterparty to all trader positions and handles PnL settlement.
//!
//! ## Key Features
//! - **LP Deposits/Withdrawals**: Users deposit tokens and receive LP shares proportionally.
//!   Withdrawals return tokens based on current share value (may differ from deposit due to PnL).
//! - **Position Collateral**: Tracks collateral deposited by traders for each position.
//! - **Liquidity Reservation**: Reserves liquidity when positions open, releases on close.
//! - **PnL Settlement**: Pays profitable traders from pool reserves.
//!
//! ## Share Calculation
//! - First deposit: shares = amount (1:1 ratio)
//! - Subsequent deposits: shares = (deposit * total_shares) / pool_value_before_deposit
//! This ensures existing LPs maintain their proportional ownership.
//!
//! ## Safety Mechanisms
//! - **Utilization Ratio**: Limits how much liquidity can be reserved for positions
//! - **Minimum Reserve Ratio**: Ensures minimum liquidity always available for withdrawals
//! - **Position Manager Authorization**: Only the authorized PositionManager can modify positions
//!
//! ## Usage
//! - LPs call `deposit()` and `withdraw()` directly
//! - PositionManager calls collateral and reservation functions when managing positions

use soroban_sdk::{contract, contractimpl, contracttype, log, token, Address, Env};

mod config_manager {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/config_manager.wasm");
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    ConfigManager,
    Token,
    TotalShares,
    TotalDeposits,
    Shares(Address),
    // Liquidity reservation for positions
    ReservedLiquidity,
    AuthorizedPositionManager,
    // Position collateral tracking
    PositionCollateral(u64),
}

#[contract]
pub struct LiquidityPool;

// Helper functions for storage access
fn get_config_manager(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::ConfigManager).unwrap()
}

fn put_config_manager(e: &Env, address: &Address) {
    e.storage().instance().set(&DataKey::ConfigManager, address);
}

fn get_token(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Token).unwrap()
}

fn put_token(e: &Env, token: Address) {
    e.storage().instance().set(&DataKey::Token, &token);
}

fn get_balance(e: &Env) -> i128 {
    let token = get_token(e);
    token::Client::new(e, &token).balance(&e.current_contract_address())
}

fn get_total_shares(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&DataKey::TotalShares)
        .unwrap_or(0)
}

fn put_total_shares(e: &Env, amount: i128) {
    e.storage().instance().set(&DataKey::TotalShares, &amount)
}

fn get_total_deposits(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&DataKey::TotalDeposits)
        .unwrap_or(0)
}

fn put_total_deposits(e: &Env, amount: i128) {
    e.storage().instance().set(&DataKey::TotalDeposits, &amount)
}

fn get_shares(e: &Env, user: &Address) -> i128 {
    e.storage()
        .persistent()
        .get(&DataKey::Shares(user.clone()))
        .unwrap_or(0)
}

fn put_shares(e: &Env, user: &Address, amount: i128) {
    e.storage()
        .persistent()
        .set(&DataKey::Shares(user.clone()), &amount);
}

fn mint_shares(e: &Env, to: &Address, amount: i128) {
    let current_shares = get_shares(e, to);
    let total = get_total_shares(e);
    put_shares(e, to, current_shares + amount);
    put_total_shares(e, total + amount);
}

fn burn_shares(e: &Env, from: &Address, amount: i128) {
    let current_shares = get_shares(e, from);
    if current_shares < amount {
        panic!("insufficient shares");
    }
    let total = get_total_shares(e);
    put_shares(e, from, current_shares - amount);
    put_total_shares(e, total - amount);
}

fn get_reserved_liquidity(e: &Env) -> u128 {
    e.storage()
        .instance()
        .get(&DataKey::ReservedLiquidity)
        .unwrap_or(0)
}

fn put_reserved_liquidity(e: &Env, amount: u128) {
    e.storage()
        .instance()
        .set(&DataKey::ReservedLiquidity, &amount);
}

fn get_authorized_position_manager(e: &Env) -> Option<Address> {
    e.storage()
        .instance()
        .get(&DataKey::AuthorizedPositionManager)
}

fn put_authorized_position_manager(e: &Env, address: &Address) {
    e.storage()
        .instance()
        .set(&DataKey::AuthorizedPositionManager, address);
}

fn require_position_manager(e: &Env, caller: &Address) {
    caller.require_auth();
    if let Some(authorized) = get_authorized_position_manager(e) {
        if caller != &authorized {
            panic!("unauthorized: not position manager");
        }
    } else {
        panic!("position manager not set");
    }
}

fn get_position_collateral(e: &Env, position_id: u64) -> u128 {
    e.storage()
        .persistent()
        .get(&DataKey::PositionCollateral(position_id))
        .unwrap_or(0)
}

fn put_position_collateral(e: &Env, position_id: u64, amount: u128) {
    e.storage()
        .persistent()
        .set(&DataKey::PositionCollateral(position_id), &amount);
}

fn delete_position_collateral(e: &Env, position_id: u64) {
    e.storage()
        .persistent()
        .remove(&DataKey::PositionCollateral(position_id));
}

#[contractimpl]
impl LiquidityPool {
    /// Initialize the liquidity pool with config manager and token addresses.
    ///
    /// # Arguments
    ///
    /// * `admin` - The administrator address (must authorize)
    /// * `config_manager` - The Config Manager contract address
    /// * `token` - The token contract address for this pool
    ///
    /// # Panics
    ///
    /// Panics if the pool is already initialized or admin doesn't authorize
    pub fn initialize(env: Env, admin: Address, config_manager: Address, token: Address) {
        if env.storage().instance().has(&DataKey::ConfigManager) {
            panic!("already initialized");
        }

        // Require admin to authorize initialization
        admin.require_auth();

        put_config_manager(&env, &config_manager);
        put_token(&env, token);
        put_total_shares(&env, 0);
        put_total_deposits(&env, 0);
    }

    /// Get the Config Manager address.
    ///
    /// # Returns
    ///
    /// The Config Manager contract address
    pub fn config_manager(env: Env) -> Address {
        get_config_manager(&env)
    }

    /// Get the token address for this pool.
    ///
    /// # Returns
    ///
    /// The token contract address
    pub fn token(env: Env) -> Address {
        get_token(&env)
    }

    /// Deposit tokens into the liquidity pool and receive LP shares.
    ///
    /// # Arguments
    ///
    /// * `user` - The address of the depositor
    /// * `amount` - The amount of tokens to deposit
    ///
    /// # Returns
    ///
    /// The number of LP shares minted to the user
    ///
    /// # Panics
    ///
    /// Panics if amount is not positive
    pub fn deposit(env: Env, user: Address, amount: i128) -> i128 {
        // Verify user authorization
        user.require_auth();

        // Validate amount is positive
        if amount <= 0 {
            panic!("amount must be positive");
        }

        // Get token and current pool state
        let token = get_token(&env);
        let total_shares = get_total_shares(&env);
        let total_deposits = get_total_deposits(&env);

        // Transfer tokens from user to contract first
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&user, &env.current_contract_address(), &amount);

        // Get actual balance after transfer (protects against PnL changes)
        let balance = get_balance(&env);

        // Calculate shares to mint using pro-rata formula to maintain fair LP ownership
        // First deposit: 1:1 ratio (no existing shares to dilute)
        // Subsequent deposits: new_shares = (deposit * total_shares) / pool_value_before
        // This ensures new depositors get shares proportional to their contribution
        // Example: If pool has 1000 tokens and 100 shares, depositing 100 tokens gets 10 shares
        // because 100 * 100 / 1000 = 10, maintaining 10% ownership for 10% contribution
        let shares_to_mint = if total_shares == 0 {
            amount
        } else {
            // pool_value_before = current balance minus the just-deposited amount
            let pool_value_before = balance - amount;
            if pool_value_before <= 0 {
                panic!("invalid pool state");
            }
            (amount * total_shares) / pool_value_before
        };

        // Prevent zero-share deposits that would lock user funds
        if shares_to_mint == 0 {
            panic!("deposit too small to mint shares");
        }

        // Mint shares to user
        mint_shares(&env, &user, shares_to_mint);

        // Update total deposits
        put_total_deposits(&env, total_deposits + amount);

        shares_to_mint
    }

    /// Withdraw tokens from the liquidity pool by burning LP shares.
    ///
    /// # Arguments
    ///
    /// * `user` - The address of the withdrawer
    /// * `shares` - The number of LP shares to burn
    ///
    /// # Returns
    ///
    /// The amount of tokens returned to the user
    ///
    /// # Panics
    ///
    /// Panics if shares is not positive, if total_shares is zero,
    /// or if withdrawal would violate liquidity constraints
    pub fn withdraw(env: Env, user: Address, shares: i128) -> i128 {
        // Verify user authorization
        user.require_auth();

        // Validate shares is positive
        if shares <= 0 {
            panic!("shares must be positive");
        }

        // Get token and current pool state
        let token = get_token(&env);
        let total_shares = get_total_shares(&env);
        let total_deposits = get_total_deposits(&env);

        // Prevent division by zero
        if total_shares == 0 {
            panic!("no shares to burn");
        }

        // Get actual balance (reflects PnL from trading)
        let balance = get_balance(&env);

        // Calculate tokens to return based on actual pool value
        // tokens = (shares * balance) / total_shares
        let tokens_to_return = (shares * balance) / total_shares;

        // Check available liquidity
        let reserved = get_reserved_liquidity(&env) as i128;
        let available = balance - reserved;

        if tokens_to_return > available {
            panic!("insufficient available liquidity");
        }

        // Enforce minimum reserve ratio to ensure pool solvency
        // This protects LPs by ensuring the pool always has enough unreserved liquidity
        // to handle potential position closures and payouts
        let config_manager = get_config_manager(&env);
        let config_client = crate::config_manager::Client::new(&env, &config_manager);
        let min_reserve_ratio = config_client.min_liquidity_reserve_ratio();

        // Calculate how much unreserved liquidity must remain after withdrawal
        // Example: If min_reserve_ratio = 2000 (20%) and balance_after = 1000,
        // then min_reserve_required = 200, and (balance - reserved) must be >= 200
        let balance_after_withdrawal = balance - tokens_to_return;
        let min_reserve_required = (balance_after_withdrawal * min_reserve_ratio) / 10000;

        if (balance_after_withdrawal - reserved) < min_reserve_required {
            panic!("withdrawal would violate minimum reserve ratio");
        }

        // Burn shares from user (includes validation)
        burn_shares(&env, &user, shares);

        // Update total deposits proportionally
        let deposits_to_reduce = (shares * total_deposits) / total_shares;
        put_total_deposits(&env, total_deposits - deposits_to_reduce);

        // Transfer tokens from contract to user
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &user, &tokens_to_return);

        tokens_to_return
    }

    /// Get the LP share balance for a user.
    ///
    /// # Arguments
    ///
    /// * `user` - The address to query
    ///
    /// # Returns
    ///
    /// The number of LP shares owned by the user
    pub fn get_shares(env: Env, user: Address) -> i128 {
        get_shares(&env, &user)
    }

    /// Get the total number of LP shares in circulation.
    ///
    /// # Returns
    ///
    /// The total number of LP shares
    pub fn get_total_shares(env: Env) -> i128 {
        get_total_shares(&env)
    }

    /// Get the total amount of tokens deposited in the pool.
    ///
    /// # Returns
    ///
    /// The total deposited token amount
    pub fn get_total_deposits(env: Env) -> i128 {
        get_total_deposits(&env)
    }

    /// Set the authorized position manager that can reserve/release liquidity.
    ///
    /// # Arguments
    ///
    /// * `admin` - The admin address (must match ConfigManager admin)
    /// * `position_manager` - The Position Manager contract address
    ///
    /// # Panics
    ///
    /// Panics if caller is not authorized
    pub fn set_position_manager(env: Env, admin: Address, position_manager: Address) {
        admin.require_auth();

        // Verify caller is the admin from ConfigManager
        let config_manager = get_config_manager(&env);
        let config_client = crate::config_manager::Client::new(&env, &config_manager);
        let config_admin = config_client.admin();

        if admin != config_admin {
            panic!("unauthorized: not admin");
        }

        put_authorized_position_manager(&env, &position_manager);
    }

    /// Reserve liquidity when a position is opened.
    ///
    /// # Arguments
    ///
    /// * `position_manager` - The Position Manager contract address
    /// * `position_id` - The position ID
    /// * `size` - The position size (notional value) to reserve
    /// * `collateral` - The collateral amount deposited
    ///
    /// # Panics
    ///
    /// Panics if caller is not the authorized position manager
    pub fn reserve_liquidity(
        env: Env,
        position_manager: Address,
        position_id: u64,
        size: u128,
        collateral: u128,
    ) {
        require_position_manager(&env, &position_manager);

        let reserved = get_reserved_liquidity(&env);
        let new_reserved = reserved + size;

        put_reserved_liquidity(&env, new_reserved);
        put_position_collateral(&env, position_id, collateral);
    }

    /// Release liquidity when a position is closed.
    ///
    /// # Arguments
    ///
    /// * `position_manager` - The Position Manager contract address
    /// * `position_id` - The position ID
    /// * `size` - The position size (notional value) to release
    ///
    /// # Panics
    ///
    /// Panics if caller is not the authorized position manager
    pub fn release_liquidity(env: Env, position_manager: Address, position_id: u64, size: u128) {
        require_position_manager(&env, &position_manager);

        let reserved = get_reserved_liquidity(&env);
        if size > reserved {
            panic!("cannot release more than reserved");
        }

        let new_reserved = reserved - size;
        put_reserved_liquidity(&env, new_reserved);
    }

    /// Get the total reserved liquidity.
    ///
    /// # Returns
    ///
    /// The total liquidity reserved for open positions
    pub fn get_reserved_liquidity(env: Env) -> u128 {
        get_reserved_liquidity(&env)
    }

    /// Get the available liquidity (total balance - reserved).
    ///
    /// # Returns
    ///
    /// The liquidity available for withdrawal or new positions
    pub fn get_available_liquidity(env: Env) -> i128 {
        let balance = get_balance(&env);
        let reserved = get_reserved_liquidity(&env) as i128;
        balance - reserved
    }

    /// Get the pool utilization ratio in basis points.
    ///
    /// # Returns
    ///
    /// The utilization ratio in basis points (e.g., 8000 = 80%)
    pub fn get_utilization_ratio(env: Env) -> u32 {
        let balance = get_balance(&env);
        if balance == 0 {
            return 0;
        }

        let reserved = get_reserved_liquidity(&env) as i128;
        let utilization = (reserved * 10000) / balance;

        utilization as u32
    }

    /// Get the collateral deposited for a specific position.
    ///
    /// # Arguments
    ///
    /// * `position_id` - The position ID
    ///
    /// # Returns
    ///
    /// The collateral amount for the position
    pub fn get_position_collateral(env: Env, position_id: u64) -> u128 {
        get_position_collateral(&env, position_id)
    }

    /// Deposit collateral for a position.
    ///
    /// # Arguments
    ///
    /// * `position_manager` - The Position Manager contract address
    /// * `position_id` - The position ID
    /// * `trader` - The trader's address
    /// * `amount` - The collateral amount to deposit
    ///
    /// # Panics
    ///
    /// Panics if caller is not the authorized position manager
    pub fn deposit_position_collateral(
        env: Env,
        position_manager: Address,
        position_id: u64,
        trader: Address,
        amount: u128,
    ) {
        require_position_manager(&env, &position_manager);

        // Transfer collateral from trader to pool
        let token = get_token(&env);
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&trader, &env.current_contract_address(), &(amount as i128));

        // Track collateral for this position
        let current = get_position_collateral(&env, position_id);
        put_position_collateral(&env, position_id, current + amount);
    }

    /// Record position collateral that was already transferred to the pool.
    /// Used by limit orders where collateral is escrowed in position manager
    /// and then transferred directly to pool.
    ///
    /// # Arguments
    ///
    /// * `position_manager` - The Position Manager contract address
    /// * `position_id` - The position ID
    /// * `amount` - The collateral amount to record
    ///
    /// # Panics
    ///
    /// Panics if caller is not the authorized position manager
    pub fn record_position_collateral(
        env: Env,
        position_manager: Address,
        position_id: u64,
        amount: u128,
    ) {
        require_position_manager(&env, &position_manager);

        // Just track collateral - assumes tokens already transferred
        let current = get_position_collateral(&env, position_id);
        put_position_collateral(&env, position_id, current + amount);
    }

    /// Withdraw collateral for a position (when closing).
    ///
    /// # Arguments
    ///
    /// * `position_manager` - The Position Manager contract address
    /// * `position_id` - The position ID
    /// * `trader` - The trader's address
    /// * `amount` - The collateral amount to withdraw
    ///
    /// # Panics
    ///
    /// Panics if caller is not the authorized position manager
    pub fn withdraw_position_collateral(
        env: Env,
        position_manager: Address,
        position_id: u64,
        trader: Address,
        amount: u128,
    ) {
        require_position_manager(&env, &position_manager);

        let current = get_position_collateral(&env, position_id);
        if amount > current {
            panic!("insufficient position collateral");
        }

        // Update or delete collateral tracking
        let remaining = current - amount;
        if remaining == 0 {
            delete_position_collateral(&env, position_id);
        } else {
            put_position_collateral(&env, position_id, remaining);
        }

        // Transfer collateral from pool to trader
        let token = get_token(&env);
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &trader, &(amount as i128));
    }

    /// Settle trader PnL by transferring profit from pool reserves.
    ///
    /// # Arguments
    ///
    /// * `position_manager` - The Position Manager contract address
    /// * `trader` - The trader's address
    /// * `pnl` - The PnL amount (positive = profit to pay trader)
    ///
    /// # Panics
    ///
    /// Panics if caller is not the authorized position manager
    pub fn settle_trader_pnl(env: Env, position_manager: Address, trader: Address, pnl: i128) {
        require_position_manager(&env, &position_manager);

        // Only pay out positive PnL (losses handled by reduced collateral withdrawal)
        if pnl <= 0 {
            return;
        }

        // Transfer profit from pool to trader
        let token = get_token(&env);
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &trader, &pnl);
    }
}

#[cfg(test)]
mod test;

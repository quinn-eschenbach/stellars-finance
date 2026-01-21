//! Reflector Oracle Interface
//!
//! SEP-40 compliant interface for the Reflector oracle on Stellar/Soroban.
//! See: https://reflector.network/docs/interface

use soroban_sdk::{contracttype, Address, Env, Symbol, Vec};

/// Oracle contract interface exported as ReflectorClient
#[soroban_sdk::contractclient(name = "ReflectorClient")]
pub trait Contract {
    /// Base oracle symbol the price is reported in (e.g., USD)
    fn base(e: Env) -> Asset;

    /// All assets quoted by the contract
    fn assets(e: Env) -> Vec<Asset>;

    /// Number of decimal places used to represent price for all assets quoted by the oracle
    fn decimals(e: Env) -> u32;

    /// Quotes asset price in base asset at specific timestamp
    fn price(e: Env, asset: Asset, timestamp: u64) -> Option<PriceData>;

    /// Quotes the most recent price for an asset
    fn lastprice(e: Env, asset: Asset) -> Option<PriceData>;

    /// Quotes last N price records for the given asset
    fn prices(e: Env, asset: Asset, records: u32) -> Option<Vec<PriceData>>;

    /// Quotes the most recent cross price record for the pair of assets
    fn x_last_price(e: Env, base_asset: Asset, quote_asset: Asset) -> Option<PriceData>;

    /// Quotes the cross price for the pair of assets at specific timestamp
    fn x_price(e: Env, base_asset: Asset, quote_asset: Asset, timestamp: u64) -> Option<PriceData>;

    /// Quotes last N cross price records for the pair of assets
    fn x_prices(
        e: Env,
        base_asset: Asset,
        quote_asset: Asset,
        records: u32,
    ) -> Option<Vec<PriceData>>;

    /// Quotes the time-weighted average price for the given asset over N recent records
    fn twap(e: Env, asset: Asset, records: u32) -> Option<i128>;

    /// Quotes the time-weighted average cross price for the given asset pair over N recent records
    fn x_twap(e: Env, base_asset: Asset, quote_asset: Asset, records: u32) -> Option<i128>;

    /// Price feed resolution (default tick period timeframe, in seconds - 5 minutes by default)
    fn resolution(e: Env) -> u32;

    /// Historical records retention period, in seconds (24 hours by default)
    fn period(e: Env) -> Option<u64>;

    /// The most recent price update timestamp
    fn last_timestamp(e: Env) -> u64;

    /// Contract protocol version
    fn version(e: Env) -> u32;

    /// Contract admin address
    fn admin(e: Env) -> Option<Address>;
}

/// Quoted asset definition
#[contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Asset {
    /// For Stellar Classic and Soroban assets (use token contract address)
    Stellar(Address),
    /// For any external currencies/tokens/assets/symbols (e.g., "BTC", "ETH", "XLM")
    Other(Symbol),
}

/// Price record definition
#[contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct PriceData {
    /// Asset price at given point in time
    pub price: i128,
    /// Record timestamp
    pub timestamp: u64,
}

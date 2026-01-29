#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Price(u32),
    PriceTimestamp(u32),
    Pusher(Address),
}

fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("not initialized")
}

fn require_admin(env: &Env, admin: &Address) {
    admin.require_auth();
    let stored_admin = get_admin(env);
    if admin != &stored_admin {
        panic!("unauthorized");
    }
}

fn require_pusher(env: &Env, pusher: &Address) {
    pusher.require_auth();
    let is_authorized: bool = env
        .storage()
        .instance()
        .get(&DataKey::Pusher(pusher.clone()))
        .unwrap_or(false);
    if !is_authorized {
        panic!("not an authorized pusher");
    }
}

#[contract]
pub struct CustomOracle;

#[contractimpl]
impl CustomOracle {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn admin(env: Env) -> Address {
        get_admin(&env)
    }

    pub fn add_pusher(env: Env, admin: Address, pusher: Address) {
        require_admin(&env, &admin);
        env.storage()
            .instance()
            .set(&DataKey::Pusher(pusher), &true);
    }

    pub fn remove_pusher(env: Env, admin: Address, pusher: Address) {
        require_admin(&env, &admin);
        env.storage().instance().remove(&DataKey::Pusher(pusher));
    }

    pub fn is_pusher(env: Env, pusher: Address) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Pusher(pusher))
            .unwrap_or(false)
    }

    pub fn push_price(env: Env, pusher: Address, market_id: u32, price: i128) {
        require_pusher(&env, &pusher);
        if price <= 0 {
            panic!("price must be positive");
        }
        let timestamp = env.ledger().timestamp();
        env.storage()
            .instance()
            .set(&DataKey::Price(market_id), &price);
        env.storage()
            .instance()
            .set(&DataKey::PriceTimestamp(market_id), &timestamp);
    }

    pub fn push_prices(env: Env, pusher: Address, prices: Map<u32, i128>) {
        require_pusher(&env, &pusher);
        let timestamp = env.ledger().timestamp();
        for (market_id, price) in prices.iter() {
            if price <= 0 {
                panic!("price must be positive");
            }
            env.storage()
                .instance()
                .set(&DataKey::Price(market_id), &price);
            env.storage()
                .instance()
                .set(&DataKey::PriceTimestamp(market_id), &timestamp);
        }
    }

    pub fn get_price(env: Env, market_id: u32) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::Price(market_id))
            .expect("no price for market")
    }

    pub fn get_price_with_timestamp(env: Env, market_id: u32) -> (i128, u64) {
        let price: i128 = env
            .storage()
            .instance()
            .get(&DataKey::Price(market_id))
            .expect("no price for market");
        let timestamp: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PriceTimestamp(market_id))
            .expect("no timestamp for market");
        (price, timestamp)
    }
}

#[cfg(test)]
mod test;

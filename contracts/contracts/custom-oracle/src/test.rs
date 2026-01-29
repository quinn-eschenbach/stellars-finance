use soroban_sdk::{testutils::Address as _, Address, Env, Map};

use crate::{CustomOracle, CustomOracleClient};

fn setup_test() -> (Env, CustomOracleClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(CustomOracle, ());
    let client = CustomOracleClient::new(&env, &contract_id);
    client.initialize(&admin);
    (env, client, admin)
}

#[test]
fn test_initialize() {
    let (_env, client, admin) = setup_test();
    assert_eq!(client.admin(), admin);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_double_initialize() {
    let (_env, client, admin) = setup_test();
    client.initialize(&admin);
}

#[test]
fn test_add_and_remove_pusher() {
    let (env, client, admin) = setup_test();
    let pusher = Address::generate(&env);

    assert!(!client.is_pusher(&pusher));
    client.add_pusher(&admin, &pusher);
    assert!(client.is_pusher(&pusher));
    client.remove_pusher(&admin, &pusher);
    assert!(!client.is_pusher(&pusher));
}

#[test]
fn test_push_and_get_price() {
    let (env, client, admin) = setup_test();
    let pusher = Address::generate(&env);
    client.add_pusher(&admin, &pusher);

    let price: i128 = 10_000_000; // $1.00
    client.push_price(&pusher, &0, &price);
    assert_eq!(client.get_price(&0), price);
}

#[test]
fn test_push_prices_batch() {
    let (env, client, admin) = setup_test();
    let pusher = Address::generate(&env);
    client.add_pusher(&admin, &pusher);

    let mut prices = Map::new(&env);
    prices.set(0, 10_000_000i128);         // XLM: $1.00
    prices.set(1, 500_000_000_000i128);    // BTC: $50,000
    prices.set(2, 30_000_000_000i128);     // ETH: $3,000

    client.push_prices(&pusher, &prices);

    assert_eq!(client.get_price(&0), 10_000_000);
    assert_eq!(client.get_price(&1), 500_000_000_000);
    assert_eq!(client.get_price(&2), 30_000_000_000);
}

#[test]
fn test_get_price_with_timestamp() {
    let (env, client, admin) = setup_test();
    let pusher = Address::generate(&env);
    client.add_pusher(&admin, &pusher);

    client.push_price(&pusher, &0, &10_000_000);
    let (price, _timestamp) = client.get_price_with_timestamp(&0);
    assert_eq!(price, 10_000_000);
}

#[test]
#[should_panic(expected = "not an authorized pusher")]
fn test_unauthorized_pusher() {
    let (env, client, _admin) = setup_test();
    let unauthorized = Address::generate(&env);
    client.push_price(&unauthorized, &0, &10_000_000);
}

#[test]
#[should_panic(expected = "price must be positive")]
fn test_push_zero_price() {
    let (env, client, admin) = setup_test();
    let pusher = Address::generate(&env);
    client.add_pusher(&admin, &pusher);
    client.push_price(&pusher, &0, &0);
}

#[test]
#[should_panic(expected = "no price for market")]
fn test_get_price_unset_market() {
    let (_env, client, _admin) = setup_test();
    client.get_price(&99);
}

#[test]
fn test_price_update_overwrites() {
    let (env, client, admin) = setup_test();
    let pusher = Address::generate(&env);
    client.add_pusher(&admin, &pusher);

    client.push_price(&pusher, &0, &10_000_000);
    assert_eq!(client.get_price(&0), 10_000_000);

    client.push_price(&pusher, &0, &20_000_000);
    assert_eq!(client.get_price(&0), 20_000_000);
}

#[test]
fn test_multiple_pushers() {
    let (env, client, admin) = setup_test();
    let pusher1 = Address::generate(&env);
    let pusher2 = Address::generate(&env);
    client.add_pusher(&admin, &pusher1);
    client.add_pusher(&admin, &pusher2);

    client.push_price(&pusher1, &0, &10_000_000);
    assert_eq!(client.get_price(&0), 10_000_000);

    client.push_price(&pusher2, &0, &20_000_000);
    assert_eq!(client.get_price(&0), 20_000_000);
}

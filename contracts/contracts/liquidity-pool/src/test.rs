#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, token, Address, Env};

fn create_token_contract<'a>(
    env: &Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let contract_address = env.register_stellar_asset_contract_v2(admin.clone());
    (
        token::Client::new(env, &contract_address.address()),
        token::StellarAssetClient::new(env, &contract_address.address()),
    )
}

mod config_manager {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32v1-none/release/config_manager.wasm"
    );
}

fn create_mock_config_manager(env: &Env, admin: &Address) -> Address {
    // Deploy actual ConfigManager contract for tests
    let contract_id = env.register(config_manager::WASM, ());
    let client = config_manager::Client::new(env, &contract_id);

    // Initialize with admin (admin must authorize)
    client.initialize(admin);

    // Set minimum liquidity reserve ratio (e.g., 10% = 1000 bps)
    client.set_min_liquidity_reserve_ratio(admin, &1000);

    contract_id
}

#[test]
fn test_deposit_and_withdraw_happy_path() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);

    // Create token contract
    let (token_client, token_admin) = create_token_contract(&env, &admin);

    // Mint tokens to user1
    token_admin.mint(&user1, &1000);

    // Deploy config manager (mock for unit tests)
    let config_manager_id = create_mock_config_manager(&env, &admin);

    // Deploy liquidity pool contract
    let contract_id = env.register(LiquidityPool, ());
    let client = LiquidityPoolClient::new(&env, &contract_id);

    // Initialize the pool with admin, config manager and token
    client.initialize(&admin, &config_manager_id, &token_client.address);

    // Test deposit
    let shares = client.deposit(&user1, &500);

    // First deposit should be 1:1 ratio
    assert_eq!(shares, 500);
    assert_eq!(client.get_shares(&user1), 500);
    assert_eq!(client.get_total_shares(), 500);
    assert_eq!(client.get_total_deposits(), 500);

    // Verify token balance
    assert_eq!(token_client.balance(&user1), 500);
    assert_eq!(token_client.balance(&contract_id), 500);

    // Test withdraw
    let tokens_returned = client.withdraw(&user1, &250);

    // Should get back half the tokens
    assert_eq!(tokens_returned, 250);
    assert_eq!(client.get_shares(&user1), 250);
    assert_eq!(client.get_total_shares(), 250);
    assert_eq!(client.get_total_deposits(), 250);

    // Verify final token balance
    assert_eq!(token_client.balance(&user1), 750);
    assert_eq!(token_client.balance(&contract_id), 250);
}

#[test]
fn test_multiple_deposits() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    // Create token contract
    let (token_client, token_admin) = create_token_contract(&env, &admin);

    // Mint tokens to users
    token_admin.mint(&user1, &1000);
    token_admin.mint(&user2, &1000);

    // Deploy config manager (mock for unit tests)
    let config_manager_id = create_mock_config_manager(&env, &admin);

    // Deploy liquidity pool contract
    let contract_id = env.register(LiquidityPool, ());
    let client = LiquidityPoolClient::new(&env, &contract_id);

    // Initialize the pool with admin, config manager and token
    client.initialize(&admin, &config_manager_id, &token_client.address);

    // User1 deposits 500 tokens
    let shares1 = client.deposit(&user1, &500);
    assert_eq!(shares1, 500); // 1:1 ratio for first deposit

    // User2 deposits 500 tokens
    let shares2 = client.deposit(&user2, &500);
    assert_eq!(shares2, 500); // Should also get 500 shares (same ratio)

    // Verify totals
    assert_eq!(client.get_total_shares(), 1000);
    assert_eq!(client.get_total_deposits(), 1000);
    assert_eq!(client.get_shares(&user1), 500);
    assert_eq!(client.get_shares(&user2), 500);
}

#[test]
fn test_varying_deposit_sizes() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);

    // Create token contract
    let (token_client, token_admin) = create_token_contract(&env, &admin);

    // Mint tokens to users
    token_admin.mint(&user1, &10000);
    token_admin.mint(&user2, &10000);
    token_admin.mint(&user3, &10000);

    // Deploy config manager (mock for unit tests)
    let config_manager_id = create_mock_config_manager(&env, &admin);

    // Deploy liquidity pool contract
    let contract_id = env.register(LiquidityPool, ());
    let client = LiquidityPoolClient::new(&env, &contract_id);

    // Initialize the pool with admin, config manager and token
    client.initialize(&admin, &config_manager_id, &token_client.address);

    // User1 deposits 1000 tokens (first deposit, 1:1 ratio)
    let shares1 = client.deposit(&user1, &1000);
    assert_eq!(shares1, 1000);
    assert_eq!(client.get_total_shares(), 1000);
    assert_eq!(client.get_total_deposits(), 1000);

    // User2 deposits 3000 tokens (should get 3x the shares)
    let shares2 = client.deposit(&user2, &3000);
    assert_eq!(shares2, 3000);
    assert_eq!(client.get_total_shares(), 4000);
    assert_eq!(client.get_total_deposits(), 4000);

    // User3 deposits 500 tokens (should get proportional shares)
    let shares3 = client.deposit(&user3, &500);
    assert_eq!(shares3, 500);
    assert_eq!(client.get_total_shares(), 4500);
    assert_eq!(client.get_total_deposits(), 4500);

    // Verify individual balances
    assert_eq!(client.get_shares(&user1), 1000);
    assert_eq!(client.get_shares(&user2), 3000);
    assert_eq!(client.get_shares(&user3), 500);

    // Test partial withdrawal from user2
    let tokens_returned = client.withdraw(&user2, &1500);
    assert_eq!(tokens_returned, 1500); // Should get back 1500 tokens
    assert_eq!(client.get_shares(&user2), 1500);
    assert_eq!(client.get_total_shares(), 3000);

    // Verify proportions are maintained
    let balance = token_client.balance(&contract_id);
    assert_eq!(balance, 3000); // 4500 - 1500 withdrawn

    // User1 withdraws all shares
    let tokens_returned1 = client.withdraw(&user1, &1000);
    assert_eq!(tokens_returned1, 1000);
    assert_eq!(client.get_shares(&user1), 0);
}

#[test]
fn test_extreme_values() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    // Create token contract
    let (token_client, token_admin) = create_token_contract(&env, &admin);

    // Mint large amounts
    token_admin.mint(&user1, &1_000_000_000_000);
    token_admin.mint(&user2, &1_000_000_000_000);

    // Deploy config manager (mock for unit tests)
    let config_manager_id = create_mock_config_manager(&env, &admin);

    // Deploy liquidity pool contract
    let contract_id = env.register(LiquidityPool, ());
    let client = LiquidityPoolClient::new(&env, &contract_id);

    // Initialize the pool with admin, config manager and token
    client.initialize(&admin, &config_manager_id, &token_client.address);

    // Test with very large initial deposit
    let large_deposit = 1_000_000_000;
    let shares1 = client.deposit(&user1, &large_deposit);
    assert_eq!(shares1, large_deposit);

    // Test with small deposit that still yields shares (1:1 ratio here)
    let small_deposit = 1;
    let shares2 = client.deposit(&user2, &small_deposit);
    assert_eq!(shares2, 1);

    // Test withdrawal of small shares
    let withdrawn = client.withdraw(&user2, &1);
    assert_eq!(withdrawn, 1);
}

#[test]
#[should_panic(expected = "deposit too small to mint shares")]
fn test_zero_shares_deposit_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    // Create token contract
    let (token_client, token_admin) = create_token_contract(&env, &admin);

    // Mint tokens
    token_admin.mint(&user1, &1_000_000_000_000);
    token_admin.mint(&user2, &1_000_000_000_000);

    // Deploy config manager
    let config_manager_id = create_mock_config_manager(&env, &admin);

    // Deploy liquidity pool contract
    let contract_id = env.register(LiquidityPool, ());
    let client = LiquidityPoolClient::new(&env, &contract_id);

    // Initialize the pool
    client.initialize(&admin, &config_manager_id, &token_client.address);

    // User1 deposits a large amount first
    let large_deposit = 1_000_000_000_000;
    client.deposit(&user1, &large_deposit);

    // User2 tries to deposit an amount that would yield 0 shares
    // shares = (amount * total_shares) / pool_value = (1 * 1_000_000_000_000) / 1_000_000_000_000 = 1
    // So we need a deposit where (amount * total_shares) < pool_value
    // With pool_value = 1_000_000_000_000 and total_shares = 1_000_000_000_000
    // We need amount < 1, but minimum is 1, so this case won't trigger with equal values

    // To trigger zero shares: pool_value_before must be > amount * total_shares
    // After large deposit: pool = 1_000_000_000_000, shares = 1_000_000_000_000
    // Depositing 1 token: shares_to_mint = (1 * 1_000_000_000_000) / 1_000_000_000_000 = 1
    // This won't trigger the panic, so we need a different setup.

    // Let's artificially create the scenario by having pool gain value (simulating LP profit)
    // We can do this by directly transferring tokens to the pool without deposit
    token_admin.mint(&contract_id, &1_000_000_000_000); // Double the pool balance

    // Now: pool_balance = 2_000_000_000_000, total_shares = 1_000_000_000_000
    // Depositing 1 token: shares = (1 * 1_000_000_000_000) / 2_000_000_000_000 = 0
    // This should panic with "deposit too small to mint shares"
    client.deposit(&user2, &1);
}

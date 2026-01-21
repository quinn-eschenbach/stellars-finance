#!/usr/bin/env tsx

/**
 * Initialize all Stellars Finance contracts after deployment
 * Must be run after deploy.ts
 *
 * Initialization order:
 * 1. FaucetToken - Initialize test token with name, symbol, decimals
 * 2. ConfigManager - Set admin address
 * 3. OracleIntegrator - Initialize with config manager
 * 4. LiquidityPool - Initialize with config manager and token
 * 5. MarketManager - Initialize with config manager
 * 6. PositionManager - Initialize with config manager
 *
 * Usage:
 *   npm run initialize:testnet
 *   npm run initialize:mainnet
 */

import 'dotenv/config';
import { readFileSync } from 'fs';
import { NETWORK_CONFIGS } from './config';
import { Client as FaucetTokenClient } from '@stellars-finance/faucet-token';
import { Client as ConfigManagerClient } from '@stellars-finance/config-manager';
import { Client as OracleIntegratorClient } from '@stellars-finance/oracle-integrator';
import { Client as LiquidityPoolClient } from '@stellars-finance/liquidity-pool';
import { Client as PositionManagerClient } from '@stellars-finance/position-manager';
import { Client as MarketManagerClient } from '@stellars-finance/market-manager';
import { Keypair, TransactionBuilder } from '@stellar/stellar-sdk';

type NetworkType = 'testnet' | 'mainnet';

// Parse command line arguments
const args = process.argv.slice(2);
const networkArg = args.find(arg => arg.startsWith('--network='));
const network: NetworkType = (networkArg?.split('=')[1] as NetworkType) || 'testnet';

if (!['testnet', 'mainnet'].includes(network)) {
  console.error('Error: Network must be either "testnet" or "mainnet"');
  process.exit(1);
}

console.log(`\n⚙️  Initializing Stellars Finance contracts on ${network.toUpperCase()}\n`);

// Load deployment data
const deploymentFile = `deployments/${network}.json`;
let deployment: any;

try {
  const data = readFileSync(deploymentFile, 'utf-8');
  deployment = JSON.parse(data);
} catch (error) {
  console.error(`Error: Could not load deployment file ${deploymentFile}`);
  console.error('Please run deployment first: npm run deploy:' + network);
  process.exit(1);
}

const contracts = deployment.contracts;
const networkConfig = NETWORK_CONFIGS[network];

// Load source keypair from environment or use default
const sourceSecretKey = process.env.STELLAR_SECRET_KEY;
if (!sourceSecretKey) {
  console.error('Error: STELLAR_SECRET_KEY environment variable not set');
  console.error('Please set it to your deployer account secret key');
  process.exit(1);
}

const sourceKeypair = Keypair.fromSecret(sourceSecretKey);
const publicKey = sourceKeypair.publicKey();

console.log(`Using source account: ${publicKey}\n`);

// Common client options
const clientOptions = {
  publicKey,
  networkPassphrase: networkConfig.networkPassphrase,
  rpcUrl: networkConfig.rpcUrl,
};

async function main() {
  try {
    // 1. Initialize FaucetToken
    console.log('1️⃣  Initializing FaucetToken...');
    const faucetTokenClient = new FaucetTokenClient({
      ...clientOptions,
      contractId: contracts['faucet-token'],
    });

    const faucetInitTx = await faucetTokenClient.initialize({
      name: 'Test USDC',
      symbol: 'TUSDC',
      decimals: 7,
    });

    await faucetInitTx.signAndSend({
      signTransaction: async (xdr: string) => {
        const tx = TransactionBuilder.fromXDR(xdr, networkConfig.networkPassphrase);
        tx.sign(sourceKeypair);
        return { signedTxXdr: tx.toXDR() };
      }
    });
    console.log('   ✓ FaucetToken initialized (Test USDC, TUSDC, 7 decimals)\n');

    // 2. Initialize ConfigManager
    console.log('2️⃣  Initializing ConfigManager...');
    const configManagerClient = new ConfigManagerClient({
      ...clientOptions,
      contractId: contracts['config-manager'],
    });

    const configInitTx = await configManagerClient.initialize({
      admin: publicKey,
    });

    await configInitTx.signAndSend({
      signTransaction: async (xdr: string) => {
        const tx = TransactionBuilder.fromXDR(xdr, networkConfig.networkPassphrase);
        tx.sign(sourceKeypair);
        return { signedTxXdr: tx.toXDR() };
      }
    });
    console.log(`   ✓ ConfigManager initialized with admin: ${publicKey}`);

    // 2b. Set token address in ConfigManager
    console.log('   → Setting token address in ConfigManager...');
    const setTokenTx = await configManagerClient.set_token({
      admin: publicKey,
      contract: contracts['faucet-token'],
    });

    await setTokenTx.signAndSend({
      signTransaction: async (xdr: string) => {
        const tx = TransactionBuilder.fromXDR(xdr, networkConfig.networkPassphrase);
        tx.sign(sourceKeypair);
        return { signedTxXdr: tx.toXDR() };
      }
    });
    console.log('   ✓ Token address set');

    // 2c. Set oracle integrator address in ConfigManager
    console.log('   → Setting oracle integrator address in ConfigManager...');
    const setOracleTx = await configManagerClient.set_oracle_integrator({
      admin: publicKey,
      contract: contracts['oracle-integrator'],
    });

    await setOracleTx.signAndSend({
      signTransaction: async (xdr: string) => {
        const tx = TransactionBuilder.fromXDR(xdr, networkConfig.networkPassphrase);
        tx.sign(sourceKeypair);
        return { signedTxXdr: tx.toXDR() };
      }
    });
    console.log('   ✓ Oracle integrator address set');

    // 2d. Set liquidity pool address in ConfigManager
    console.log('   → Setting liquidity pool address in ConfigManager...');
    const setLiquidityPoolTx = await configManagerClient.set_liquidity_pool({
      admin: publicKey,
      contract: contracts['liquidity-pool'],
    });

    await setLiquidityPoolTx.signAndSend({
      signTransaction: async (xdr: string) => {
        const tx = TransactionBuilder.fromXDR(xdr, networkConfig.networkPassphrase);
        tx.sign(sourceKeypair);
        return { signedTxXdr: tx.toXDR() };
      }
    });
    console.log('   ✓ Liquidity pool address set');

    // 2e. Set position manager address in ConfigManager
    console.log('   → Setting position manager address in ConfigManager...');
    const setPositionManagerTx = await configManagerClient.set_position_manager({
      admin: publicKey,
      contract: contracts['position-manager'],
    });

    await setPositionManagerTx.signAndSend({
      signTransaction: async (xdr: string) => {
        const tx = TransactionBuilder.fromXDR(xdr, networkConfig.networkPassphrase);
        tx.sign(sourceKeypair);
        return { signedTxXdr: tx.toXDR() };
      }
    });
    console.log('   ✓ Position manager address set');

    // 2f. Set market manager address in ConfigManager
    console.log('   → Setting market manager address in ConfigManager...');
    const setMarketManagerTx = await configManagerClient.set_market_manager({
      admin: publicKey,
      contract: contracts['market-manager'],
    });

    await setMarketManagerTx.signAndSend({
      signTransaction: async (xdr: string) => {
        const tx = TransactionBuilder.fromXDR(xdr, networkConfig.networkPassphrase);
        tx.sign(sourceKeypair);
        return { signedTxXdr: tx.toXDR() };
      }
    });
    console.log('   ✓ Market manager address set\n');

    // 3. Initialize OracleIntegrator
    console.log('3️⃣  Initializing OracleIntegrator...');
    const oracleIntegratorClient = new OracleIntegratorClient({
      ...clientOptions,
      contractId: contracts['oracle-integrator'],
    });

    const oracleInitTx = await oracleIntegratorClient.initialize({
      config_manager: contracts['config-manager'],
    });

    await oracleInitTx.signAndSend({
      signTransaction: async (xdr: string) => {
        const tx = TransactionBuilder.fromXDR(xdr, networkConfig.networkPassphrase);
        tx.sign(sourceKeypair);
        return { signedTxXdr: tx.toXDR() };
      }
    });
    console.log('   ✓ OracleIntegrator initialized\n');

    // 4. Initialize LiquidityPool
    console.log('4️⃣  Initializing LiquidityPool...');
    const liquidityPoolClient = new LiquidityPoolClient({
      ...clientOptions,
      contractId: contracts['liquidity-pool'],
    });

    const liquidityInitTx = await liquidityPoolClient.initialize({
      admin: publicKey,
      config_manager: contracts['config-manager'],
      token: contracts['faucet-token'],
    });

    await liquidityInitTx.signAndSend({
      signTransaction: async (xdr: string) => {
        const tx = TransactionBuilder.fromXDR(xdr, networkConfig.networkPassphrase);
        tx.sign(sourceKeypair);
        return { signedTxXdr: tx.toXDR() };
      }
    });
    console.log('   ✓ LiquidityPool initialized');

    // 4b. Authorize PositionManager to manage liquidity
    console.log('   → Authorizing PositionManager to manage liquidity...');
    const setPositionManagerAuthTx = await liquidityPoolClient.set_position_manager({
      admin: publicKey,
      position_manager: contracts['position-manager'],
    });

    await setPositionManagerAuthTx.signAndSend({
      signTransaction: async (xdr: string) => {
        const tx = TransactionBuilder.fromXDR(xdr, networkConfig.networkPassphrase);
        tx.sign(sourceKeypair);
        return { signedTxXdr: tx.toXDR() };
      }
    });
    console.log('   ✓ PositionManager authorized to reserve/release liquidity\n');

    // 5. Initialize MarketManager
    console.log('5️⃣  Initializing MarketManager...');
    const marketManagerClient = new MarketManagerClient({
      ...clientOptions,
      contractId: contracts['market-manager'],
    });

    const marketInitTx = await marketManagerClient.initialize({
      config_manager: contracts['config-manager'],
      admin: publicKey,
    });

    await marketInitTx.signAndSend({
      signTransaction: async (xdr: string) => {
        const tx = TransactionBuilder.fromXDR(xdr, networkConfig.networkPassphrase);
        tx.sign(sourceKeypair);
        return { signedTxXdr: tx.toXDR() };
      }
    });
    console.log('   ✓ MarketManager initialized');

    // 5b. Authorize PositionManager to update OI
    console.log('   → Authorizing PositionManager to update open interest...');
    const setMarketPositionManagerTx = await marketManagerClient.set_position_manager({
      admin: publicKey,
      position_manager: contracts['position-manager'],
    });

    await setMarketPositionManagerTx.signAndSend({
      signTransaction: async (xdr: string) => {
        const tx = TransactionBuilder.fromXDR(xdr, networkConfig.networkPassphrase);
        tx.sign(sourceKeypair);
        return { signedTxXdr: tx.toXDR() };
      }
    });
    console.log('   ✓ PositionManager authorized for MarketManager\n');

    // 6. Initialize PositionManager
    console.log('6️⃣  Initializing PositionManager...');
    const positionManagerClient = new PositionManagerClient({
      ...clientOptions,
      contractId: contracts['position-manager'],
    });

    const positionInitTx = await positionManagerClient.initialize({
      admin: publicKey,
      config_manager: contracts['config-manager'],
    });

    await positionInitTx.signAndSend({
      signTransaction: async (xdr: string) => {
        const tx = TransactionBuilder.fromXDR(xdr, networkConfig.networkPassphrase);
        tx.sign(sourceKeypair);
        return { signedTxXdr: tx.toXDR() };
      }
    });
    console.log('   ✓ PositionManager initialized\n');

    // 7. Set Reflector Oracle address (testnet)
    console.log('7️⃣  Configuring Reflector Oracle...');
    // Reflector testnet oracle address - see https://reflector.network/docs
    // Note: Testnet addresses may change after network resets, verify at reflector.network
    const REFLECTOR_TESTNET_ADDRESS = 'CCY0ZJCOPG34LLQQ7N24YXBM7LL62R7ONMZ3G6WZAAYPB5OYKOMJRN63';

    const setReflectorTx = await configManagerClient.set_reflector_oracle({
      admin: publicKey,
      contract: REFLECTOR_TESTNET_ADDRESS,
    });

    await setReflectorTx.signAndSend({
      signTransaction: async (xdr: string) => {
        const tx = TransactionBuilder.fromXDR(xdr, networkConfig.networkPassphrase);
        tx.sign(sourceKeypair);
        return { signedTxXdr: tx.toXDR() };
      }
    });
    console.log(`   ✓ Reflector Oracle set to: ${REFLECTOR_TESTNET_ADDRESS}\n`);

    // 8. Create markets (XLM, BTC, ETH)
    console.log('8️⃣  Creating markets...');

    const markets = [
      { id: 0, ticker: 'XLM', maxOI: BigInt('1000000000000000'), maxFunding: 1000 },  // 10% max funding
      { id: 1, ticker: 'BTC', maxOI: BigInt('1000000000000000'), maxFunding: 1000 },
      { id: 2, ticker: 'ETH', maxOI: BigInt('1000000000000000'), maxFunding: 1000 },
    ];

    for (const market of markets) {
      console.log(`   → Creating ${market.ticker} market (id=${market.id})...`);
      const createMarketTx = await marketManagerClient.create_market({
        admin: publicKey,
        market_id: market.id,
        ticker: market.ticker,
        max_open_interest: market.maxOI,
        max_funding_rate: BigInt(market.maxFunding),
      });

      await createMarketTx.signAndSend({
        signTransaction: async (xdr: string) => {
          const tx = TransactionBuilder.fromXDR(xdr, networkConfig.networkPassphrase);
          tx.sign(sourceKeypair);
          return { signedTxXdr: tx.toXDR() };
        }
      });
      console.log(`   ✓ ${market.ticker} market created`);
    }
    console.log('');

    // 9. Enable test mode for oracle (recommended for testnet)
    console.log('9️⃣  Configuring Oracle test mode...');

    // Set test mode with base prices
    // Prices are in 1e7 format: $1.00 = 10_000_000
    const testBasePrices = new Map<number, bigint>([
      [0, BigInt(10_000_000)],      // XLM: $1.00
      [1, BigInt(500_000_000_000)], // BTC: $50,000
      [2, BigInt(30_000_000_000)],  // ETH: $3,000
    ]);

    const setTestModeTx = await oracleIntegratorClient.set_test_mode({
      admin: publicKey,
      enabled: true,
      base_prices: testBasePrices,
    });

    await setTestModeTx.signAndSend({
      signTransaction: async (xdr: string) => {
        const tx = TransactionBuilder.fromXDR(xdr, networkConfig.networkPassphrase);
        tx.sign(sourceKeypair);
        return { signedTxXdr: tx.toXDR() };
      }
    });
    console.log('   ✓ Oracle test mode enabled with base prices');

    // Enable fixed price mode (no oscillation) for predictable testing
    const setFixedPriceTx = await oracleIntegratorClient.set_fixed_price_mode({
      admin: publicKey,
      enabled: true,
    });

    await setFixedPriceTx.signAndSend({
      signTransaction: async (xdr: string) => {
        const tx = TransactionBuilder.fromXDR(xdr, networkConfig.networkPassphrase);
        tx.sign(sourceKeypair);
        return { signedTxXdr: tx.toXDR() };
      }
    });
    console.log('   ✓ Fixed price mode enabled (no oscillation)\n');

    // 10. Set staleness threshold to 300 seconds (to match Reflector's 5-minute updates)
    console.log('🔟 Configuring price staleness threshold...');
    const setTimeParamsTx = await configManagerClient.set_time_params({
      admin: publicKey,
      funding_interval: BigInt(60),      // 60 seconds
      staleness_threshold: BigInt(300),  // 300 seconds (5 minutes) for Reflector
    });

    await setTimeParamsTx.signAndSend({
      signTransaction: async (xdr: string) => {
        const tx = TransactionBuilder.fromXDR(xdr, networkConfig.networkPassphrase);
        tx.sign(sourceKeypair);
        return { signedTxXdr: tx.toXDR() };
      }
    });
    console.log('   ✓ Staleness threshold set to 300 seconds (matches Reflector 5-min updates)\n');

    console.log('\n✅ All contracts initialized successfully!\n');
    console.log('Contract addresses:');
    Object.entries(contracts).forEach(([name, address]) => {
      console.log(`   ${name.padEnd(20)} → ${address}`);
    });

    console.log('\n📝 Next steps:');
    console.log('   1. Mint test tokens from the faucet');
    console.log('   2. Fund the liquidity pool for trading');
    console.log('   3. Optionally switch to production oracle by disabling test mode\n');
    console.log('⚠️  Note: Oracle is in TEST MODE with fixed prices.');
    console.log('   To use live Reflector prices, call oracle_integrator.set_test_mode(false)\n');

  } catch (error) {
    console.error('\n❌ Initialization failed:', error);
    process.exit(1);
  }
}

main();

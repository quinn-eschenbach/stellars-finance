#!/usr/bin/env tsx

/**
 * Enable test oracle mode (use simulated prices)
 *
 * This script enables test mode on the oracle-integrator contract
 * with configurable base prices for testing.
 *
 * Usage:
 *   npm run oracle:test:testnet
 *   npm run oracle:test:mainnet
 */

import 'dotenv/config';
import { readFileSync } from 'fs';
import { NETWORK_CONFIGS } from './config';
import { Client as OracleIntegratorClient } from '@stellars-finance/oracle-integrator';
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

console.log(`\n🧪 Enabling test oracle on ${network.toUpperCase()}\n`);

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

// Load source keypair from environment
const sourceSecretKey = process.env.STELLAR_SECRET_KEY;
if (!sourceSecretKey) {
  console.error('Error: STELLAR_SECRET_KEY environment variable not set');
  console.error('Please set it to your deployer account secret key');
  process.exit(1);
}

const sourceKeypair = Keypair.fromSecret(sourceSecretKey);
const publicKey = sourceKeypair.publicKey();

console.log(`Using admin account: ${publicKey}\n`);

const clientOptions = {
  publicKey,
  networkPassphrase: networkConfig.networkPassphrase,
  rpcUrl: networkConfig.rpcUrl,
};

// Test base prices (1e7 format: $1.00 = 10_000_000)
const TEST_BASE_PRICES = new Map<number, bigint>([
  [0, BigInt(10_000_000)],      // XLM: $1.00
  [1, BigInt(500_000_000_000)], // BTC: $50,000
  [2, BigInt(30_000_000_000)],  // ETH: $3,000
]);

async function main() {
  try {
    const oracleIntegratorClient = new OracleIntegratorClient({
      ...clientOptions,
      contractId: contracts['oracle-integrator'],
    });

    // Enable test mode with base prices
    console.log('🧪 Enabling test mode with base prices...');
    console.log('   XLM: $1.00');
    console.log('   BTC: $50,000');
    console.log('   ETH: $3,000\n');

    const setTestModeTx = await oracleIntegratorClient.set_test_mode({
      admin: publicKey,
      enabled: true,
      base_prices: TEST_BASE_PRICES,
    });

    await setTestModeTx.signAndSend({
      signTransaction: async (xdr: string) => {
        const tx = TransactionBuilder.fromXDR(xdr, networkConfig.networkPassphrase);
        tx.sign(sourceKeypair);
        return { signedTxXdr: tx.toXDR() };
      }
    });
    console.log('   ✓ Test mode enabled\n');

    // Enable fixed price mode (no oscillation)
    console.log('📌 Enabling fixed price mode (no oscillation)...');
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
    console.log('   ✓ Fixed price mode enabled\n');

    console.log('✅ Test oracle mode enabled!\n');
    console.log('The oracle-integrator will now return fixed test prices.');
    console.log(`Oracle contract: ${contracts['oracle-integrator']}\n`);

    console.log('To switch to production (live Reflector prices):');
    console.log(`   npm run oracle:production:${network}\n`);

  } catch (error) {
    console.error('\n❌ Failed to enable test oracle:', error);
    process.exit(1);
  }
}

main();

#!/usr/bin/env tsx

/**
 * Update Reflector Oracle address in ConfigManager
 *
 * This script updates the Reflector oracle address for already-deployed contracts.
 * Use this when the oracle address needs to be changed without redeploying.
 *
 * Usage:
 *   npm run oracle:update:testnet
 *   npm run oracle:update:mainnet
 */

import 'dotenv/config';
import { readFileSync } from 'fs';
import { NETWORK_CONFIGS } from './config';
import { Client as ConfigManagerClient } from '@stellars-finance/config-manager';
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

// Reflector oracle addresses
const REFLECTOR_ADDRESSES: Record<NetworkType, string> = {
  testnet: 'CCYOZJCOPG34LLQQ7N24YXBM7LL62R7ONMZ3G6WZAAYPB5OYKOMJRN63',
  mainnet: '', // Update if mainnet differs
};

console.log(`\n🔄 Updating Reflector Oracle configuration on ${network.toUpperCase()}\n`);

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

async function main() {
  try {
    const configManagerClient = new ConfigManagerClient({
      ...clientOptions,
      contractId: contracts['config-manager'],
    });

    const newOracleAddress = REFLECTOR_ADDRESSES[network];

    console.log(`📡 Setting Reflector Oracle address...`);
    console.log(`   New address: ${newOracleAddress}`);

    const setOracleTx = await configManagerClient.set_reflector_oracle({
      admin: publicKey,
      contract: newOracleAddress,
    });

    await setOracleTx.signAndSend({
      signTransaction: async (xdr: string) => {
        const tx = TransactionBuilder.fromXDR(xdr, networkConfig.networkPassphrase);
        tx.sign(sourceKeypair);
        return { signedTxXdr: tx.toXDR() };
      }
    });
    console.log('   ✓ Reflector Oracle address updated\n');

    // Update staleness threshold to match Reflector's 5-minute update interval
    console.log(`⏱️  Updating price staleness threshold...`);
    console.log(`   New threshold: 300 seconds (5 minutes)`);

    const setTimeParamsTx = await configManagerClient.set_time_params({
      admin: publicKey,
      funding_interval: BigInt(60),      // Keep funding interval at 60 seconds
      staleness_threshold: BigInt(300),  // Increase to 300 seconds for Reflector
    });

    await setTimeParamsTx.signAndSend({
      signTransaction: async (xdr: string) => {
        const tx = TransactionBuilder.fromXDR(xdr, networkConfig.networkPassphrase);
        tx.sign(sourceKeypair);
        return { signedTxXdr: tx.toXDR() };
      }
    });
    console.log('   ✓ Staleness threshold updated to 300 seconds\n');

    console.log('✅ Oracle configuration updated successfully!\n');
    console.log(`ConfigManager: ${contracts['config-manager']}`);
    console.log(`Reflector Oracle: ${newOracleAddress}\n`);

    console.log('📝 Next steps:');
    console.log('   1. If using test mode, no changes needed - it ignores Reflector');
    console.log('   2. To use live prices, run: npm run oracle:production:' + network);
    console.log('');

  } catch (error) {
    console.error('\n❌ Failed to update oracle address:', error);
    process.exit(1);
  }
}

main();

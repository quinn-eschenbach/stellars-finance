#!/usr/bin/env tsx

/**
 * Enable production oracle mode (use real Reflector prices)
 *
 * This script disables test mode on the oracle-integrator contract,
 * causing it to fetch live prices from the Reflector oracle network.
 *
 * Usage:
 *   npm run oracle:production:testnet
 *   npm run oracle:production:mainnet
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

console.log(`\n🔄 Enabling production oracle on ${network.toUpperCase()}\n`);

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
    const oracleIntegratorClient = new OracleIntegratorClient({
      ...clientOptions,
      contractId: contracts['oracle-integrator'],
    });

    // Disable test mode - this will cause the oracle to use Reflector prices
    console.log('📡 Disabling test mode...');
    const disableTestModeTx = await oracleIntegratorClient.set_test_mode({
      admin: publicKey,
      enabled: false,
      base_prices: new Map(), // Empty map since we're disabling
    });

    await disableTestModeTx.signAndSend({
      signTransaction: async (xdr: string) => {
        const tx = TransactionBuilder.fromXDR(xdr, networkConfig.networkPassphrase);
        tx.sign(sourceKeypair);
        return { signedTxXdr: tx.toXDR() };
      }
    });
    console.log('   ✓ Test mode disabled\n');

    console.log('✅ Production oracle mode enabled!\n');
    console.log('The oracle-integrator will now fetch live prices from Reflector.');
    console.log(`Oracle contract: ${contracts['oracle-integrator']}\n`);

    console.log('⚠️  Important notes:');
    console.log('   - Ensure Reflector oracle address is correctly configured');
    console.log('   - Prices will now reflect real market conditions');
    console.log('   - To revert to test mode, run: npm run oracle:test:' + network + '\n');

  } catch (error) {
    console.error('\n❌ Failed to enable production oracle:', error);
    process.exit(1);
  }
}

main();

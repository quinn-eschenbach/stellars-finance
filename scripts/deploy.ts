#!/usr/bin/env tsx

/**
 * Deploy all Stellars Finance contracts to Stellar network
 *
 * Usage:
 *   npm run deploy:testnet
 *   npm run deploy:mainnet
 */

import { execSync } from 'child_process';
import { writeFileSync } from 'fs';
import { CONTRACT_ALIASES } from './config';

type NetworkType = 'testnet' | 'mainnet';

interface DeploymentResult {
  contractName: string;
  alias: string;
  contractId: string;
  wasmPath: string;
}

// Parse command line arguments
const args = process.argv.slice(2);
const networkArg = args.find(arg => arg.startsWith('--network='));
const network: NetworkType = (networkArg?.split('=')[1] as NetworkType) || 'testnet';

if (!['testnet', 'mainnet'].includes(network)) {
  console.error('Error: Network must be either "testnet" or "mainnet"');
  process.exit(1);
}

// Source account for signing transactions
const sourceAccount = process.env.STELLAR_ACCOUNT || 'perps-testnet';

console.log(`\n🚀 Deploying Stellars Finance contracts to ${network.toUpperCase()}\n`);
console.log(`Using source account: ${sourceAccount}`);
console.log('Make sure you have funded this account and built the contracts first!\n');

const deployments: DeploymentResult[] = [];

// Contracts to deploy in order
const contracts = [
  { name: 'faucet-token', alias: CONTRACT_ALIASES.faucetToken },
  { name: 'config-manager', alias: CONTRACT_ALIASES.configManager },
  { name: 'custom-oracle', alias: CONTRACT_ALIASES.customOracle },
  { name: 'oracle-integrator', alias: CONTRACT_ALIASES.oracleIntegrator },
  { name: 'liquidity-pool', alias: CONTRACT_ALIASES.liquidityPool },
  { name: 'market-manager', alias: CONTRACT_ALIASES.marketManager },
  { name: 'position-manager', alias: CONTRACT_ALIASES.positionManager },
];

for (const contract of contracts) {
  console.log(`\n📦 Deploying ${contract.name}...`);

  const wasmPath = `contracts/target/wasm32v1-none/release/${contract.name.replace(/-/g, '_')}.wasm`;

  try {
    const command = `stellar contract deploy \\
      --wasm ${wasmPath} \\
      --source-account ${sourceAccount} \\
      --network ${network} \\
      --alias ${contract.alias}`;

    console.log(`   Command: ${command.replace(/\\/g, '').replace(/\s+/g, ' ')}`);

    const contractId = execSync(command, {
      encoding: 'utf-8',
      stdio: ['pipe', 'pipe', 'inherit'],
    }).trim();

    console.log(`   ✓ Deployed: ${contractId}`);

    deployments.push({
      contractName: contract.name,
      alias: contract.alias,
      contractId,
      wasmPath,
    });
  } catch (error) {
    console.error(`   ✗ Failed to deploy ${contract.name}`);
    console.error(error);
    process.exit(1);
  }
}

// Save deployment results
const deploymentFile = `deployments/${network}.json`;
const deploymentData = {
  network,
  timestamp: new Date().toISOString(),
  sourceAccount,
  contracts: deployments.reduce((acc, dep) => {
    acc[dep.alias] = dep.contractId;
    return acc;
  }, {} as Record<string, string>),
};

try {
  execSync('mkdir -p deployments');
  writeFileSync(deploymentFile, JSON.stringify(deploymentData, null, 2));
  console.log(`\n💾 Deployment data saved to ${deploymentFile}`);
} catch (error) {
  console.error('Failed to save deployment data:', error);
}

// No need to update config.ts - it now reads from the deployment JSON file
console.log('\n✓ config.ts will automatically load addresses from deployment JSON');

console.log('\n✅ Deployment complete!\n');
console.log('Deployed contracts:');
deployments.forEach(dep => {
  console.log(`   ${dep.contractName.padEnd(20)} → ${dep.contractId}`);
});

console.log('\nNext steps:');
console.log(`   1. Initialize the contracts: npm run initialize:${network}`);
console.log(`   2. Generate TypeScript bindings with deployed contract IDs if needed`);
console.log(`   3. Test the deployment with the frontend\n`);

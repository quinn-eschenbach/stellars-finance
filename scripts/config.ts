/**
 * Network configuration and contract addresses for Stellars Finance
 */

import { readFileSync, existsSync } from 'fs';
import { join } from 'path';

export type NetworkType = 'testnet' | 'mainnet';

export interface NetworkConfig {
  rpcUrl: string;
  networkPassphrase: string;
  horizonUrl?: string;
}

export interface ContractAddresses {
  faucetToken: string;
  positionManager: string;
  liquidityPool: string;
  configManager: string;
  marketManager: string;
  oracleIntegrator: string;
  customOracle: string;
}

interface DeploymentData {
  network: string;
  timestamp: string;
  sourceAccount: string;
  contracts: Record<string, string>;
}

// Network configurations
export const NETWORK_CONFIGS: Record<NetworkType, NetworkConfig> = {
  testnet: {
    rpcUrl: 'https://soroban-testnet.stellar.org:443',
    networkPassphrase: 'Test SDF Network ; September 2015',
    horizonUrl: 'https://horizon-testnet.stellar.org',
  },
  mainnet: {
    rpcUrl: 'https://soroban.stellar.org:443',
    networkPassphrase: 'Public Global Stellar Network ; September 2015',
    horizonUrl: 'https://horizon.stellar.org',
  },
};

// Load contract addresses from deployment JSON files
function loadContractAddresses(network: NetworkType): Partial<ContractAddresses> {
  const deploymentPath = join(process.cwd(), 'deployments', `${network}.json`);

  if (!existsSync(deploymentPath)) {
    console.warn(`No deployment file found for ${network} at ${deploymentPath}`);
    return {};
  }

  try {
    const deploymentData: DeploymentData = JSON.parse(readFileSync(deploymentPath, 'utf-8'));

    return {
      faucetToken: deploymentData.contracts['faucet-token'],
      positionManager: deploymentData.contracts['position-manager'],
      liquidityPool: deploymentData.contracts['liquidity-pool'],
      configManager: deploymentData.contracts['config-manager'],
      marketManager: deploymentData.contracts['market-manager'],
      oracleIntegrator: deploymentData.contracts['oracle-integrator'],
      customOracle: deploymentData.contracts['custom-oracle'],
    };
  } catch (error) {
    console.error(`Failed to load deployment data for ${network}:`, error);
    return {};
  }
}

// Contract addresses loaded from deployment files
export const CONTRACT_ADDRESSES: Record<NetworkType, Partial<ContractAddresses>> = {
  testnet: loadContractAddresses('testnet'),
  mainnet: loadContractAddresses('mainnet'),
};

// Contract aliases for stellar CLI
export const CONTRACT_ALIASES = {
  faucetToken: 'faucet-token',
  positionManager: 'position-manager',
  liquidityPool: 'liquidity-pool',
  configManager: 'config-manager',
  marketManager: 'market-manager',
  oracleIntegrator: 'oracle-integrator',
  customOracle: 'custom-oracle',
} as const;

export function getNetworkConfig(network: NetworkType): NetworkConfig {
  return NETWORK_CONFIGS[network];
}

export function getContractAddresses(network: NetworkType): Partial<ContractAddresses> {
  return CONTRACT_ADDRESSES[network];
}

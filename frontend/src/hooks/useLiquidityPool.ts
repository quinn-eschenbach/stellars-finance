/**
 * TanStack Query hooks for Liquidity Pool Contract
 *
 * Provides React hooks for interacting with the liquidity pool smart contract.
 * Uses TanStack Query for caching, loading states, and automatic refetching.
 */

import { useQuery, useMutation, useQueryClient, UseQueryOptions } from '@tanstack/react-query';
import { useWallet } from './useWallet';
import {
  getUserShares,
  getTotalShares,
  getTotalDeposits,
  getUserTokenBalance,
  createDepositTransaction,
  createWithdrawTransaction,
  createApproveTransaction,
  fromContractAmount,
  toContractAmount,
  calculateUserShareValue,
  getReservedLiquidity,
  getAvailableLiquidity,
  getUtilizationRatio,
} from '../services/liquidityPool';
import { signTransaction } from '@stellar/freighter-api';

// Query keys for cache management
export const LIQUIDITY_POOL_KEYS = {
  userShares: (address: string) => ['liquidityPool', 'userShares', address] as const,
  totalShares: () => ['liquidityPool', 'totalShares'] as const,
  totalDeposits: () => ['liquidityPool', 'totalDeposits'] as const,
  userTokenBalance: (address: string) => ['liquidityPool', 'userTokenBalance', address] as const,
  userShareValue: (address: string) => ['liquidityPool', 'userShareValue', address] as const,
  reservedLiquidity: () => ['liquidityPool', 'reservedLiquidity'] as const,
  availableLiquidity: () => ['liquidityPool', 'availableLiquidity'] as const,
  utilization: () => ['liquidityPool', 'utilization'] as const,
};

/**
 * Hook to get user's LP shares
 * @returns Query result with user's LP shares
 */
export function useUserShares() {
  const { publicKey } = useWallet();

  return useQuery({
    queryKey: LIQUIDITY_POOL_KEYS.userShares(publicKey || ''),
    queryFn: async () => {
      if (!publicKey) throw new Error('Wallet not connected');
      const shares = await getUserShares(publicKey);
      return {
        raw: shares,
        formatted: fromContractAmount(shares),
      };
    },
    enabled: !!publicKey,
    staleTime: 10000, // 10 seconds
    refetchInterval: 30000, // Refetch every 30 seconds
  });
}

/**
 * Hook to get total LP shares in circulation
 * @returns Query result with total shares
 */
export function useTotalShares() {
  return useQuery({
    queryKey: LIQUIDITY_POOL_KEYS.totalShares(),
    queryFn: async () => {
      const shares = await getTotalShares();
      return {
        raw: shares,
        formatted: fromContractAmount(shares),
      };
    },
    staleTime: 10000,
    refetchInterval: 30000,
  });
}

/**
 * Hook to get total deposits in the pool
 * @returns Query result with total deposits
 */
export function useTotalDeposits() {
  return useQuery({
    queryKey: LIQUIDITY_POOL_KEYS.totalDeposits(),
    queryFn: async () => {
      const deposits = await getTotalDeposits();
      return {
        raw: deposits,
        formatted: fromContractAmount(deposits),
      };
    },
    staleTime: 10000,
    refetchInterval: 30000,
  });
}

/**
 * Hook to get user's token balance
 * @returns Query result with user's token balance
 */
export function useUserTokenBalance() {
  const { publicKey } = useWallet();

  return useQuery({
    queryKey: LIQUIDITY_POOL_KEYS.userTokenBalance(publicKey || ''),
    queryFn: async () => {
      if (!publicKey) throw new Error('Wallet not connected');
      const balance = await getUserTokenBalance(publicKey);
      return {
        raw: balance,
        formatted: fromContractAmount(balance),
      };
    },
    enabled: !!publicKey,
    staleTime: 10000,
    refetchInterval: 30000,
  });
}

/**
 * Hook to calculate user's share value in the pool
 * Uses actual pool balance (available + reserved) to include profits/losses
 * @returns Query result with user's share value
 */
export function useUserShareValue() {
  const { publicKey } = useWallet();
  const { data: userShares } = useUserShares();
  const { data: totalShares } = useTotalShares();
  const { data: reservedLiquidity } = useReservedLiquidity();
  const { data: availableLiquidity } = useAvailableLiquidity();

  return useQuery({
    queryKey: LIQUIDITY_POOL_KEYS.userShareValue(publicKey || ''),
    queryFn: () => {
      if (!userShares || !totalShares || !reservedLiquidity || !availableLiquidity) {
        return { raw: BigInt(0), formatted: 0 };
      }

      // Use actual pool balance (available + reserved) instead of totalDeposits
      // This ensures profits/losses from trading are reflected in share value
      const actualPoolBalance = availableLiquidity.raw + reservedLiquidity.raw;

      const value = calculateUserShareValue(
        userShares.raw,
        totalShares.raw,
        actualPoolBalance
      );

      return {
        raw: value,
        formatted: fromContractAmount(value),
      };
    },
    enabled: !!publicKey && !!userShares && !!totalShares && !!reservedLiquidity && !!availableLiquidity,
    staleTime: 10000,
  });
}

/**
 * Hook to deposit tokens into the liquidity pool
 * @returns Mutation function and state
 */
export function useDeposit() {
  const { publicKey } = useWallet();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (amount: number) => {
      if (!publicKey) throw new Error('Wallet not connected');

      const contractAmount = toContractAmount(amount);

      // Create and sign deposit transaction
      // Note: No approval needed - Soroban uses auth via user.require_auth()
      const depositTx = await createDepositTransaction(publicKey, contractAmount);

      // Sign and send deposit transaction
      const depositResult = await depositTx.signAndSend({
        signTransaction: async (xdr) => {
          return await signTransaction(xdr, {
            networkPassphrase: depositTx.options.networkPassphrase,
          });
        },
      });

      return {
        depositResult,
        sharesMinted: depositResult,
      };
    },
    onSuccess: () => {
      // Invalidate and refetch relevant queries
      if (publicKey) {
        queryClient.invalidateQueries({ queryKey: LIQUIDITY_POOL_KEYS.userShares(publicKey) });
        queryClient.invalidateQueries({ queryKey: LIQUIDITY_POOL_KEYS.userTokenBalance(publicKey) });
        queryClient.invalidateQueries({ queryKey: LIQUIDITY_POOL_KEYS.userShareValue(publicKey) });
      }
      queryClient.invalidateQueries({ queryKey: LIQUIDITY_POOL_KEYS.totalShares() });
      queryClient.invalidateQueries({ queryKey: LIQUIDITY_POOL_KEYS.totalDeposits() });
    },
  });
}

/**
 * Hook to withdraw tokens from the liquidity pool
 * @returns Mutation function and state
 */
export function useWithdraw() {
  const { publicKey } = useWallet();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (shares: number) => {
      if (!publicKey) throw new Error('Wallet not connected');

      const contractShares = toContractAmount(shares);

      // Create withdraw transaction
      const withdrawTx = await createWithdrawTransaction(publicKey, contractShares);

      // Sign and send using Freighter
      const result = await withdrawTx.signAndSend({
        signTransaction: async (xdr) => {
          return await signTransaction(xdr, {
            networkPassphrase: withdrawTx.options.networkPassphrase,
          });
        },
      });

      return {
        result,
        tokensReturned: result,
      };
    },
    onSuccess: () => {
      // Invalidate and refetch relevant queries
      if (publicKey) {
        queryClient.invalidateQueries({ queryKey: LIQUIDITY_POOL_KEYS.userShares(publicKey) });
        queryClient.invalidateQueries({ queryKey: LIQUIDITY_POOL_KEYS.userTokenBalance(publicKey) });
        queryClient.invalidateQueries({ queryKey: LIQUIDITY_POOL_KEYS.userShareValue(publicKey) });
      }
      queryClient.invalidateQueries({ queryKey: LIQUIDITY_POOL_KEYS.totalShares() });
      queryClient.invalidateQueries({ queryKey: LIQUIDITY_POOL_KEYS.totalDeposits() });
    },
  });
}

/**
 * Hook to get reserved liquidity
 * @returns Query result with reserved liquidity
 */
export function useReservedLiquidity() {
  return useQuery({
    queryKey: LIQUIDITY_POOL_KEYS.reservedLiquidity(),
    queryFn: async () => {
      const reserved = await getReservedLiquidity();
      return {
        raw: reserved,
        formatted: fromContractAmount(reserved),
      };
    },
    staleTime: 10000,
    refetchInterval: 30000,
  });
}

/**
 * Hook to get available liquidity
 * @returns Query result with available liquidity
 */
export function useAvailableLiquidity() {
  return useQuery({
    queryKey: LIQUIDITY_POOL_KEYS.availableLiquidity(),
    queryFn: async () => {
      const available = await getAvailableLiquidity();
      return {
        raw: available,
        formatted: fromContractAmount(available),
      };
    },
    staleTime: 10000,
    refetchInterval: 30000,
  });
}

/**
 * Hook to get pool utilization ratio
 * @returns Query result with utilization ratio as percentage
 */
export function useUtilizationRatio() {
  return useQuery({
    queryKey: LIQUIDITY_POOL_KEYS.utilization(),
    queryFn: async () => {
      const utilizationBps = await getUtilizationRatio();
      return {
        bps: utilizationBps,
        percentage: utilizationBps / 100,
      };
    },
    staleTime: 10000,
    refetchInterval: 30000,
  });
}

/**
 * Hook to get all pool data at once
 * @returns Combined pool statistics
 */
export function usePoolStats() {
  const userShares = useUserShares();
  const totalShares = useTotalShares();
  const totalDeposits = useTotalDeposits();
  const userTokenBalance = useUserTokenBalance();
  const userShareValue = useUserShareValue();
  const reservedLiquidity = useReservedLiquidity();
  const availableLiquidity = useAvailableLiquidity();
  const utilization = useUtilizationRatio();

  return {
    userShares: userShares.data?.formatted || 0,
    totalShares: totalShares.data?.formatted || 0,
    totalDeposits: totalDeposits.data?.formatted || 0,
    userTokenBalance: userTokenBalance.data?.formatted || 0,
    userShareValue: userShareValue.data?.formatted || 0,
    reservedLiquidity: reservedLiquidity.data?.formatted || 0,
    availableLiquidity: availableLiquidity.data?.formatted || 0,
    utilization: utilization.data?.percentage || 0,
    isLoading:
      userShares.isLoading ||
      totalShares.isLoading ||
      totalDeposits.isLoading ||
      userTokenBalance.isLoading ||
      userShareValue.isLoading ||
      reservedLiquidity.isLoading ||
      availableLiquidity.isLoading ||
      utilization.isLoading,
    isError:
      userShares.isError ||
      totalShares.isError ||
      totalDeposits.isError ||
      userTokenBalance.isError ||
      userShareValue.isError ||
      reservedLiquidity.isError ||
      availableLiquidity.isError ||
      utilization.isError,
  };
}

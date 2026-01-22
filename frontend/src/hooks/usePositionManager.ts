/**
 * TanStack Query hooks for Position Manager Contract
 *
 * Provides React hooks for interacting with the position manager smart contract.
 * Uses TanStack Query for caching, loading states, and automatic refetching.
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useWallet } from './useWallet';
import {
  getUserPositions,
  getPosition,
  calculatePnL,
  createOpenPositionTransaction,
  createClosePositionTransaction,
  fromContractAmount,
  fromContractPrice,
  toContractAmount,
  toContractPrice,
  calculateLiquidationPrice,
  calculateLeverage,
  getUserTokenBalance,
  getUserOrders,
  getOrder,
  getPositionOrders,
  getMinExecutionFee,
  createLimitOrderTransaction,
  createStopLossTransaction,
  createTakeProfitTransaction,
  createCancelOrderTransaction,
  MarketId,
  Order,
  OrderType,
} from '../services/positionManager';

// Re-export types for UI components
export type { Order, OrderType };
export { MarketId };
import { signTransaction } from '@stellar/freighter-api';
import { toast } from 'sonner';

// Query keys for cache management
export const POSITION_MANAGER_KEYS = {
  userPositions: (address: string) => ['positionManager', 'userPositions', address] as const,
  position: (positionId: bigint) => ['positionManager', 'position', positionId.toString()] as const,
  positionPnL: (positionId: bigint) => ['positionManager', 'pnl', positionId.toString()] as const,
  userTokenBalance: (address: string) => ['positionManager', 'userTokenBalance', address] as const,
  userOrders: (address: string) => ['positionManager', 'userOrders', address] as const,
  order: (orderId: bigint) => ['positionManager', 'order', orderId.toString()] as const,
  positionOrders: (positionId: bigint) => ['positionManager', 'positionOrders', positionId.toString()] as const,
  minExecutionFee: () => ['positionManager', 'minExecutionFee'] as const,
};

/**
 * Formatted position data for UI display
 */
export interface FormattedPosition {
  id: bigint;
  trader: string;
  marketId: number;
  collateral: number;
  size: number;
  isLong: boolean;
  entryPrice: number;
  leverage: number;
  liquidationPrice: number;
  pnl?: number;
  pnlPercent?: number;
}

/**
 * Order type as readable string
 */
export type OrderTypeString = 'Limit' | 'StopLoss' | 'TakeProfit';

/**
 * Get order type as string
 */
export function getOrderTypeString(orderType: OrderType): OrderTypeString {
  if ('tag' in orderType) {
    return orderType.tag as OrderTypeString;
  }
  return 'Limit';
}

/**
 * Market names for display
 */
export const MARKET_NAMES: Record<number, string> = {
  0: 'XLM-PERP',
  1: 'BTC-PERP',
  2: 'ETH-PERP',
};

/**
 * Formatted order data for UI display
 */
export interface FormattedOrder {
  id: bigint;
  orderType: OrderType;
  orderTypeString: OrderTypeString;
  trader: string;
  marketId: number;
  marketName: string;
  positionId: bigint;
  triggerPrice: number;
  acceptablePrice: number;
  collateral: number;
  size: number;
  leverage: number;
  isLong: boolean;
  closePercentage: number;
  executionFee: number;
  expiration: bigint;
  createdAt: bigint;
  isExpired: boolean;
}

/**
 * Format raw order to UI-friendly format
 */
function formatOrder(order: Order): FormattedOrder {
  const now = BigInt(Math.floor(Date.now() / 1000));
  const isExpired = order.expiration > 0n && order.expiration < now;

  return {
    id: order.order_id,
    orderType: order.order_type,
    orderTypeString: getOrderTypeString(order.order_type),
    trader: order.trader,
    marketId: order.market_id,
    marketName: MARKET_NAMES[order.market_id] || `Market ${order.market_id}`,
    positionId: order.position_id,
    triggerPrice: fromContractPrice(order.trigger_price),
    acceptablePrice: fromContractPrice(order.acceptable_price),
    collateral: fromContractAmount(order.collateral),
    size: fromContractAmount(order.size),
    leverage: order.leverage,
    isLong: order.is_long,
    closePercentage: order.close_percentage / 100, // Convert 10000 -> 100%
    executionFee: fromContractAmount(order.execution_fee),
    expiration: order.expiration,
    createdAt: order.created_at,
    isExpired,
  };
}

/**
 * Hook to get all positions for the connected user
 * @returns Query result with user's positions
 */
export function useUserPositions() {
  const { publicKey } = useWallet();

  return useQuery({
    queryKey: POSITION_MANAGER_KEYS.userPositions(publicKey || ''),
    queryFn: async () => {
      if (!publicKey) throw new Error('Wallet not connected');

      const positions = await getUserPositions(publicKey);

      // Format positions for UI
      const formatted: FormattedPosition[] = positions.map((pos) => ({
        id: pos.id,
        trader: pos.trader,
        marketId: pos.market_id,
        collateral: fromContractAmount(pos.collateral),
        size: fromContractAmount(pos.size),
        isLong: pos.is_long,
        entryPrice: fromContractPrice(pos.entry_price),
        leverage: calculateLeverage(pos),
        liquidationPrice: calculateLiquidationPrice(pos),
      }));

      return formatted;
    },
    enabled: !!publicKey,
    staleTime: 5000, // 5 seconds
    refetchInterval: 10000, // Refetch every 10 seconds for live PnL updates
  });
}

/**
 * Hook to get a specific position by ID
 * @param positionId - Position ID to fetch
 * @returns Query result with position data
 */
export function usePosition(positionId: bigint | null) {
  return useQuery({
    queryKey: POSITION_MANAGER_KEYS.position(positionId || BigInt(0)),
    queryFn: async () => {
      if (!positionId) throw new Error('Position ID required');

      const position = await getPosition(positionId);

      return {
        raw: position,
        formatted: {
          id: positionId,
          trader: position.trader,
          marketId: position.market_id,
          collateral: fromContractAmount(position.collateral),
          size: fromContractAmount(position.size),
          isLong: position.is_long,
          entryPrice: fromContractPrice(position.entry_price),
          leverage: calculateLeverage(position),
          liquidationPrice: calculateLiquidationPrice(position),
        },
      };
    },
    enabled: !!positionId,
    staleTime: 5000,
  });
}

/**
 * Hook to calculate PnL for a specific position
 * Note: Currently returns 0 in MVP, but hook is ready for future implementation
 * @param positionId - Position ID
 * @returns Query result with PnL data
 */
export function usePositionPnL(positionId: bigint | null) {
  return useQuery({
    queryKey: POSITION_MANAGER_KEYS.positionPnL(positionId || BigInt(0)),
    queryFn: async () => {
      if (!positionId) throw new Error('Position ID required');

      const pnl = await calculatePnL(positionId);

      return {
        raw: pnl,
        formatted: fromContractAmount(pnl),
      };
    },
    enabled: !!positionId,
    staleTime: 5000,
    refetchInterval: 10000, // Update PnL frequently
  });
}

/**
 * Hook to get user's token balance (for collateral)
 * @returns Query result with user's token balance
 */
export function useUserTokenBalance() {
  const { publicKey } = useWallet();

  return useQuery({
    queryKey: POSITION_MANAGER_KEYS.userTokenBalance(publicKey || ''),
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
 * Parameters for opening a position
 */
export interface OpenPositionParams {
  marketId: MarketId;
  collateral: number; // Human-readable amount
  leverage: number;
  isLong: boolean;
}

/**
 * Hook to open a new position
 * @returns Mutation function and state
 */
export function useOpenPosition() {
  const { publicKey } = useWallet();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (params: OpenPositionParams) => {
      if (!publicKey) throw new Error('Wallet not connected');

      const contractCollateral = toContractAmount(params.collateral);

      // Open position (Soroban handles auth via require_auth, no separate approve needed)
      toast.info('Opening position...');
      const openTx = await createOpenPositionTransaction(
        publicKey,
        params.marketId,
        contractCollateral,
        params.leverage,
        params.isLong
      );

      const result = await openTx.signAndSend({
        signTransaction: async (xdr) => {
          return await signTransaction(xdr, {
            networkPassphrase: openTx.options.networkPassphrase,
          });
        },
      });

      return {
        result,
        positionId: result, // The contract returns the position ID
      };
    },
    onSuccess: () => {
      toast.success('Position opened successfully!');

      // Invalidate and refetch relevant queries
      if (publicKey) {
        queryClient.invalidateQueries({
          queryKey: POSITION_MANAGER_KEYS.userPositions(publicKey)
        });
        queryClient.invalidateQueries({
          queryKey: POSITION_MANAGER_KEYS.userTokenBalance(publicKey)
        });
      }
    },
    onError: (error: Error) => {
      toast.error(`Failed to open position: ${error.message}`);
    },
  });
}

/**
 * Hook to close an existing position
 * @returns Mutation function and state
 */
export function useClosePosition() {
  const { publicKey } = useWallet();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (positionId: bigint) => {
      if (!publicKey) throw new Error('Wallet not connected');

      toast.info('Closing position...');
      const closeTx = await createClosePositionTransaction(publicKey, positionId);

      const result = await closeTx.signAndSend({
        signTransaction: async (xdr) => {
          return await signTransaction(xdr, {
            networkPassphrase: closeTx.options.networkPassphrase,
          });
        },
      });

      return {
        result,
        pnl: 0, // MVP: PnL is always 0 (fixed price)
      };
    },
    onSuccess: (data) => {
      const pnl = data.pnl;

      if (pnl > 0) {
        toast.success(`Position closed with profit: $${pnl.toFixed(2)}`);
      } else if (pnl < 0) {
        toast.success(`Position closed with loss: $${Math.abs(pnl).toFixed(2)}`);
      } else {
        toast.success('Position closed successfully!');
      }

      // Invalidate and refetch relevant queries
      if (publicKey) {
        queryClient.invalidateQueries({
          queryKey: POSITION_MANAGER_KEYS.userPositions(publicKey)
        });
        queryClient.invalidateQueries({
          queryKey: POSITION_MANAGER_KEYS.userTokenBalance(publicKey)
        });
      }
    },
    onError: (error: Error) => {
      toast.error(`Failed to close position: ${error.message}`);
    },
  });
}

/**
 * Hook to get position statistics for the user
 * @returns Combined position statistics
 */
export function usePositionStats() {
  const { data: positions, isLoading, isError } = useUserPositions();

  if (!positions || positions.length === 0) {
    return {
      totalPositions: 0,
      totalCollateral: 0,
      totalSize: 0,
      longPositions: 0,
      shortPositions: 0,
      isLoading,
      isError,
    };
  }

  const stats = positions.reduce(
    (acc, pos) => {
      acc.totalCollateral += pos.collateral;
      acc.totalSize += pos.size;
      if (pos.isLong) {
        acc.longPositions += 1;
      } else {
        acc.shortPositions += 1;
      }
      return acc;
    },
    { totalCollateral: 0, totalSize: 0, longPositions: 0, shortPositions: 0 }
  );

  return {
    totalPositions: positions.length,
    ...stats,
    isLoading,
    isError,
  };
}

// ============================================================================
// Order Hooks
// ============================================================================

/**
 * Hook to get minimum execution fee for orders
 * @returns Query result with minimum execution fee
 */
export function useMinExecutionFee() {
  return useQuery({
    queryKey: POSITION_MANAGER_KEYS.minExecutionFee(),
    queryFn: async () => {
      const fee = await getMinExecutionFee();
      return {
        raw: fee,
        formatted: fromContractAmount(fee),
      };
    },
    staleTime: 60000, // 1 minute - this rarely changes
  });
}

/**
 * Hook to get all orders for the connected user
 * @returns Query result with user's orders
 */
export function useUserOrders() {
  const { publicKey } = useWallet();

  return useQuery({
    queryKey: POSITION_MANAGER_KEYS.userOrders(publicKey || ''),
    queryFn: async () => {
      if (!publicKey) throw new Error('Wallet not connected');

      const orders = await getUserOrders(publicKey);
      return orders.map(formatOrder);
    },
    enabled: !!publicKey,
    staleTime: 5000,
    refetchInterval: 10000,
  });
}

/**
 * Hook to get a specific order by ID
 * @param orderId - Order ID to fetch
 * @returns Query result with order data
 */
export function useOrder(orderId: bigint | null) {
  return useQuery({
    queryKey: POSITION_MANAGER_KEYS.order(orderId || BigInt(0)),
    queryFn: async () => {
      if (!orderId) throw new Error('Order ID required');

      const order = await getOrder(orderId);
      return {
        raw: order,
        formatted: formatOrder(order),
      };
    },
    enabled: !!orderId,
    staleTime: 5000,
  });
}

/**
 * Hook to get all orders attached to a position (SL/TP)
 * @param positionId - Position ID
 * @returns Query result with position's orders
 */
export function usePositionOrders(positionId: bigint | null) {
  return useQuery({
    queryKey: POSITION_MANAGER_KEYS.positionOrders(positionId || BigInt(0)),
    queryFn: async () => {
      if (!positionId) throw new Error('Position ID required');

      const orders = await getPositionOrders(positionId);
      return orders.map(formatOrder);
    },
    enabled: !!positionId,
    staleTime: 5000,
    refetchInterval: 10000,
  });
}

/**
 * Enriched order with associated position data (for SL/TP orders)
 */
export interface EnrichedOrder extends FormattedOrder {
  position?: FormattedPosition;
  distanceFromTrigger?: number; // Percentage distance from current price to trigger
  estimatedPnl?: number; // Estimated PnL if order executes at trigger price
}

/**
 * Hook to get all orders for user with enriched position data
 * Useful for displaying orders with their associated position info
 * @returns Query result with enriched orders
 */
export function useUserOrdersWithPositions() {
  const { data: positions } = useUserPositions();
  const { data: orders } = useUserOrders();

  // Create a map of positions for quick lookup
  const positionMap = new Map<string, FormattedPosition>();
  if (positions) {
    positions.forEach((pos) => {
      positionMap.set(pos.id.toString(), pos);
    });
  }

  // Enrich orders with position data
  const enrichedOrders: EnrichedOrder[] | undefined = orders?.map((order) => {
    const enriched: EnrichedOrder = { ...order };

    // For SL/TP orders, attach the position data
    if (order.positionId > 0n) {
      const position = positionMap.get(order.positionId.toString());
      if (position) {
        enriched.position = position;

        // Calculate distance from trigger (how far current price is from trigger)
        // Note: This would need current price from PriceContext for accurate calculation
        // For now, we calculate based on entry price as a reference
        const priceDiff = order.triggerPrice - position.entryPrice;
        enriched.distanceFromTrigger = (priceDiff / position.entryPrice) * 100;

        // Estimate PnL if order executes at trigger price
        const priceChange = order.triggerPrice - position.entryPrice;
        const pnlMultiplier = position.isLong ? 1 : -1;
        enriched.estimatedPnl = (priceChange / position.entryPrice) * position.size * pnlMultiplier;
      }
    }

    return enriched;
  });

  return {
    data: enrichedOrders,
    isLoading: !orders || !positions,
    orders,
    positions,
  };
}

/**
 * Hook to get order statistics for the user
 * @returns Combined order statistics
 */
export function useOrderStats() {
  const { data: orders, isLoading, isError } = useUserOrders();

  if (!orders || orders.length === 0) {
    return {
      totalOrders: 0,
      limitOrders: 0,
      stopLossOrders: 0,
      takeProfitOrders: 0,
      expiredOrders: 0,
      isLoading,
      isError,
    };
  }

  const stats = orders.reduce(
    (acc, order) => {
      if (order.orderTypeString === 'Limit') acc.limitOrders += 1;
      if (order.orderTypeString === 'StopLoss') acc.stopLossOrders += 1;
      if (order.orderTypeString === 'TakeProfit') acc.takeProfitOrders += 1;
      if (order.isExpired) acc.expiredOrders += 1;
      return acc;
    },
    { limitOrders: 0, stopLossOrders: 0, takeProfitOrders: 0, expiredOrders: 0 }
  );

  return {
    totalOrders: orders.length,
    ...stats,
    isLoading,
    isError,
  };
}

/**
 * Parameters for creating a limit order
 */
export interface CreateLimitOrderParams {
  marketId: MarketId;
  triggerPrice: number;
  acceptablePrice?: number;
  collateral: number;
  leverage: number;
  isLong: boolean;
  executionFee: number;
  expiration?: number; // Unix timestamp, 0 = no expiry
}

/**
 * Hook to create a limit order
 * @returns Mutation function and state
 */
export function useCreateLimitOrder() {
  const { publicKey } = useWallet();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (params: CreateLimitOrderParams) => {
      if (!publicKey) throw new Error('Wallet not connected');

      toast.info('Creating limit order...');
      const tx = await createLimitOrderTransaction(
        publicKey,
        params.marketId,
        toContractPrice(params.triggerPrice),
        toContractPrice(params.acceptablePrice || 0),
        toContractAmount(params.collateral),
        params.leverage,
        params.isLong,
        toContractAmount(params.executionFee),
        BigInt(params.expiration || 0)
      );

      const result = await tx.signAndSend({
        signTransaction: async (xdr) => {
          return await signTransaction(xdr, {
            networkPassphrase: tx.options.networkPassphrase,
          });
        },
      });

      return { orderId: result };
    },
    onSuccess: () => {
      toast.success('Limit order created!');

      if (publicKey) {
        queryClient.invalidateQueries({
          queryKey: POSITION_MANAGER_KEYS.userOrders(publicKey)
        });
        queryClient.invalidateQueries({
          queryKey: POSITION_MANAGER_KEYS.userTokenBalance(publicKey)
        });
      }
    },
    onError: (error: Error) => {
      toast.error(`Failed to create limit order: ${error.message}`);
    },
  });
}

/**
 * Parameters for creating a stop-loss order
 */
export interface CreateStopLossParams {
  positionId: bigint;
  triggerPrice: number;
  acceptablePrice?: number;
  closePercentage: number; // 1-100 (percentage)
  executionFee: number;
  expiration?: number;
}

/**
 * Hook to create a stop-loss order
 * @returns Mutation function and state
 */
export function useCreateStopLoss() {
  const { publicKey } = useWallet();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (params: CreateStopLossParams) => {
      if (!publicKey) throw new Error('Wallet not connected');

      toast.info('Creating stop-loss order...');
      const tx = await createStopLossTransaction(
        publicKey,
        params.positionId,
        toContractPrice(params.triggerPrice),
        toContractPrice(params.acceptablePrice || 0),
        Math.round(params.closePercentage * 100), // Convert 100% -> 10000
        toContractAmount(params.executionFee),
        BigInt(params.expiration || 0)
      );

      const result = await tx.signAndSend({
        signTransaction: async (xdr) => {
          return await signTransaction(xdr, {
            networkPassphrase: tx.options.networkPassphrase,
          });
        },
      });

      return { orderId: result };
    },
    onSuccess: (_data, variables) => {
      toast.success('Stop-loss order created!');

      if (publicKey) {
        queryClient.invalidateQueries({
          queryKey: POSITION_MANAGER_KEYS.userOrders(publicKey)
        });
        queryClient.invalidateQueries({
          queryKey: POSITION_MANAGER_KEYS.positionOrders(variables.positionId)
        });
        queryClient.invalidateQueries({
          queryKey: POSITION_MANAGER_KEYS.userTokenBalance(publicKey)
        });
      }
    },
    onError: (error: Error) => {
      toast.error(`Failed to create stop-loss: ${error.message}`);
    },
  });
}

/**
 * Parameters for creating a take-profit order
 */
export interface CreateTakeProfitParams {
  positionId: bigint;
  triggerPrice: number;
  acceptablePrice?: number;
  closePercentage: number; // 1-100 (percentage)
  executionFee: number;
  expiration?: number;
}

/**
 * Hook to create a take-profit order
 * @returns Mutation function and state
 */
export function useCreateTakeProfit() {
  const { publicKey } = useWallet();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (params: CreateTakeProfitParams) => {
      if (!publicKey) throw new Error('Wallet not connected');

      toast.info('Creating take-profit order...');
      const tx = await createTakeProfitTransaction(
        publicKey,
        params.positionId,
        toContractPrice(params.triggerPrice),
        toContractPrice(params.acceptablePrice || 0),
        Math.round(params.closePercentage * 100), // Convert 100% -> 10000
        toContractAmount(params.executionFee),
        BigInt(params.expiration || 0)
      );

      const result = await tx.signAndSend({
        signTransaction: async (xdr) => {
          return await signTransaction(xdr, {
            networkPassphrase: tx.options.networkPassphrase,
          });
        },
      });

      return { orderId: result };
    },
    onSuccess: (_data, variables) => {
      toast.success('Take-profit order created!');

      if (publicKey) {
        queryClient.invalidateQueries({
          queryKey: POSITION_MANAGER_KEYS.userOrders(publicKey)
        });
        queryClient.invalidateQueries({
          queryKey: POSITION_MANAGER_KEYS.positionOrders(variables.positionId)
        });
        queryClient.invalidateQueries({
          queryKey: POSITION_MANAGER_KEYS.userTokenBalance(publicKey)
        });
      }
    },
    onError: (error: Error) => {
      toast.error(`Failed to create take-profit: ${error.message}`);
    },
  });
}

/**
 * Hook to cancel an order
 * @returns Mutation function and state
 */
export function useCancelOrder() {
  const { publicKey } = useWallet();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (orderId: bigint) => {
      if (!publicKey) throw new Error('Wallet not connected');

      toast.info('Canceling order...');
      const tx = await createCancelOrderTransaction(publicKey, orderId);

      await tx.signAndSend({
        signTransaction: async (xdr) => {
          return await signTransaction(xdr, {
            networkPassphrase: tx.options.networkPassphrase,
          });
        },
      });

      return { orderId };
    },
    onSuccess: () => {
      toast.success('Order canceled!');

      if (publicKey) {
        queryClient.invalidateQueries({
          queryKey: POSITION_MANAGER_KEYS.userOrders(publicKey)
        });
        queryClient.invalidateQueries({
          queryKey: POSITION_MANAGER_KEYS.userTokenBalance(publicKey)
        });
      }
    },
    onError: (error: Error) => {
      toast.error(`Failed to cancel order: ${error.message}`);
    },
  });
}

/**
 * Position Manager Service
 *
 * Provides contract interaction for the position manager.
 * Handles opening, closing, and querying perpetual positions.
 */

import { Client as PositionManagerClient, Order, OrderType } from '@stellars-finance/position-manager';
import { Client as TokenClient } from '@stellars-finance/faucet-token';
import { CONTRACT_ADDRESSES, getRpcUrl, getNetworkPassphrase } from '../config/contracts';
import type { i128, u32, u64, u128 } from '@stellar/stellar-sdk/contract';

export type { Order, OrderType };

// Stellar typically uses 7 decimal places for tokens
const DECIMALS = 7;
const DECIMAL_MULTIPLIER = Math.pow(10, DECIMALS);

/**
 * Position data structure matching the contract
 */
export interface Position {
  trader: string;
  collateral: bigint;
  size: bigint;
  is_long: boolean;
  entry_price: bigint;
}

/**
 * Market IDs matching the contract
 */
export enum MarketId {
  XLM_PERP = 0,
  BTC_PERP = 1,
  ETH_PERP = 2,
}

/**
 * Convert display amount to contract amount (u128)
 * @param amount - Human-readable amount (e.g., 100.5)
 * @returns Contract amount as bigint
 */
export function toContractAmount(amount: number): bigint {
  return BigInt(Math.floor(amount * DECIMAL_MULTIPLIER));
}

/**
 * Convert contract amount (u128) to display amount
 * @param amount - Contract amount as bigint
 * @returns Human-readable amount
 */
export function fromContractAmount(amount: bigint): number {
  return Number(amount) / DECIMAL_MULTIPLIER;
}

/**
 * Convert display price to contract price (with 7 decimals)
 * Price format: 100,000,000 = $1.00
 * @param price - Human-readable price (e.g., 43250.50)
 * @returns Contract price as bigint
 */
export function toContractPrice(price: number): bigint {
  return BigInt(Math.floor(price * DECIMAL_MULTIPLIER));
}

/**
 * Convert contract price to display price
 * @param price - Contract price as bigint
 * @returns Human-readable price
 */
export function fromContractPrice(price: bigint): number {
  return Number(price) / DECIMAL_MULTIPLIER;
}

/**
 * Get position manager contract client
 * @param publicKey - Optional public key for transaction source account
 */
export function getPositionManagerClient(publicKey?: string) {
  return new PositionManagerClient({
    contractId: CONTRACT_ADDRESSES.positionManager,
    rpcUrl: getRpcUrl(),
    networkPassphrase: getNetworkPassphrase(),
    ...(publicKey && { publicKey }),
  });
}

/**
 * Get token contract client
 * @param publicKey - Optional public key for transaction source account
 */
export function getTokenClient(publicKey?: string) {
  return new TokenClient({
    contractId: CONTRACT_ADDRESSES.faucetToken,
    rpcUrl: getRpcUrl(),
    networkPassphrase: getNetworkPassphrase(),
    ...(publicKey && { publicKey }),
  });
}

/**
 * Get all position IDs for a user
 * @param userAddress - User's Stellar address
 * @returns Array of position IDs
 */
export async function getUserPositionIds(userAddress: string): Promise<bigint[]> {
  const client = getPositionManagerClient();
  const tx = await client.get_user_open_positions({ trader: userAddress });
  const result = await tx.simulate();
  return result.result;
}

/**
 * Get position details by ID
 * @param positionId - Position ID
 * @returns Position data
 */
export async function getPosition(positionId: bigint): Promise<Position> {
  const client = getPositionManagerClient();
  const tx = await client.get_position({ position_id: positionId as u64 });
  const result = await tx.simulate();
  return result.result;
}

/**
 * Get all positions for a user (with full details)
 * @param userAddress - User's Stellar address
 * @returns Array of positions with their IDs
 */
export async function getUserPositions(userAddress: string): Promise<Array<Position & { id: bigint }>> {
  const positionIds = await getUserPositionIds(userAddress);

  const positions = await Promise.all(
    positionIds.map(async (id) => {
      const position = await getPosition(id);
      return { ...position, id };
    })
  );

  return positions;
}

/**
 * Calculate unrealized PnL for a position
 * @param positionId - Position ID
 * @returns PnL as bigint (can be negative)
 */
export async function calculatePnL(positionId: bigint): Promise<bigint> {
  const client = getPositionManagerClient();
  const tx = await client.calculate_pnl({ position_id: positionId as u64 });
  const result = await tx.simulate();
  return result.result;
}

/**
 * Create an open position transaction
 * @param userAddress - User's Stellar address
 * @param marketId - Market identifier (0=XLM, 1=BTC, 2=ETH)
 * @param collateral - Collateral amount (in contract units)
 * @param leverage - Leverage multiplier (e.g., 5, 10, 20)
 * @param isLong - True for long, false for short
 * @returns AssembledTransaction ready for signing
 */
export async function createOpenPositionTransaction(
  userAddress: string,
  marketId: MarketId,
  collateral: bigint,
  leverage: number,
  isLong: boolean
) {
  const client = getPositionManagerClient(userAddress);
  return await client.open_position({
    trader: userAddress,
    market_id: marketId as u32,
    collateral: collateral,
    leverage: leverage as u32,
    is_long: isLong,
  });
}

/**
 * Create a close position transaction
 * @param userAddress - User's Stellar address
 * @param positionId - Position ID to close
 * @returns AssembledTransaction ready for signing
 */
export async function createClosePositionTransaction(
  userAddress: string,
  positionId: bigint
) {
  const client = getPositionManagerClient(userAddress);
  return await client.close_position({
    trader: userAddress,
    position_id: positionId as u64,
  });
}

/**
 * Create an approve transaction for the position manager
 * This allows the position manager to spend user's tokens for collateral
 * @param userAddress - User's Stellar address
 * @param amount - Amount to approve (in contract units)
 * @param liveUntilLedger - Ledger number when approval expires (optional, defaults to current + 100000)
 * @returns AssembledTransaction ready for signing
 */
export async function createApproveTransaction(
  userAddress: string,
  amount: bigint,
  liveUntilLedger?: number
) {
  const client = getTokenClient(userAddress);

  // Default to a far future ledger if not specified
  const expiration = liveUntilLedger || (await getCurrentLedger()) + 100000;

  return await client.approve({
    from: userAddress,
    spender: CONTRACT_ADDRESSES.positionManager,
    amount: amount as i128,
    live_until_ledger: expiration,
  });
}

/**
 * Get current ledger number (approximation)
 * In production, you'd want to fetch this from the RPC
 */
async function getCurrentLedger(): Promise<number> {
  // Placeholder - in production, fetch from RPC
  return 1000000;
}

/**
 * Calculate liquidation price for a position
 * Liquidation occurs when: remaining_collateral < size × 0.01 (1% maintenance margin)
 *
 * For long: liquidation_price = entry_price × (1 - collateral/size + 0.01)
 * For short: liquidation_price = entry_price × (1 + collateral/size - 0.01)
 *
 * @param position - Position data
 * @returns Liquidation price as number
 */
export function calculateLiquidationPrice(position: Position): number {
  const entryPrice = fromContractPrice(position.entry_price);
  const collateral = fromContractAmount(position.collateral);
  const size = fromContractAmount(position.size);

  const maintenanceMargin = 0.01; // 1%
  const collateralRatio = collateral / size;

  if (position.is_long) {
    // Long: price drops until collateral is exhausted
    return entryPrice * (1 - collateralRatio + maintenanceMargin);
  } else {
    // Short: price rises until collateral is exhausted
    return entryPrice * (1 + collateralRatio - maintenanceMargin);
  }
}

/**
 * Calculate leverage for a position
 * @param position - Position data
 * @returns Leverage as number
 */
export function calculateLeverage(position: Position): number {
  const collateral = fromContractAmount(position.collateral);
  const size = fromContractAmount(position.size);
  return size / collateral;
}

/**
 * Get user's token balance
 * @param userAddress - User's Stellar address
 * @returns Token balance as bigint
 */
export async function getUserTokenBalance(userAddress: string): Promise<bigint> {
  const client = getTokenClient();
  const tx = await client.balance({ addr: userAddress });
  const result = await tx.simulate();
  return result.result;
}

// ============================================================================
// Order Functions
// ============================================================================

/**
 * Get all order IDs for a user
 * @param userAddress - User's Stellar address
 * @returns Array of order IDs
 */
export async function getUserOrderIds(userAddress: string): Promise<bigint[]> {
  const client = getPositionManagerClient();
  const tx = await client.get_user_orders({ trader: userAddress });
  const result = await tx.simulate();
  return result.result;
}

/**
 * Get order details by ID
 * @param orderId - Order ID
 * @returns Order data
 */
export async function getOrder(orderId: bigint): Promise<Order> {
  const client = getPositionManagerClient();
  const tx = await client.get_order({ order_id: orderId as u64 });
  const result = await tx.simulate();
  return result.result;
}

/**
 * Get all orders for a user (with full details)
 * @param userAddress - User's Stellar address
 * @returns Array of orders
 */
export async function getUserOrders(userAddress: string): Promise<Order[]> {
  const orderIds = await getUserOrderIds(userAddress);

  const orders = await Promise.all(
    orderIds.map((id) => getOrder(id))
  );

  return orders;
}

/**
 * Get all orders attached to a position (SL/TP)
 * @param positionId - Position ID
 * @returns Array of order IDs
 */
export async function getPositionOrderIds(positionId: bigint): Promise<bigint[]> {
  const client = getPositionManagerClient();
  const tx = await client.get_position_orders({ position_id: positionId as u64 });
  const result = await tx.simulate();
  return result.result;
}

/**
 * Get all orders attached to a position (with full details)
 * @param positionId - Position ID
 * @returns Array of orders
 */
export async function getPositionOrders(positionId: bigint): Promise<Order[]> {
  const orderIds = await getPositionOrderIds(positionId);

  const orders = await Promise.all(
    orderIds.map((id) => getOrder(id))
  );

  return orders;
}

/**
 * Get minimum execution fee required for orders
 * @returns Minimum fee in contract units
 */
export async function getMinExecutionFee(): Promise<bigint> {
  const client = getPositionManagerClient();
  const tx = await client.min_execution_fee();
  const result = await tx.simulate();
  return result.result;
}

/**
 * Create a limit order transaction
 * @param userAddress - User's Stellar address
 * @param marketId - Market identifier (0=XLM, 1=BTC, 2=ETH)
 * @param triggerPrice - Price at which to execute (in contract units)
 * @param acceptablePrice - Max slippage from trigger (0 = any price)
 * @param collateral - Collateral for the new position (in contract units)
 * @param leverage - Leverage for the new position
 * @param isLong - True for long, false for short
 * @param executionFee - Fee to pay keeper (in contract units)
 * @param expiration - Timestamp when order expires (0 = no expiry)
 * @returns AssembledTransaction ready for signing
 */
export async function createLimitOrderTransaction(
  userAddress: string,
  marketId: MarketId,
  triggerPrice: bigint,
  acceptablePrice: bigint,
  collateral: bigint,
  leverage: number,
  isLong: boolean,
  executionFee: bigint,
  expiration: bigint
) {
  const client = getPositionManagerClient(userAddress);
  return await client.create_limit_order({
    trader: userAddress,
    market_id: marketId as u32,
    trigger_price: triggerPrice as i128,
    acceptable_price: acceptablePrice as i128,
    collateral: collateral as u128,
    leverage: leverage as u32,
    is_long: isLong,
    execution_fee: executionFee as u128,
    expiration: expiration as u64,
  });
}

/**
 * Create a stop-loss order transaction
 * @param userAddress - User's Stellar address
 * @param positionId - Position to protect
 * @param triggerPrice - Price at which to close (in contract units)
 * @param acceptablePrice - Min acceptable price for closure (0 = any)
 * @param closePercentage - Percentage to close (10000 = 100%)
 * @param executionFee - Fee to pay keeper (in contract units)
 * @param expiration - Order expiration (0 = no expiry)
 * @returns AssembledTransaction ready for signing
 */
export async function createStopLossTransaction(
  userAddress: string,
  positionId: bigint,
  triggerPrice: bigint,
  acceptablePrice: bigint,
  closePercentage: number,
  executionFee: bigint,
  expiration: bigint
) {
  const client = getPositionManagerClient(userAddress);
  return await client.create_stop_loss({
    trader: userAddress,
    position_id: positionId as u64,
    trigger_price: triggerPrice as i128,
    acceptable_price: acceptablePrice as i128,
    close_percentage: closePercentage as u32,
    execution_fee: executionFee as u128,
    expiration: expiration as u64,
  });
}

/**
 * Create a take-profit order transaction
 * @param userAddress - User's Stellar address
 * @param positionId - Position to take profit from
 * @param triggerPrice - Price at which to close (in contract units)
 * @param acceptablePrice - Min acceptable price for closure (0 = any)
 * @param closePercentage - Percentage to close (10000 = 100%)
 * @param executionFee - Fee to pay keeper (in contract units)
 * @param expiration - Order expiration (0 = no expiry)
 * @returns AssembledTransaction ready for signing
 */
export async function createTakeProfitTransaction(
  userAddress: string,
  positionId: bigint,
  triggerPrice: bigint,
  acceptablePrice: bigint,
  closePercentage: number,
  executionFee: bigint,
  expiration: bigint
) {
  const client = getPositionManagerClient(userAddress);
  return await client.create_take_profit({
    trader: userAddress,
    position_id: positionId as u64,
    trigger_price: triggerPrice as i128,
    acceptable_price: acceptablePrice as i128,
    close_percentage: closePercentage as u32,
    execution_fee: executionFee as u128,
    expiration: expiration as u64,
  });
}

/**
 * Create a cancel order transaction
 * @param userAddress - User's Stellar address
 * @param orderId - Order to cancel
 * @returns AssembledTransaction ready for signing
 */
export async function createCancelOrderTransaction(
  userAddress: string,
  orderId: bigint
) {
  const client = getPositionManagerClient(userAddress);
  return await client.cancel_order({
    trader: userAddress,
    order_id: orderId as u64,
  });
}

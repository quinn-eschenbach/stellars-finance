import { Buffer } from "buffer";
import { Address } from '@stellar/stellar-sdk';
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from '@stellar/stellar-sdk/contract';
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Typepoint,
  Duration,
} from '@stellar/stellar-sdk/contract';
export * from '@stellar/stellar-sdk'
export * as contract from '@stellar/stellar-sdk/contract'
export * as rpc from '@stellar/stellar-sdk/rpc'

if (typeof window !== 'undefined') {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}





export interface Market {
  base_funding_rate: i128;
  cumulative_funding_long: i128;
  cumulative_funding_short: i128;
  funding_rate: i128;
  is_paused: boolean;
  last_funding_update: u64;
  long_open_interest: u128;
  market_id: u32;
  max_funding_rate: i128;
  max_open_interest: u128;
  short_open_interest: u128;
  ticker: string;
}

export type DataKey = {tag: "ConfigManager", values: void} | {tag: "Admin", values: void} | {tag: "Market", values: readonly [u32]} | {tag: "MarketCount", values: void} | {tag: "AuthorizedPositionManager", values: void};




export interface Client {
  /**
   * Construct and simulate a initialize transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Initialize the MarketManager contract.
   * 
   * # Arguments
   * 
   * * `config_manager` - Address of the ConfigManager contract
   * * `admin` - Address of the admin
   */
  initialize: ({config_manager, admin}: {config_manager: string, admin: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a pause_market transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Pause a market to prevent new positions from being opened.
   * 
   * # Arguments
   * 
   * * `admin` - Address of the admin
   * * `market_id` - The market identifier
   */
  pause_market: ({admin, market_id}: {admin: string, market_id: u32}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a create_market transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Create a new perpetual market.
   * 
   * # Arguments
   * 
   * * `admin` - Address of the admin
   * * `market_id` - Unique identifier for the market (e.g., 0 = XLM-PERP)
   * * `ticker` - Oracle ticker symbol (e.g., "XLM", "BTC", "ETH")
   * * `max_open_interest` - Maximum total open interest allowed for this market
   * * `max_funding_rate` - Maximum funding rate per hour (in basis points)
   */
  create_market: ({admin, market_id, ticker, max_open_interest, max_funding_rate}: {admin: string, market_id: u32, ticker: string, max_open_interest: u128, max_funding_rate: i128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a unpause_market transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Unpause a market to allow new positions.
   * 
   * # Arguments
   * 
   * * `admin` - Address of the admin
   * * `market_id` - The market identifier
   */
  unpause_market: ({admin, market_id}: {admin: string, market_id: u32}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_funding_rate transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Get the current funding rate for a market.
   * 
   * # Arguments
   * 
   * * `market_id` - The market identifier
   * 
   * # Returns
   * 
   * The current funding rate (in basis points per hour)
   */
  get_funding_rate: ({market_id}: {market_id: u32}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a is_market_paused transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Check if a market is currently paused.
   * 
   * # Arguments
   * 
   * * `market_id` - The market identifier
   * 
   * # Returns
   * 
   * True if market is paused, false otherwise
   */
  is_market_paused: ({market_id}: {market_id: u32}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a can_open_position transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Check if a new position can be opened based on OI limits.
   * 
   * # Arguments
   * 
   * * `market_id` - The market identifier
   * * `is_long` - True if long position, false if short
   * * `size` - The size of the position to open
   * 
   * # Returns
   * 
   * True if position can be opened, false otherwise
   */
  can_open_position: ({market_id, is_long, size}: {market_id: u32, is_long: boolean, size: u128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a get_market_ticker transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Get the ticker symbol for a market.
   * 
   * # Arguments
   * 
   * * `market_id` - The market identifier
   * 
   * # Returns
   * 
   * The ticker symbol (e.g., "XLM", "BTC", "ETH")
   */
  get_market_ticker: ({market_id}: {market_id: u32}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a get_open_interest transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Get the current open interest for a market.
   * 
   * # Arguments
   * 
   * * `market_id` - The market identifier
   * 
   * # Returns
   * 
   * Tuple of (long_open_interest, short_open_interest)
   */
  get_open_interest: ({market_id}: {market_id: u32}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<readonly [u128, u128]>>

  /**
   * Construct and simulate a update_funding_rate transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Update the funding rate for a market.
   * 
   * Called every 60 seconds by the keeper bot.
   * Calculates funding rate based on market imbalance and updates cumulative funding.
   * Funding rate is expressed in basis points per hour.
   * 
   * # Arguments
   * 
   * * `caller` - Address calling this function
   * * `market_id` - The market identifier
   */
  update_funding_rate: ({caller, market_id}: {caller: string, market_id: u32}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a set_position_manager transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Set the authorized PositionManager contract.
   * 
   * # Arguments
   * 
   * * `admin` - Address of the admin
   * * `position_manager` - Address of the PositionManager contract
   */
  set_position_manager: ({admin, position_manager}: {admin: string, position_manager: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a update_open_interest transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Update open interest when positions are opened or closed.
   * 
   * # Arguments
   * 
   * * `position_manager` - Address of the PositionManager contract
   * * `market_id` - The market identifier
   * * `is_long` - True if long position, false if short
   * * `size_delta` - Change in position size (positive = increase, negative = decrease)
   */
  update_open_interest: ({position_manager, market_id, is_long, size_delta}: {position_manager: string, market_id: u32, is_long: boolean, size_delta: i128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_cumulative_funding transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Get cumulative funding for a position side.
   * 
   * # Arguments
   * 
   * * `market_id` - The market identifier
   * * `is_long` - True if long position, false if short
   * 
   * # Returns
   * 
   * The cumulative funding paid by the specified side
   */
  get_cumulative_funding: ({market_id, is_long}: {market_id: u32, is_long: boolean}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<i128>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy(null, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAQAAAAAAAAAAAAAABk1hcmtldAAAAAAADAAAAAAAAAARYmFzZV9mdW5kaW5nX3JhdGUAAAAAAAALAAAAAAAAABdjdW11bGF0aXZlX2Z1bmRpbmdfbG9uZwAAAAALAAAAAAAAABhjdW11bGF0aXZlX2Z1bmRpbmdfc2hvcnQAAAALAAAAAAAAAAxmdW5kaW5nX3JhdGUAAAALAAAAAAAAAAlpc19wYXVzZWQAAAAAAAABAAAAAAAAABNsYXN0X2Z1bmRpbmdfdXBkYXRlAAAAAAYAAAAAAAAAEmxvbmdfb3Blbl9pbnRlcmVzdAAAAAAACgAAAAAAAAAJbWFya2V0X2lkAAAAAAAABAAAAAAAAAAQbWF4X2Z1bmRpbmdfcmF0ZQAAAAsAAAAAAAAAEW1heF9vcGVuX2ludGVyZXN0AAAAAAAACgAAAAAAAAATc2hvcnRfb3Blbl9pbnRlcmVzdAAAAAAKAAAAAAAAAAZ0aWNrZXIAAAAAABE=",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAABQAAAAAAAAAAAAAADUNvbmZpZ01hbmFnZXIAAAAAAAAAAAAAAAAAAAVBZG1pbgAAAAAAAAEAAAAAAAAABk1hcmtldAAAAAAAAQAAAAQAAAAAAAAAAAAAAAtNYXJrZXRDb3VudAAAAAAAAAAAAAAAABlBdXRob3JpemVkUG9zaXRpb25NYW5hZ2VyAAAA",
        "AAAABQAAAAAAAAAAAAAADk9JVXBkYXRlZEV2ZW50AAAAAAABAAAAEG9pX3VwZGF0ZWRfZXZlbnQAAAADAAAAAAAAAAltYXJrZXRfaWQAAAAAAAAEAAAAAAAAAAAAAAAHbG9uZ19vaQAAAAAKAAAAAAAAAAAAAAAIc2hvcnRfb2kAAAAKAAAAAAAAAAI=",
        "AAAAAAAAAJBJbml0aWFsaXplIHRoZSBNYXJrZXRNYW5hZ2VyIGNvbnRyYWN0LgoKIyBBcmd1bWVudHMKCiogYGNvbmZpZ19tYW5hZ2VyYCAtIEFkZHJlc3Mgb2YgdGhlIENvbmZpZ01hbmFnZXIgY29udHJhY3QKKiBgYWRtaW5gIC0gQWRkcmVzcyBvZiB0aGUgYWRtaW4AAAAKaW5pdGlhbGl6ZQAAAAAAAgAAAAAAAAAOY29uZmlnX21hbmFnZXIAAAAAABMAAAAAAAAABWFkbWluAAAAAAAAEwAAAAA=",
        "AAAAAAAAAI9QYXVzZSBhIG1hcmtldCB0byBwcmV2ZW50IG5ldyBwb3NpdGlvbnMgZnJvbSBiZWluZyBvcGVuZWQuCgojIEFyZ3VtZW50cwoKKiBgYWRtaW5gIC0gQWRkcmVzcyBvZiB0aGUgYWRtaW4KKiBgbWFya2V0X2lkYCAtIFRoZSBtYXJrZXQgaWRlbnRpZmllcgAAAAAMcGF1c2VfbWFya2V0AAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAltYXJrZXRfaWQAAAAAAAAEAAAAAA==",
        "AAAABQAAAAAAAAAAAAAAEk1hcmtldENyZWF0ZWRFdmVudAAAAAAAAQAAABRtYXJrZXRfY3JlYXRlZF9ldmVudAAAAAIAAAAAAAAACW1hcmtldF9pZAAAAAAAAAQAAAAAAAAAAAAAAAZtYXhfb2kAAAAAAAoAAAAAAAAAAg==",
        "AAAAAAAAAWRDcmVhdGUgYSBuZXcgcGVycGV0dWFsIG1hcmtldC4KCiMgQXJndW1lbnRzCgoqIGBhZG1pbmAgLSBBZGRyZXNzIG9mIHRoZSBhZG1pbgoqIGBtYXJrZXRfaWRgIC0gVW5pcXVlIGlkZW50aWZpZXIgZm9yIHRoZSBtYXJrZXQgKGUuZy4sIDAgPSBYTE0tUEVSUCkKKiBgdGlja2VyYCAtIE9yYWNsZSB0aWNrZXIgc3ltYm9sIChlLmcuLCAiWExNIiwgIkJUQyIsICJFVEgiKQoqIGBtYXhfb3Blbl9pbnRlcmVzdGAgLSBNYXhpbXVtIHRvdGFsIG9wZW4gaW50ZXJlc3QgYWxsb3dlZCBmb3IgdGhpcyBtYXJrZXQKKiBgbWF4X2Z1bmRpbmdfcmF0ZWAgLSBNYXhpbXVtIGZ1bmRpbmcgcmF0ZSBwZXIgaG91ciAoaW4gYmFzaXMgcG9pbnRzKQAAAA1jcmVhdGVfbWFya2V0AAAAAAAABQAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAltYXJrZXRfaWQAAAAAAAAEAAAAAAAAAAZ0aWNrZXIAAAAAABEAAAAAAAAAEW1heF9vcGVuX2ludGVyZXN0AAAAAAAACgAAAAAAAAAQbWF4X2Z1bmRpbmdfcmF0ZQAAAAsAAAAA",
        "AAAAAAAAAH1VbnBhdXNlIGEgbWFya2V0IHRvIGFsbG93IG5ldyBwb3NpdGlvbnMuCgojIEFyZ3VtZW50cwoKKiBgYWRtaW5gIC0gQWRkcmVzcyBvZiB0aGUgYWRtaW4KKiBgbWFya2V0X2lkYCAtIFRoZSBtYXJrZXQgaWRlbnRpZmllcgAAAAAAAA51bnBhdXNlX21hcmtldAAAAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAltYXJrZXRfaWQAAAAAAAAEAAAAAA==",
        "AAAAAAAAAJ5HZXQgdGhlIGN1cnJlbnQgZnVuZGluZyByYXRlIGZvciBhIG1hcmtldC4KCiMgQXJndW1lbnRzCgoqIGBtYXJrZXRfaWRgIC0gVGhlIG1hcmtldCBpZGVudGlmaWVyCgojIFJldHVybnMKClRoZSBjdXJyZW50IGZ1bmRpbmcgcmF0ZSAoaW4gYmFzaXMgcG9pbnRzIHBlciBob3VyKQAAAAAAEGdldF9mdW5kaW5nX3JhdGUAAAABAAAAAAAAAAltYXJrZXRfaWQAAAAAAAAEAAAAAQAAAAs=",
        "AAAAAAAAAJBDaGVjayBpZiBhIG1hcmtldCBpcyBjdXJyZW50bHkgcGF1c2VkLgoKIyBBcmd1bWVudHMKCiogYG1hcmtldF9pZGAgLSBUaGUgbWFya2V0IGlkZW50aWZpZXIKCiMgUmV0dXJucwoKVHJ1ZSBpZiBtYXJrZXQgaXMgcGF1c2VkLCBmYWxzZSBvdGhlcndpc2UAAAAQaXNfbWFya2V0X3BhdXNlZAAAAAEAAAAAAAAACW1hcmtldF9pZAAAAAAAAAQAAAABAAAAAQ==",
        "AAAAAAAAAQlDaGVjayBpZiBhIG5ldyBwb3NpdGlvbiBjYW4gYmUgb3BlbmVkIGJhc2VkIG9uIE9JIGxpbWl0cy4KCiMgQXJndW1lbnRzCgoqIGBtYXJrZXRfaWRgIC0gVGhlIG1hcmtldCBpZGVudGlmaWVyCiogYGlzX2xvbmdgIC0gVHJ1ZSBpZiBsb25nIHBvc2l0aW9uLCBmYWxzZSBpZiBzaG9ydAoqIGBzaXplYCAtIFRoZSBzaXplIG9mIHRoZSBwb3NpdGlvbiB0byBvcGVuCgojIFJldHVybnMKClRydWUgaWYgcG9zaXRpb24gY2FuIGJlIG9wZW5lZCwgZmFsc2Ugb3RoZXJ3aXNlAAAAAAAAEWNhbl9vcGVuX3Bvc2l0aW9uAAAAAAAAAwAAAAAAAAAJbWFya2V0X2lkAAAAAAAABAAAAAAAAAAHaXNfbG9uZwAAAAABAAAAAAAAAARzaXplAAAACgAAAAEAAAAB",
        "AAAAAAAAAJFHZXQgdGhlIHRpY2tlciBzeW1ib2wgZm9yIGEgbWFya2V0LgoKIyBBcmd1bWVudHMKCiogYG1hcmtldF9pZGAgLSBUaGUgbWFya2V0IGlkZW50aWZpZXIKCiMgUmV0dXJucwoKVGhlIHRpY2tlciBzeW1ib2wgKGUuZy4sICJYTE0iLCAiQlRDIiwgIkVUSCIpAAAAAAAAEWdldF9tYXJrZXRfdGlja2VyAAAAAAAAAQAAAAAAAAAJbWFya2V0X2lkAAAAAAAABAAAAAEAAAAR",
        "AAAAAAAAAJ5HZXQgdGhlIGN1cnJlbnQgb3BlbiBpbnRlcmVzdCBmb3IgYSBtYXJrZXQuCgojIEFyZ3VtZW50cwoKKiBgbWFya2V0X2lkYCAtIFRoZSBtYXJrZXQgaWRlbnRpZmllcgoKIyBSZXR1cm5zCgpUdXBsZSBvZiAobG9uZ19vcGVuX2ludGVyZXN0LCBzaG9ydF9vcGVuX2ludGVyZXN0KQAAAAAAEWdldF9vcGVuX2ludGVyZXN0AAAAAAAAAQAAAAAAAAAJbWFya2V0X2lkAAAAAAAABAAAAAEAAAPtAAAAAgAAAAoAAAAK",
        "AAAABQAAAAAAAAAAAAAAF0Z1bmRpbmdSYXRlVXBkYXRlZEV2ZW50AAAAAAEAAAAaZnVuZGluZ19yYXRlX3VwZGF0ZWRfZXZlbnQAAAAAAAQAAAAAAAAACW1hcmtldF9pZAAAAAAAAAQAAAAAAAAAAAAAAAxmdW5kaW5nX3JhdGUAAAALAAAAAAAAAAAAAAAHbG9uZ19vaQAAAAAKAAAAAAAAAAAAAAAIc2hvcnRfb2kAAAAKAAAAAAAAAAI=",
        "AAAAAAAAATZVcGRhdGUgdGhlIGZ1bmRpbmcgcmF0ZSBmb3IgYSBtYXJrZXQuCgpDYWxsZWQgZXZlcnkgNjAgc2Vjb25kcyBieSB0aGUga2VlcGVyIGJvdC4KQ2FsY3VsYXRlcyBmdW5kaW5nIHJhdGUgYmFzZWQgb24gbWFya2V0IGltYmFsYW5jZSBhbmQgdXBkYXRlcyBjdW11bGF0aXZlIGZ1bmRpbmcuCkZ1bmRpbmcgcmF0ZSBpcyBleHByZXNzZWQgaW4gYmFzaXMgcG9pbnRzIHBlciBob3VyLgoKIyBBcmd1bWVudHMKCiogYGNhbGxlcmAgLSBBZGRyZXNzIGNhbGxpbmcgdGhpcyBmdW5jdGlvbgoqIGBtYXJrZXRfaWRgIC0gVGhlIG1hcmtldCBpZGVudGlmaWVyAAAAAAATdXBkYXRlX2Z1bmRpbmdfcmF0ZQAAAAACAAAAAAAAAAZjYWxsZXIAAAAAABMAAAAAAAAACW1hcmtldF9pZAAAAAAAAAQAAAAA",
        "AAAAAAAAAJpTZXQgdGhlIGF1dGhvcml6ZWQgUG9zaXRpb25NYW5hZ2VyIGNvbnRyYWN0LgoKIyBBcmd1bWVudHMKCiogYGFkbWluYCAtIEFkZHJlc3Mgb2YgdGhlIGFkbWluCiogYHBvc2l0aW9uX21hbmFnZXJgIC0gQWRkcmVzcyBvZiB0aGUgUG9zaXRpb25NYW5hZ2VyIGNvbnRyYWN0AAAAAAAUc2V0X3Bvc2l0aW9uX21hbmFnZXIAAAACAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAAEHBvc2l0aW9uX21hbmFnZXIAAAATAAAAAA==",
        "AAAAAAAAATRVcGRhdGUgb3BlbiBpbnRlcmVzdCB3aGVuIHBvc2l0aW9ucyBhcmUgb3BlbmVkIG9yIGNsb3NlZC4KCiMgQXJndW1lbnRzCgoqIGBwb3NpdGlvbl9tYW5hZ2VyYCAtIEFkZHJlc3Mgb2YgdGhlIFBvc2l0aW9uTWFuYWdlciBjb250cmFjdAoqIGBtYXJrZXRfaWRgIC0gVGhlIG1hcmtldCBpZGVudGlmaWVyCiogYGlzX2xvbmdgIC0gVHJ1ZSBpZiBsb25nIHBvc2l0aW9uLCBmYWxzZSBpZiBzaG9ydAoqIGBzaXplX2RlbHRhYCAtIENoYW5nZSBpbiBwb3NpdGlvbiBzaXplIChwb3NpdGl2ZSA9IGluY3JlYXNlLCBuZWdhdGl2ZSA9IGRlY3JlYXNlKQAAABR1cGRhdGVfb3Blbl9pbnRlcmVzdAAAAAQAAAAAAAAAEHBvc2l0aW9uX21hbmFnZXIAAAATAAAAAAAAAAltYXJrZXRfaWQAAAAAAAAEAAAAAAAAAAdpc19sb25nAAAAAAEAAAAAAAAACnNpemVfZGVsdGEAAAAAAAsAAAAA",
        "AAAAAAAAANFHZXQgY3VtdWxhdGl2ZSBmdW5kaW5nIGZvciBhIHBvc2l0aW9uIHNpZGUuCgojIEFyZ3VtZW50cwoKKiBgbWFya2V0X2lkYCAtIFRoZSBtYXJrZXQgaWRlbnRpZmllcgoqIGBpc19sb25nYCAtIFRydWUgaWYgbG9uZyBwb3NpdGlvbiwgZmFsc2UgaWYgc2hvcnQKCiMgUmV0dXJucwoKVGhlIGN1bXVsYXRpdmUgZnVuZGluZyBwYWlkIGJ5IHRoZSBzcGVjaWZpZWQgc2lkZQAAAAAAABZnZXRfY3VtdWxhdGl2ZV9mdW5kaW5nAAAAAAACAAAAAAAAAAltYXJrZXRfaWQAAAAAAAAEAAAAAAAAAAdpc19sb25nAAAAAAEAAAABAAAACw==" ]),
      options
    )
  }
  public readonly fromJSON = {
    initialize: this.txFromJSON<null>,
        pause_market: this.txFromJSON<null>,
        create_market: this.txFromJSON<null>,
        unpause_market: this.txFromJSON<null>,
        get_funding_rate: this.txFromJSON<i128>,
        is_market_paused: this.txFromJSON<boolean>,
        can_open_position: this.txFromJSON<boolean>,
        get_market_ticker: this.txFromJSON<string>,
        get_open_interest: this.txFromJSON<readonly [u128, u128]>,
        update_funding_rate: this.txFromJSON<null>,
        set_position_manager: this.txFromJSON<null>,
        update_open_interest: this.txFromJSON<null>,
        get_cumulative_funding: this.txFromJSON<i128>
  }
}
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




export type DataKey = {tag: "ConfigManager", values: void} | {tag: "TestMode", values: void} | {tag: "TestBasePrice", values: readonly [u32]} | {tag: "FixedPriceMode", values: void};

export interface Client {
  /**
   * Construct and simulate a get_price transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Get the current price for a specific asset from Reflector oracle.
   * 
   * # Arguments
   * 
   * * `market_id` - The market identifier (0=XLM, 1=BTC, 2=ETH)
   * 
   * # Returns
   * 
   * The price in protocol format (1e7 decimals)
   * 
   * # Implementation
   * 
   * In test mode: Returns time-based simulated price
   * In production mode: Fetches from Reflector oracle, validates, and returns
   */
  get_price: ({market_id}: {market_id: u32}, options?: {
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
   * Construct and simulate a initialize transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Initialize the OracleIntegrator contract.
   * 
   * # Arguments
   * 
   * * `config_manager` - Address of the ConfigManager contract
   */
  initialize: ({config_manager}: {config_manager: string}, options?: {
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
   * Construct and simulate a get_test_mode transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Check if test mode is enabled.
   * 
   * # Returns
   * 
   * True if test mode is enabled, false otherwise
   */
  get_test_mode: (options?: {
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
   * Construct and simulate a set_test_mode transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Enable or disable test mode with base prices.
   * 
   * # Arguments
   * 
   * * `admin` - The administrator address (must match ConfigManager admin)
   * * `enabled` - Whether to enable test mode
   * * `base_prices` - Map of market_id to base price for simulation
   * 
   * # Panics
   * 
   * Panics if caller is not the admin
   */
  set_test_mode: ({admin, enabled, base_prices}: {admin: string, enabled: boolean, base_prices: Map<u32, i128>}, options?: {
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
   * Construct and simulate a validate_price transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Validate a price feed for staleness and bounds.
   * 
   * # Arguments
   * 
   * * `price` - The price to validate
   * * `timestamp` - The price timestamp
   * * `min_price` - Minimum acceptable price
   * * `max_price` - Maximum acceptable price
   * 
   * # Returns
   * 
   * True if price is valid, false otherwise
   */
  validate_price: ({price, timestamp, min_price, max_price}: {price: i128, timestamp: u64, min_price: i128, max_price: i128}, options?: {
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
   * Construct and simulate a calculate_median transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Calculate median price from multiple oracle sources.
   * 
   * # Arguments
   * 
   * * `price1` - Price from first oracle
   * * `price2` - Price from second oracle
   * 
   * # Returns
   * 
   * The median price (average of 2 prices)
   */
  calculate_median: ({price1, price2}: {price1: i128, price2: i128}, options?: {
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
   * Construct and simulate a fetch_pyth_price transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Fetch price from Pyth Network oracle.
   * 
   * # Arguments
   * 
   * * `asset_id` - The asset identifier
   * 
   * # Returns
   * 
   * Tuple of (price, confidence, timestamp)
   */
  fetch_pyth_price: ({asset_id}: {asset_id: u32}, options?: {
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
  }) => Promise<AssembledTransaction<readonly [i128, i128, u64]>>

  /**
   * Construct and simulate a get_oracle_health transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Get the health status of all oracle sources.
   * 
   * # Returns
   * 
   * Tuple of (pyth_healthy, dia_healthy, reflector_healthy)
   */
  get_oracle_health: (options?: {
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
  }) => Promise<AssembledTransaction<readonly [boolean, boolean, boolean]>>

  /**
   * Construct and simulate a update_cached_price transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Update the cached price for an asset.
   * 
   * Called periodically by keeper bots to maintain fresh prices.
   * 
   * # Arguments
   * 
   * * `asset_id` - The asset identifier
   */
  update_cached_price: ({asset_id}: {asset_id: u32}, options?: {
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
   * Construct and simulate a set_fixed_price_mode transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Enable or disable fixed price mode (no oscillation).
   * When enabled, prices will remain at base price without time-based variation.
   * Useful for testing funding rates in isolation.
   * 
   * # Arguments
   * 
   * * `admin` - The administrator address
   * * `enabled` - Whether to enable fixed price mode
   */
  set_fixed_price_mode: ({admin, enabled}: {admin: string, enabled: boolean}, options?: {
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
   * Construct and simulate a check_price_deviation transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Check if price deviation between sources exceeds threshold.
   * 
   * # Arguments
   * 
   * * `prices` - Array of prices from different oracles
   * * `threshold_bps` - Maximum allowed deviation in basis points
   * 
   * # Returns
   * 
   * True if deviation is acceptable, false if excessive
   */
  check_price_deviation: ({threshold_bps}: {threshold_bps: u32}, options?: {
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
   * Construct and simulate a fetch_reflector_price transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Fetch price from Reflector oracle (SEP-40 compliant).
   * 
   * # Arguments
   * 
   * * `market_id` - The market identifier (0=XLM, 1=BTC, 2=ETH)
   * 
   * # Returns
   * 
   * Tuple of (price, timestamp) in protocol format (1e7 decimals)
   */
  fetch_reflector_price: ({market_id}: {market_id: u32}, options?: {
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
  }) => Promise<AssembledTransaction<readonly [i128, u64]>>

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
      new ContractSpec([ "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAABAAAAAAAAAAAAAAADUNvbmZpZ01hbmFnZXIAAAAAAAAAAAAAAAAAAAhUZXN0TW9kZQAAAAEAAAAAAAAADVRlc3RCYXNlUHJpY2UAAAAAAAABAAAABAAAAAAAAAAAAAAADkZpeGVkUHJpY2VNb2RlAAA=",
        "AAAAAAAAAVFHZXQgdGhlIGN1cnJlbnQgcHJpY2UgZm9yIGEgc3BlY2lmaWMgYXNzZXQgZnJvbSBSZWZsZWN0b3Igb3JhY2xlLgoKIyBBcmd1bWVudHMKCiogYG1hcmtldF9pZGAgLSBUaGUgbWFya2V0IGlkZW50aWZpZXIgKDA9WExNLCAxPUJUQywgMj1FVEgpCgojIFJldHVybnMKClRoZSBwcmljZSBpbiBwcm90b2NvbCBmb3JtYXQgKDFlNyBkZWNpbWFscykKCiMgSW1wbGVtZW50YXRpb24KCkluIHRlc3QgbW9kZTogUmV0dXJucyB0aW1lLWJhc2VkIHNpbXVsYXRlZCBwcmljZQpJbiBwcm9kdWN0aW9uIG1vZGU6IEZldGNoZXMgZnJvbSBSZWZsZWN0b3Igb3JhY2xlLCB2YWxpZGF0ZXMsIGFuZCByZXR1cm5zAAAAAAAACWdldF9wcmljZQAAAAAAAAEAAAAAAAAACW1hcmtldF9pZAAAAAAAAAQAAAABAAAACw==",
        "AAAAAAAAAHJJbml0aWFsaXplIHRoZSBPcmFjbGVJbnRlZ3JhdG9yIGNvbnRyYWN0LgoKIyBBcmd1bWVudHMKCiogYGNvbmZpZ19tYW5hZ2VyYCAtIEFkZHJlc3Mgb2YgdGhlIENvbmZpZ01hbmFnZXIgY29udHJhY3QAAAAAAAppbml0aWFsaXplAAAAAAABAAAAAAAAAA5jb25maWdfbWFuYWdlcgAAAAAAEwAAAAA=",
        "AAAAAAAAAFhDaGVjayBpZiB0ZXN0IG1vZGUgaXMgZW5hYmxlZC4KCiMgUmV0dXJucwoKVHJ1ZSBpZiB0ZXN0IG1vZGUgaXMgZW5hYmxlZCwgZmFsc2Ugb3RoZXJ3aXNlAAAADWdldF90ZXN0X21vZGUAAAAAAAAAAAAAAQAAAAE=",
        "AAAAAAAAARlFbmFibGUgb3IgZGlzYWJsZSB0ZXN0IG1vZGUgd2l0aCBiYXNlIHByaWNlcy4KCiMgQXJndW1lbnRzCgoqIGBhZG1pbmAgLSBUaGUgYWRtaW5pc3RyYXRvciBhZGRyZXNzIChtdXN0IG1hdGNoIENvbmZpZ01hbmFnZXIgYWRtaW4pCiogYGVuYWJsZWRgIC0gV2hldGhlciB0byBlbmFibGUgdGVzdCBtb2RlCiogYGJhc2VfcHJpY2VzYCAtIE1hcCBvZiBtYXJrZXRfaWQgdG8gYmFzZSBwcmljZSBmb3Igc2ltdWxhdGlvbgoKIyBQYW5pY3MKClBhbmljcyBpZiBjYWxsZXIgaXMgbm90IHRoZSBhZG1pbgAAAAAAAA1zZXRfdGVzdF9tb2RlAAAAAAAAAwAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAdlbmFibGVkAAAAAAEAAAAAAAAAC2Jhc2VfcHJpY2VzAAAAA+wAAAAEAAAACwAAAAA=",
        "AAAAAAAAAQlWYWxpZGF0ZSBhIHByaWNlIGZlZWQgZm9yIHN0YWxlbmVzcyBhbmQgYm91bmRzLgoKIyBBcmd1bWVudHMKCiogYHByaWNlYCAtIFRoZSBwcmljZSB0byB2YWxpZGF0ZQoqIGB0aW1lc3RhbXBgIC0gVGhlIHByaWNlIHRpbWVzdGFtcAoqIGBtaW5fcHJpY2VgIC0gTWluaW11bSBhY2NlcHRhYmxlIHByaWNlCiogYG1heF9wcmljZWAgLSBNYXhpbXVtIGFjY2VwdGFibGUgcHJpY2UKCiMgUmV0dXJucwoKVHJ1ZSBpZiBwcmljZSBpcyB2YWxpZCwgZmFsc2Ugb3RoZXJ3aXNlAAAAAAAADnZhbGlkYXRlX3ByaWNlAAAAAAAEAAAAAAAAAAVwcmljZQAAAAAAAAsAAAAAAAAACXRpbWVzdGFtcAAAAAAAAAYAAAAAAAAACW1pbl9wcmljZQAAAAAAAAsAAAAAAAAACW1heF9wcmljZQAAAAAAAAsAAAABAAAAAQ==",
        "AAAAAAAAAMBDYWxjdWxhdGUgbWVkaWFuIHByaWNlIGZyb20gbXVsdGlwbGUgb3JhY2xlIHNvdXJjZXMuCgojIEFyZ3VtZW50cwoKKiBgcHJpY2UxYCAtIFByaWNlIGZyb20gZmlyc3Qgb3JhY2xlCiogYHByaWNlMmAgLSBQcmljZSBmcm9tIHNlY29uZCBvcmFjbGUKCiMgUmV0dXJucwoKVGhlIG1lZGlhbiBwcmljZSAoYXZlcmFnZSBvZiAyIHByaWNlcykAAAAQY2FsY3VsYXRlX21lZGlhbgAAAAIAAAAAAAAABnByaWNlMQAAAAAACwAAAAAAAAAGcHJpY2UyAAAAAAALAAAAAQAAAAs=",
        "AAAAAAAAAItGZXRjaCBwcmljZSBmcm9tIFB5dGggTmV0d29yayBvcmFjbGUuCgojIEFyZ3VtZW50cwoKKiBgYXNzZXRfaWRgIC0gVGhlIGFzc2V0IGlkZW50aWZpZXIKCiMgUmV0dXJucwoKVHVwbGUgb2YgKHByaWNlLCBjb25maWRlbmNlLCB0aW1lc3RhbXApAAAAABBmZXRjaF9weXRoX3ByaWNlAAAAAQAAAAAAAAAIYXNzZXRfaWQAAAAEAAAAAQAAA+0AAAADAAAACwAAAAsAAAAG",
        "AAAAAAAAAHBHZXQgdGhlIGhlYWx0aCBzdGF0dXMgb2YgYWxsIG9yYWNsZSBzb3VyY2VzLgoKIyBSZXR1cm5zCgpUdXBsZSBvZiAocHl0aF9oZWFsdGh5LCBkaWFfaGVhbHRoeSwgcmVmbGVjdG9yX2hlYWx0aHkpAAAAEWdldF9vcmFjbGVfaGVhbHRoAAAAAAAAAAAAAAEAAAPtAAAAAwAAAAEAAAABAAAAAQ==",
        "AAAAAAAAAJVVcGRhdGUgdGhlIGNhY2hlZCBwcmljZSBmb3IgYW4gYXNzZXQuCgpDYWxsZWQgcGVyaW9kaWNhbGx5IGJ5IGtlZXBlciBib3RzIHRvIG1haW50YWluIGZyZXNoIHByaWNlcy4KCiMgQXJndW1lbnRzCgoqIGBhc3NldF9pZGAgLSBUaGUgYXNzZXQgaWRlbnRpZmllcgAAAAAAABN1cGRhdGVfY2FjaGVkX3ByaWNlAAAAAAEAAAAAAAAACGFzc2V0X2lkAAAABAAAAAA=",
        "AAAAAAAAARVFbmFibGUgb3IgZGlzYWJsZSBmaXhlZCBwcmljZSBtb2RlIChubyBvc2NpbGxhdGlvbikuCldoZW4gZW5hYmxlZCwgcHJpY2VzIHdpbGwgcmVtYWluIGF0IGJhc2UgcHJpY2Ugd2l0aG91dCB0aW1lLWJhc2VkIHZhcmlhdGlvbi4KVXNlZnVsIGZvciB0ZXN0aW5nIGZ1bmRpbmcgcmF0ZXMgaW4gaXNvbGF0aW9uLgoKIyBBcmd1bWVudHMKCiogYGFkbWluYCAtIFRoZSBhZG1pbmlzdHJhdG9yIGFkZHJlc3MKKiBgZW5hYmxlZGAgLSBXaGV0aGVyIHRvIGVuYWJsZSBmaXhlZCBwcmljZSBtb2RlAAAAAAAAFHNldF9maXhlZF9wcmljZV9tb2RlAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAdlbmFibGVkAAAAAAEAAAAA",
        "AAAAAAAAAPtDaGVjayBpZiBwcmljZSBkZXZpYXRpb24gYmV0d2VlbiBzb3VyY2VzIGV4Y2VlZHMgdGhyZXNob2xkLgoKIyBBcmd1bWVudHMKCiogYHByaWNlc2AgLSBBcnJheSBvZiBwcmljZXMgZnJvbSBkaWZmZXJlbnQgb3JhY2xlcwoqIGB0aHJlc2hvbGRfYnBzYCAtIE1heGltdW0gYWxsb3dlZCBkZXZpYXRpb24gaW4gYmFzaXMgcG9pbnRzCgojIFJldHVybnMKClRydWUgaWYgZGV2aWF0aW9uIGlzIGFjY2VwdGFibGUsIGZhbHNlIGlmIGV4Y2Vzc2l2ZQAAAAAVY2hlY2tfcHJpY2VfZGV2aWF0aW9uAAAAAAAAAQAAAAAAAAANdGhyZXNob2xkX2JwcwAAAAAAAAQAAAABAAAAAQ==",
        "AAAAAAAAAMlGZXRjaCBwcmljZSBmcm9tIFJlZmxlY3RvciBvcmFjbGUgKFNFUC00MCBjb21wbGlhbnQpLgoKIyBBcmd1bWVudHMKCiogYG1hcmtldF9pZGAgLSBUaGUgbWFya2V0IGlkZW50aWZpZXIgKDA9WExNLCAxPUJUQywgMj1FVEgpCgojIFJldHVybnMKClR1cGxlIG9mIChwcmljZSwgdGltZXN0YW1wKSBpbiBwcm90b2NvbCBmb3JtYXQgKDFlNyBkZWNpbWFscykAAAAAAAAVZmV0Y2hfcmVmbGVjdG9yX3ByaWNlAAAAAAAAAQAAAAAAAAAJbWFya2V0X2lkAAAAAAAABAAAAAEAAAPtAAAAAgAAAAsAAAAG" ]),
      options
    )
  }
  public readonly fromJSON = {
    get_price: this.txFromJSON<i128>,
        initialize: this.txFromJSON<null>,
        get_test_mode: this.txFromJSON<boolean>,
        set_test_mode: this.txFromJSON<null>,
        validate_price: this.txFromJSON<boolean>,
        calculate_median: this.txFromJSON<i128>,
        fetch_pyth_price: this.txFromJSON<readonly [i128, i128, u64]>,
        get_oracle_health: this.txFromJSON<readonly [boolean, boolean, boolean]>,
        update_cached_price: this.txFromJSON<null>,
        set_fixed_price_mode: this.txFromJSON<null>,
        check_price_deviation: this.txFromJSON<boolean>,
        fetch_reflector_price: this.txFromJSON<readonly [i128, u64]>
  }
}
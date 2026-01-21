import { Buffer } from "buffer";
import { AssembledTransaction, Client as ContractClient, ClientOptions as ContractClientOptions, MethodOptions } from '@stellar/stellar-sdk/contract';
import type { u32, u64, i128 } from '@stellar/stellar-sdk/contract';
export * from '@stellar/stellar-sdk';
export * as contract from '@stellar/stellar-sdk/contract';
export * as rpc from '@stellar/stellar-sdk/rpc';
export type DataKey = {
    tag: "ConfigManager";
    values: void;
} | {
    tag: "TestMode";
    values: void;
} | {
    tag: "TestBasePrice";
    values: readonly [u32];
} | {
    tag: "FixedPriceMode";
    values: void;
};
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
    get_price: ({ market_id }: {
        market_id: u32;
    }, options?: {
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
    }) => Promise<AssembledTransaction<i128>>;
    /**
     * Construct and simulate a initialize transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
     * Initialize the OracleIntegrator contract.
     *
     * # Arguments
     *
     * * `config_manager` - Address of the ConfigManager contract
     */
    initialize: ({ config_manager }: {
        config_manager: string;
    }, options?: {
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
    }) => Promise<AssembledTransaction<null>>;
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
    }) => Promise<AssembledTransaction<boolean>>;
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
    set_test_mode: ({ admin, enabled, base_prices }: {
        admin: string;
        enabled: boolean;
        base_prices: Map<u32, i128>;
    }, options?: {
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
    }) => Promise<AssembledTransaction<null>>;
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
    validate_price: ({ price, timestamp, min_price, max_price }: {
        price: i128;
        timestamp: u64;
        min_price: i128;
        max_price: i128;
    }, options?: {
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
    }) => Promise<AssembledTransaction<boolean>>;
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
    calculate_median: ({ price1, price2 }: {
        price1: i128;
        price2: i128;
    }, options?: {
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
    }) => Promise<AssembledTransaction<i128>>;
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
    fetch_pyth_price: ({ asset_id }: {
        asset_id: u32;
    }, options?: {
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
    }) => Promise<AssembledTransaction<readonly [i128, i128, u64]>>;
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
    }) => Promise<AssembledTransaction<readonly [boolean, boolean, boolean]>>;
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
    update_cached_price: ({ asset_id }: {
        asset_id: u32;
    }, options?: {
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
    }) => Promise<AssembledTransaction<null>>;
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
    set_fixed_price_mode: ({ admin, enabled }: {
        admin: string;
        enabled: boolean;
    }, options?: {
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
    }) => Promise<AssembledTransaction<null>>;
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
    check_price_deviation: ({ threshold_bps }: {
        threshold_bps: u32;
    }, options?: {
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
    }) => Promise<AssembledTransaction<boolean>>;
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
    fetch_reflector_price: ({ market_id }: {
        market_id: u32;
    }, options?: {
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
    }) => Promise<AssembledTransaction<readonly [i128, u64]>>;
}
export declare class Client extends ContractClient {
    readonly options: ContractClientOptions;
    static deploy<T = Client>(
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions & Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
    }): Promise<AssembledTransaction<T>>;
    constructor(options: ContractClientOptions);
    readonly fromJSON: {
        get_price: (json: string) => AssembledTransaction<bigint>;
        initialize: (json: string) => AssembledTransaction<null>;
        get_test_mode: (json: string) => AssembledTransaction<boolean>;
        set_test_mode: (json: string) => AssembledTransaction<null>;
        validate_price: (json: string) => AssembledTransaction<boolean>;
        calculate_median: (json: string) => AssembledTransaction<bigint>;
        fetch_pyth_price: (json: string) => AssembledTransaction<readonly [bigint, bigint, bigint]>;
        get_oracle_health: (json: string) => AssembledTransaction<readonly [boolean, boolean, boolean]>;
        update_cached_price: (json: string) => AssembledTransaction<null>;
        set_fixed_price_mode: (json: string) => AssembledTransaction<null>;
        check_price_deviation: (json: string) => AssembledTransaction<boolean>;
        fetch_reflector_price: (json: string) => AssembledTransaction<readonly [bigint, bigint]>;
    };
}

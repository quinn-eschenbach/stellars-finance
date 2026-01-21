import { Buffer } from "buffer";
import { Client as ContractClient, Spec as ContractSpec, } from '@stellar/stellar-sdk/contract';
export * from '@stellar/stellar-sdk';
export * as contract from '@stellar/stellar-sdk/contract';
export * as rpc from '@stellar/stellar-sdk/rpc';
if (typeof window !== 'undefined') {
    //@ts-ignore Buffer exists
    window.Buffer = window.Buffer || Buffer;
}
export class Client extends ContractClient {
    options;
    static async deploy(
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options) {
        return ContractClient.deploy(null, options);
    }
    constructor(options) {
        super(new ContractSpec(["AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAABAAAAAAAAAAAAAAADUNvbmZpZ01hbmFnZXIAAAAAAAAAAAAAAAAAAAhUZXN0TW9kZQAAAAEAAAAAAAAADVRlc3RCYXNlUHJpY2UAAAAAAAABAAAABAAAAAAAAAAAAAAADkZpeGVkUHJpY2VNb2RlAAA=",
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
            "AAAAAAAAAMlGZXRjaCBwcmljZSBmcm9tIFJlZmxlY3RvciBvcmFjbGUgKFNFUC00MCBjb21wbGlhbnQpLgoKIyBBcmd1bWVudHMKCiogYG1hcmtldF9pZGAgLSBUaGUgbWFya2V0IGlkZW50aWZpZXIgKDA9WExNLCAxPUJUQywgMj1FVEgpCgojIFJldHVybnMKClR1cGxlIG9mIChwcmljZSwgdGltZXN0YW1wKSBpbiBwcm90b2NvbCBmb3JtYXQgKDFlNyBkZWNpbWFscykAAAAAAAAVZmV0Y2hfcmVmbGVjdG9yX3ByaWNlAAAAAAAAAQAAAAAAAAAJbWFya2V0X2lkAAAAAAAABAAAAAEAAAPtAAAAAgAAAAsAAAAG"]), options);
        this.options = options;
    }
    fromJSON = {
        get_price: (this.txFromJSON),
        initialize: (this.txFromJSON),
        get_test_mode: (this.txFromJSON),
        set_test_mode: (this.txFromJSON),
        validate_price: (this.txFromJSON),
        calculate_median: (this.txFromJSON),
        fetch_pyth_price: (this.txFromJSON),
        get_oracle_health: (this.txFromJSON),
        update_cached_price: (this.txFromJSON),
        set_fixed_price_mode: (this.txFromJSON),
        check_price_deviation: (this.txFromJSON),
        fetch_reflector_price: (this.txFromJSON)
    };
}

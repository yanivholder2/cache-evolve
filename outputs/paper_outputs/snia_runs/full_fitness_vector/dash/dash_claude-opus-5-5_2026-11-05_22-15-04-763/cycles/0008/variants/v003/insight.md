# Variant v003 Insight

## Helped

- per-op-time (`get` p50): The median get got faster by about the cost of one SipHash call. This fits the change: `hash_key` is now a single store plus the murmur3 `fmix64` finalizer for `u64` keys, and `promotion_get` takes the stash index, target index and mask bit from one precomputed hash. Cycle 0007 v001 deduplicated hash calls without cheapening them, and that did not move p50. This variant cut the cost of each call, and p50 did move. So the per-call hashing cost was the remaining common-path overhead, not the number of calls.
- per-op-time (`put` p50, p99): Both put percentiles dropped by roughly half, which recovers most of v002's regression. `refresh_stash_mask` is structurally unchanged and still re-hashes every stash item on each removal. Each of those re-hashes is now just a few multiply/xor operations, so the per-removal cost dropped sharply. This independently confirms v002's diagnosis that the put regression came from mask upkeep: making each re-hash cheaper shrank it, with no change to how often re-hashes run.
- per-op-time (`get` p99): The tail fell sharply as well. v002 had blamed its tail on stash-hit promotions paying for a full mask refresh. A cheaper refresh, plus a single hash computation per `promotion_get`, removes most of that slow-path cost.

## Hurt

- hit-rate: No meaningful harm. The per-trace values moved by at most about 1e-4, in both directions (hm_0 at 16K up, hm_0 at 128K down; web_0 unchanged). That is the expected noise from remapping keys to buckets under a new hash and multiply-shift reduction. It is well inside the ±0.001 tolerance, so `fmix64` with high-bit bucket indexing, low-bit stash bits and low-32-bit segment selection does not create load skew on these traces.

(metadata-size did not change: the hasher is a zero-sized `BuildHasherDefault`, so `Item` and `Dash` keep their sizes.)

## Takeaway

On this cache, hashing cost was a first-order term. Swapping SipHash for an `fmix64` finalizer was a policy-neutral change, and it beat the parent on every per-op-time percentile, with no hit-rate cost. Most of v002's put/tail penalty from O(stash) mask refreshes was the cost of the hash calls inside each refresh, not the refresh structure itself. A structural O(1) mask update (counting mask, or metadata kept next to the bucket) would now remove a much smaller remaining cost. It is worth trying only if put still needs to drop below the pre-v002 level.

(Note: I reviewed the overlay's `src/shared/utils.rs`, which contains the hasher, the split hash/reduction and the segment index. I could not list or open the other overlay files in this session, so the claims about how `promotion_get` and `refresh_stash_mask` use the hash rest on the hypothesis's described changes.)

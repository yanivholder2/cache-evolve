## Recurring wins

- **Two-hit gate + MRU demotion is the retention core (0001–0003).** Gating promotion on a second stash hit raised hit rate everywhere. Demoting normal-tier victims to the stash MRU end added more, with or without the gate. Every later line keeps both.
- **"Don't evict a proven resident" pays for hit rate.** Probing overflow and the probing room gate (0002–0004) gained a little everywhere. Overflow also cut demotion cascades, which helped `put` p99 because stash kept draining.
- **hm_0 at the large size is the cell that reacts most to retention changes.** It gains most when retention improves and loses most when it drops (0001–0005). Use it as the first check for any retention change.
- **The stash probe was the `get` p50 gap, and a dense presence mask fixes it (0007).** Skipping the stash scan with a mask test was the first change to lower p50 with identical hit rate.
- **Cheap hashing is a big lever (0008).** Swapping SipHash for an fmix64-style hasher improved every per-op percentile. It also shrank the mask-refresh cost, because every refresh re-hashes.
- **Keep the read-side mask word dense; upkeep can change around it (0008–0009).** O(1) clears through a collision word, and a k=2 Bloom test in the same u64, each won with bit-identical hit rate. False-positive rate is now the main tuning axis for the filter.
- **Reusing existing per-item fields (e.g. `lfu_counter`) keeps metadata size unchanged.** Prefer this for new per-item state.

## Recurring failures

- **Lookup-path tweaks other than the stash probe.** Checking stash last (0003), reversing scan direction (0005), skipping the MRU shift on near-MRU hits (0006) and hashing once per lookup (0007) all left p50 flat or made it worse.
- **Skipping or deferring cascades moves their cost; it does not remove it (0004–0007).** Unbounded deferral jammed stash and made `put` much worse. A single bounded deferral moved the cost to the `get` tail and to small-cache hit rate.
- **Deferral signals taken from stash bucket state cannot separate harmful deferrals from harmless ones.** Fill level hardly ever fires because stash is almost always full (0006). The marked-LRU signal only moved cost between `get` and `put` (0007).
- **Demotion placement based on reuse within one normal-tier stay** (unreused items to stash LRU, 0005) lost hit rate everywhere.
- **Ungated overflow line (0003–0004):** items placed in probing are found last and get evicted, so the `get` p99 cost outweighed what overflow saved on `put`.
- **Read-side mask structures other than one dense word** (a shrunken counting mask, or per-slot fingerprints scanned in the bucket, 0008) lost the p50 gain. The first raised false positives; the second was slower than one AND.
- **Adding work to the stash-hit promotion shift** lands directly on the `get` tail (0008 v002, partly 0006 v003).

## Open questions

- **Combining k=2 with the O(1) clear:** does the fallback rate stay low under k=2 once collisions are tracked per bit? Is k=3 worth it given how full the buckets are?
- **The `get` p99 is still on the stash-hit promotion path.** How much comes from the item shift and how much from the refresh fallback? Can the shift be made cheaper without adding work to it? The fused rotate looked promising in 0006 but needs to be kept to the third-hit path or to non-inlined code.
- **Is there p50 left in the segment-level hash, or in remaining mask false positives?**
- **Stash victim selection:** a two-slot window almost never fired (0009). Does a whole-bucket window with protect-once move hit rate, or is stash eviction a weak lever on these traces?
- **Does the cheap hasher and mask change the trade-offs on the deferral line?** Revisit only with a new kind of signal.
- **Noise:** small p99 moves on untouched paths keep appearing (0005–0009), and every result has one sample. Rerun before trusting small tail changes.

## Per-policy notes

- **Dash (only policy in the ledger).** Current base: 0008 v003 (gate, MRU demotion, probing overflow, probing room gate, fast hasher, dense stash mask) plus the 0009 mask winners (O(1) collision-word clear; k=2 Bloom test).
- **Dash hit rate has plateaued since about 0004.** Recent wins are cost-only. Hit-rate ideas still open: stash victim choice and a better deferral signal.
- **Dash cost model:** hashing and the stash probe drive `get` p50. The stash-hit promotion shift drives `get` p99. Stash churn (removals, mask upkeep, cascades) drives `put`.
- **Small vs large cache in Dash:** longer stash residency helps the large cache (proven residents stay in place) but hurts the small cache (stash slots are scarce).

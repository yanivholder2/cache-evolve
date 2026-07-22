# Cycle 0009 Insight

## Cohort outcome

All three variants attempted to refine the second-chance scan's eviction quality — by adjusting the warm-all fallback index, introducing a two-generation access counter with priority passes, or expiring warm flags on deep-cold displacement — and all three regressed. v003 was selected as least-bad among a uniformly regressing cohort.

## Patterns

- **The base's LRU-among-cold fallback index (`len - 1`) was already correct; the H01 hypothesis had the layout direction inverted.** In this cache's positional model, higher index means less recently used, so `len - 1` is the genuine LRU tail. Changing the fallback to `len - scan_depth` selected the *most recently* inserted item in the cold window — anti-LRU — explaining the hit-rate drop without any algorithmic flaw in the parent.
- **Single-access items carry real reuse probability on a Zipfian workload.** Elevating them as eviction targets (generation-1 priority pass in v002) drained items that would have converted to confirmed-warm items on their next request. The second-chance flag's binary (ever/never) distinction already captures the meaningful split; finer-grained generation counts add classification cost without eviction benefit under this distribution.
- **"Deep-cold position" is not a reliable proxy for "stale warm marker."** On Zipfian access, legitimately warm items accumulate near the cold tail simply through low-but-nonzero access frequency. Expiring flags based on position alone (v003's H03) conflates low frequency with staleness, evicts genuinely reusable items, and introduces an extra conditional store that raises tail latency even on a rarely-firing path.
- **Compact struct omission is still recurring.** v002 silently dropped `item.rs` from its overlay and reverted to the 40-byte baseline layout — the third cycle in a row where this invariant has been violated. Variants that touch only `eviction_policy.rs` must explicitly carry the `item.rs` overlay forward.

## Open questions

- The put p99 regression from Cycle 0006's v003 has never been directly diagnosed; the swap-elimination experiments in Cycle 0007 ruled out write pressure, but the actual cause — metadata layout artifact, scan-fallback behavior, or otherwise — remains open.
- Has the 3-swap + 8-slot + LRU-among-cold + write-once-flag base converged near a local optimum for this workload, or is there a structurally different axis (e.g., adaptive scan depth, insertion-point policy) that hasn't been explored?

## Continuation

The three axes probed in this cycle — fallback-index tuning, generation-counter granularity, and flag expiry on displacement — have all been tested and rejected; the next cycle should not revisit them. The most tractable unresolved thread is the Cycle 0006 put p99 regression: the next hypotheses-writer should design a direct A/B comparison between the current confirmed base and the original 8-slot LIFO configuration on put-heavy access patterns, isolating whether the regression was tied to the LIFO ordering, the compact struct transition, or a scan-fallback edge case. If that diagnosis clears, insertion-point experiments — placing new items somewhere other than the conventional tail position — are a structurally distinct axis worth opening.

# Variant v002 Insight

## Helped

- hit-rate: Requiring a second stash hit before `promotion_get` moves an item into its target bucket means items re-referenced only once (read-after-write, short re-scans) now stay in stash at the MRU end. They no longer evict an established resident from the small normal tier, so the 8 normal buckets per segment keep items with real reuse. Hit rate rose on every trace and size tested. The largest gains came at the larger cache size, on hm_0 most of all, where the protected tier is big enough to hold a stable hot set once one-hit items stop flushing it.

## Hurt

- per-op-time: `get` got slower at p50 and p99, which goes against the hypothesis's guess that a first stash hit (an in-bucket move only) would be cheaper. The likely reason is that every reused item now takes two stash touches plus the full promote/demote cascade, not one. The added `lfu_counter` read/write in the stash branch also costs a little. `put` got slower by a similar amount even though its code path did not change. That suggests a second-order effect: items stay in stash longer, so stash buckets fill and evict on insert more often. Benchmark noise is the other possible cause, and it should be ruled out before anyone acts on the `put` slowdown.

## Takeaway

A 2Q-style "two stash hits to promote" filter works for Dash on MSR block traces. Reusing the existing `lfu_counter` field gave the gain without growing `Item`, so metadata-size did not change. The cost is latency: the filter makes stash do more work as a buffer, and that shows up on `put` as well as `get`. Follow-up variants should keep the gate and cut its overhead, for example by avoiding the extra MRU move on the first hit. Or they could test whether the `put` regression comes from stash pressure.

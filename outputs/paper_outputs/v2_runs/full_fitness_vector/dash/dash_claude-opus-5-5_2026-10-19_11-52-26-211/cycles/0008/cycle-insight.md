# Cycle 0008 Insight

## Cohort outcome

All three variants branched from cycle-7 v002, the double-gated latency parent. v001 cut hashing to one per key along the stash-hit/demotion chain, with no change in state. v002 added target-only rescue that needs no scan. v003 replaced `Box<dyn Bucket>` with a concrete bucket type in a contiguous `Vec`. None of the three got the get-p50 gain it was aiming for. v001 lowered the get tail with hit-rate exactly unchanged. v002 recovered almost all of full rescue's hit-rate, but it also paid almost all of rescue's get cost. v003 left the median flat and made the tails worse.

## Patterns

- The median get no longer responds to hashing or to dispatch overhead. Removing hashes and removing the bucket-level vtable hop both left p50 flat. What remains of the median cost is walking resident slots and following the pointer to each bucket's separate heap item buffer. The stash-hit → promote → demote chain is a tail event, so trimming it pays off only at p99.
- Rescue's hit-rate and its latency cost both come from the same source: target holes. Probing holes add almost nothing, and keeping rescue out of probing buckets keeps the `foreign` gate clean. Even so, the cost stays, because fuller target buckets alone lengthen scans and cause more promotion cascades. The hit-rate vs get-latency trade for extra residency in target buckets now looks irreducible as long as the scan cost per resident stays the same.
- Changing data layout only helps if it removes the hop that is actually slow. Swapping a boxed bucket for a header-sized struct that still points to a heap `Vec<Item>` and still holds a boxed policy moved the indirection down one level. The wider bucket stride may also have cost cache lines.
- Put p99 went back up in all three variants. This matches cycle 7's warning that the parent's low put tail was variance. Treat the parent's put-p99 baseline as unreliable.

## Open questions

- Would inline slot storage (fixed `[Item; N]` per bucket, or a flat per-segment item array) together with a static eviction policy finally make each probe cheaper and move get p50?
- If probes get cheaper, does target-only rescue's latency cost shrink enough to make its hit-rate worth keeping? Its cost scales with the number of residents scanned.
- Can rescue be throttled so it limits promotion cascades rather than target occupancy? For example, rescue only when the victim has shown reuse, or place rescued items so they give way first.

## Continuation

Keep v001's single-hash plumbing as the new latency parent: it improves the tail at no cost. Keep target-only rescue as the preferred hit-rate add-on, since it is better than full rescue. Next, make the cost per probe smaller by storing slots inline, with a static policy, in a flat or fixed-array layout. Then re-stack target-only rescue on top of that and re-measure the trade. Stop removing hashes and stop making bucket-level devirtualisation changes while slots stay behind a heap `Vec`: neither will move get p50. Re-measure the put-p99 baseline before judging write-path claims.

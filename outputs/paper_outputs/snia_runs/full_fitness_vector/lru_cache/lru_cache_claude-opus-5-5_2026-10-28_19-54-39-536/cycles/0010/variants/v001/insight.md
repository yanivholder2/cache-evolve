# Variant v001 Insight

## Helped

- per-op-time (put): Put p50 and p99 both fell, as predicted. An evicting `put` hashes twice (`map.entry(key)` and `map.remove(&old_key)`), so swapping SipHash-1-3 for a two-multiply fold saves on both hashes. This adds to the evidence that put cost tracks hash cost.
- per-op-time (get p99): Get p99 dropped back from the parent's outlier to roughly where put's tail sits. `get`/`on_hit` are byte-identical to the parent, so the only thing that changed on this path is the hasher. That suggests the parent's get-p99 spike came from the hashing layer: either per-run `RandomState` seeding gave a bad table layout, or SipHash cost showed up in the tail. It does not support the parent insight's alternative explanation, deleted-slot buildup from insert-before-remove, because that order is unchanged here.
- metadata-size (LruCache header): The header shrank by exactly the 16 bytes the hypothesis predicted, because the zero-sized `FoldBuild` replaces the two-word `RandomState`. `Node` stays 24 bytes, so the summary does not move.

## Hurt

- per-op-time (get p50): The summary metric did not improve and ticked up slightly. The cheaper hash on the hit path did not reach the median. The hypothesis put SipHash at about 15–20 cycles of a ~100-cycle get, and that estimate looks too high for the median get. The median get is probably dominated by the bucket/control-byte load and the `nodes[idx]` load plus the `on_hit` relinking (dependent loads on adjacent nodes), not by hash arithmetic.

## Takeaway

A well-mixed folded-multiply hasher is a safe, policy-neutral swap. Hit rate is bit-identical, it fixes the low-bit entropy problem that sank cycle 2's Fx swap, and it clearly cuts put cost and both tail latencies. It does not move get p50, though, so hash cost is not what limits the median get. Further work on the summary metric should target memory access on the hit path (the map probe, node locality, the number of nodes `on_hit` touches) rather than more hashing tweaks.

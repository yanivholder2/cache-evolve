# Variant v001 Insight

## Helped

- per-op-time: Swapping SipHash for a multiplicative Fx-style hasher cuts the fixed cost of each index lookup, so the typical `get` got faster. Typical `put` improved slightly, since it does several membership probes.
- metadata-size: The whole-cache struct shrank, even though the reported per-entry summary is unchanged. The std `RandomState` keys are gone from each index, and the zero-sized `BuildHasherDefault` replaces them.

## Hurt

- per-op-time: The `put` tail got worse. Eviction and ghost-trimming work still dominates there, and the hasher doesn't touch it. The simpler hash may also cluster keys more, and pre-sizing the tables adds up-front allocation. The tail is noisy, so this may be partly variance.

## Takeaway

Hashing was a real but small constant factor on the `get` path. Hit-rate is identical to the parent, so the change was behavior-neutral, as required. The `put` tail comes from eviction and ghost trimming, not from hashing. Further gains on `put` have to come from reducing that work, not from cheaper hashing.

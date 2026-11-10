# Variant v002 Insight

## Helped

_None measured._ The fitness vector has only one dimension, hit-rate, and it matches cycle 0003/v001 exactly in all four trace×size cells. H02 was a cost-only change, so an identical hit-rate is what it was supposed to produce, not a gain.

## Hurt

- hit-rate: no regression in any cell (web_0 and hm_0, at both 16384 and 131072). This confirms on real traces that swapping SipHash for `FastBuild` and shrinking ghost stamps to `u32` with `wrapping_add` changed no admission, graduation or eviction decision. The kept `hit_miss_identical_to_linear_reference` oracle enforces the same thing in tests.

## Takeaway

For the second cycle running, the variant targeted per-op-time and metadata-size, and neither was in the evaluation. So whether the cost work paid off is still unknown. All we know is that it did not change the policy. Reading the diff also shows the expected gains are smaller than the hypothesis suggests:
- Shrinking `Loc` to 8 bytes cuts each index slot from 24 to 16 bytes for `i64` keys. But the ghost deque's `(K, u32)` pads back to 16 bytes, so ghost slots save no memory.
- The ghost-hit path in `put` still does two hash lookups (`get`, then `get_mut` on the slab copy of the key) instead of one `entry` call.
- The graduation path already did one lookup in the parent.

That leaves three real hot-path savings: a cheaper hash on every lookup, one fewer key clone per ghost insert (the `Loc` is now re-pointed in place), and no clone on a ghost hit. The next eval should measure per-op-time and metadata-size for v002 against v001. Because the policy is bit-identical, any difference comes from the hasher and the layout alone.

# Variant v001 Insight

## Helped

- hit-rate: No movement — the intended result. Swapping `Item.timestamp` from `Instant` to a process-global `AtomicU64` logical clock (`next_timestamp()`), read by `TimestampLRU`'s `min_by_key`/`max_by_key`, preserves the exact age order (smaller = older), so every victim and promotion choice is identical.

## Hurt

- hit-rate: No regression. The counter is bumped at the same sites the old `Instant::now()` was (item creation and a hit's refresh in `get_from_position`), and the SLRU lineage never reads `timestamp`, so no admission or segmentation decision changed.

## Takeaway

This variant proves that the only property `timestamp` carries anywhere in the code is a total age order, which a monotonic u64 satisfies as faithfully as a two-word `Instant`. The actual payoff the hypothesis chased — shrinking `Item<u64,u64>` from 40 to 32 bytes — is a metadata-size win that lives on an axis this hit-rate-only fitness vector cannot observe, so the change is provably free here but its benefit goes unmeasured.

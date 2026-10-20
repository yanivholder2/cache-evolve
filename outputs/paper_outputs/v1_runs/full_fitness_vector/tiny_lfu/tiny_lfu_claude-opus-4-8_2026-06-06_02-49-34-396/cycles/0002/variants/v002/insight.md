# Variant v002 Insight

## Helped

- per-op-time: Swapping the SipHash `DefaultHasher` in `base_hash` for the local multiply-xorshift `FastHasher` reduced the single per-op hash to one wrapping multiply for `u64` keys, lowering both get and put p50 and trimming put p99.

## Hurt

- hit-rate: The weaker `FastHasher` avalanche slightly correlates the two derived column streams from `position_for`'s double hashing, inflating a few frequency estimates and nudging hit-rate down marginally.

## Takeaway

Replacing the cryptographic base hasher with an FxHash-style multiply-xorshift is a favorable trade for this sketch: it cuts the dominant remaining per-op cost since v002 already pays only one hash, at the price of a tiny avalanche-driven hit-rate loss. Counter geometry, capacity, and LRU are untouched, so metadata-size is unchanged.

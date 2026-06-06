# Variant v001 Insight

## Helped

- per-op-time: Replacing SipHash `RandomState` with the trivial `U64BuildHasher` (one Fibonacci multiply on `write_u64`) on the `target_heat` probe cut get-p50 and put-p50 substantially, and collapsed both tail latencies — the side-table probe runs on every hot target hit, so removing the keyed permutation showed through on all four op metrics.

## Hurt

- (none) — no dimension regressed.

## Takeaway

Swapping only the `target_heat` hasher — contents, capacity, and the `promotion_get`/`put` gate decisions all byte-for-byte unchanged — moved get-p50 below the parent's 86 ns floor that the previous insight pinned on this exact probe. This proves the SipHash cost on the integer-keyed side table, not the gate logic or layout, was bounding this variant's latency; a DoS-resistant hasher on a `u64`-keyed hot probe is pure overhead here, and removing it is free of any hit-rate or metadata cost.

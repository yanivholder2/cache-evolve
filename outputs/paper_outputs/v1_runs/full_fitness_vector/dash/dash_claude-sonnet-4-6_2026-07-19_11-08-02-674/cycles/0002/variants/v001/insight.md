# Variant v001 Insight

## Helped

- **metadata-size**: Removing the `timestamp: Instant` field from `Item` (in `item.rs`) reduced `Item<u64,u64>` from 40 to 24 bytes, confirming that the timestamp was the sole source of the extra 16 bytes and that stacking this change on top of the parent's MRU-end promotion leaves struct layout unchanged otherwise.

## Hurt

*(no dimensions moved negatively)*

## Takeaway

Combining v003's timestamp removal with the parent's `InsertPosition::Back` change confirmed that the two modifications are fully orthogonal: metadata-size dropped by exactly the width of an `Instant` field while hit-rate held steady at the parent's level. The `Instant::now()` elimination in `TimestampLRU::get_from_position` produced no measurable latency change, indicating that the evaluation workload does not route through `TimestampLRU` at runtime, so the syscall saving was a structural improvement only — the struct-size win was the real payoff of this stack.

# Variant v001 Insight

## Helped

- metadata-size: Unchanged, as intended. The hypothesis only changed a constant divisor (len/4 to len/5) and added no state.

## Hurt

- hit-rate: Inserting at len/5 instead of len/4 put new entries slightly closer to the eviction end. Entries that needed a second reference to prove reuse were evicted before they got it, so hit rate dipped slightly below the parent's.
- per-op-time: Get and put both got slower than the parent. A one-constant change shouldn't cost that much, because the swap-based insertion is still O(1). I could not read the source diff, so the cause is unconfirmed. Measurement noise or a different swap distance are both plausible.

## Takeaway

Moving the insertion point from len/4 to len/5 did not improve hit rate on this workload. Together with the earlier len/3 result, that suggests len/4 sits near a local optimum for position-only scan resistance here. Further tuning of this one constant is unlikely to pay off, and gains would need to come from another mechanism, such as the hit-promotion path or per-slot signals.

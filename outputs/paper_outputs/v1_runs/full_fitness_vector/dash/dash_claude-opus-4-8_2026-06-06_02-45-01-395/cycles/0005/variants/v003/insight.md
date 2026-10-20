# Variant v003 Insight

## Helped

- metadata-size: Holding `ghost`/`ghost_set` as `SegmentCore` container state (like `target_heat`) rather than as per-`Item` fields kept `Item<u64,u64>` at the 16-byte floor, so the admission lever cost nothing on footprint.

## Hurt

- per-op-time: Every `put` now does a `ghost_set` membership probe plus, on a ghost hit, a linear `VecDeque::position` scan in `ghost_take`, and every terminal stash overflow calls `record_ghost` (hash insert + FIFO trim) — the bookkeeping roughly doubled put p50 and pushed get/put p99 tails up.

## Takeaway

Adding a frequency-aware admission lever — fast-tracking re-inserted fully-evicted keys straight to target instead of stash probation — left hit-rate bit-for-bit at 0.6794, the same ceiling recency-only levers hit. Since this is the one untried admission/frequency class and it still didn't move, it resolves the standing open question: hit-rate on this trace is capacity/compulsory-capped, not admission-limited, and the only thing the ghost machinery buys is put latency.

# Variant v001 Insight

Note: only Read and Write were usable in this session, so I could not list or diff the overlay source. This insight rests on the hypothesis and the two fitness vectors, not on a line-level diff.

## Helped

- metadata-size: No change. The 2-bit per-slot counters were packed into spare bits, so the per-item and table structs stayed the same size, as the hypothesis required.

## Hurt

- hit-rate: Hit-rate dipped slightly. Picking the lowest-counter slot in the eviction scan gave no gain over the parent's victim choice. The small drop is likely noise, or aging the other scanned slots on each eviction may have discarded useful frequency information.
- per-op-time: Both get and put got slower at p50 and p99. Get now does a saturating counter update on every hit. Put now scans slots for the lowest counter and decrements the others on eviction. That is extra work on the hot paths, and it undid part of the put-latency gain from the doorkeeper variant.

## Takeaway

A frequency-aware victim choice on top of the aged doorkeeper added hot-path cost without raising hit-rate. On this trace, once admission is filtered, a 2-bit frequency signal at eviction does not separate good victims from bad ones. Eviction or promotion changes need to show a clear hit-rate gain to justify per-hit bookkeeping.

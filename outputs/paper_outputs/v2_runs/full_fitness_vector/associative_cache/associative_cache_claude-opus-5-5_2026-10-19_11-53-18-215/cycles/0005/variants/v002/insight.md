# Variant v002 Insight

## Helped

- hit-rate: Raising the fixed protected fraction from 3/4 to 7/8 gives keys that were hit twice a larger part of each physical bucket. Cold keys from scans and loops still enter at probation `len0 / 2`, so they now compete for a smaller probation segment and cannot push out more of the reused working set. This continues the pattern from the hypothesis rationale: on this LIRS trace, giving protected more room pays off.
  - The change only takes effect on buckets of 16 or more slots: 16 → 14 instead of 12, 32 → 28 instead of 24.
  - In an 8-slot bucket, the 2-slot probation floor clamps 7/8 back to 6, which is exactly the old 3/4 cap.
  - In a 4-slot bucket the cap actually falls from 3 to 2.
  - Hit rate went up, which implies the eval geometry is on the ≥16-slot side, where protected grows.

## Hurt

- per-op-time: The diff touches only the `protected_cap` arithmetic in `with_layout`, which runs once at construction. The get and put code paths are byte-for-byte the same as v003. Even so, both p50 and p99 are slower for get and for put, and the dimension summary (get p50) is worse. There are two candidate explanations, and nothing here can tell them apart:
  - Measurement noise.
  - A side effect of the larger protected segment. A bigger protected segment makes the protected-hit `rotate_left` over `items[pos..]` span more items. A smaller probation also means `len0` is reached sooner, so the `remove_at(0)` + `insert` path (ghost hit, and `len0 == 0`) shifts a longer protected tail.

## Takeaway

On this trace, the fixed SLRU split had not reached its limit at 3/4. Moving to 7/8 with a 2-slot probation floor raised hit rate without changing any eviction logic. That supports protected capacity as the main lever on this workload, with v003's mid-probation insertion carrying most of the scan resistance in the remaining probation slots. The fraction and the floor interact with bucket size: at ≤8 slots the floor cancels 7/8 or even shrinks protected. So this result depends on the eval geometry, and it should be re-checked if bucket size changes. The per-op-time regression comes without any hot-path code change. Before blaming the larger protected segment for longer rotates and shifts, re-run v003 and v002 side by side to rule out noise.

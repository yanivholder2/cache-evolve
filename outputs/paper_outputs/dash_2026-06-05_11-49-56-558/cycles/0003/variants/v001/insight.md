# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Shrinking the stash from 8 to 4 slots cut cold-item re-entry capacity below what workload-a requires, reversing the gain trend that had held across the previous two normal/stash rebalances.

## Takeaway

The 87.5/12.5 split (28 normal / 4 stash) is past the optimal point for workload-a: hit-rate declined relative to v002's 24/8, exposing the breakpoint the hypothesis set out to locate. With only 4 stash slots, cold items that lose their normal-tier position cannot be held long enough to re-enter on the next access, and the eviction pressure from a larger hot tier no longer pays for itself. The 24/8 split from v002 appears to be the ceiling for this geometry-only lever on this workload.

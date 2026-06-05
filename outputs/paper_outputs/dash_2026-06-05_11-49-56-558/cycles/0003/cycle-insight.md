# Cycle 0003 Insight

## Cohort outcome

This cycle pushed three threads simultaneously: locating the ceiling of hot-tier expansion, compounding MRU re-entry onto the winning 75/25 topology, and retrying frequency-based eviction with a softer threshold gate. Only the compound (v002) won — with a small but clean gain — while the topology push exposed the breakpoint and the gentler aging-LFU still could not match position-based recency.

## Patterns

- **The topology gain curve has peaked.** Extending to 28/4 caused a regression, confirming that 24/8 is the optimal normal/stash split for workload-a. With only four stash slots, cold items cannot be held long enough to re-enter on their next access, and the added hot-tier pressure costs more than it returns. The geometry-only lever is now exhausted.
- **MRU re-entry is additive but nearly subsumed by topology.** Stacking Back-insert onto the 75/25 geometry produces a genuine, positive increment, but it is modest — because the enlarged normal-bucket pool already provides sufficient dwell time for natural promotion. The two axes do not interfere but they are not strongly synergistic either.
- **Position-based recency consistently beats frequency discrimination on this workload.** Threshold-gating the co-resident decay rescued co-residents from aggressive drain yet still underperformed ClassicLRU. The slow counter ramp-up creates its own eviction blind spot in early item life, replacing one failure mode with another. Item position encodes a stronger future-access signal than any frequency counter variant tested so far.

## Open questions

- With hot-tier topology exhausted and re-entry gains marginal, what genuinely new axes remain? Ghost/victim caches, admission control, or scan resistance have not been probed.
- Is there a re-entry formulation more nuanced than full Back-insert — e.g., inserting proportionally based on access count — that could extract more from the position lever?
- Would frequency-based eviction ever win on workload-a, or is the access pattern so recency-dominant that it is structurally incompatible?

## Continuation

The two previously dominant levers — hot-tier topology and MRU re-entry — are now close to fully exploited on workload-a; future hypotheses should pivot to unexplored structural axes such as ghost/victim-cache admission control or scan-resistant insertion policies. Frequency-based eviction in any form should be retired for this workload until evidence from a different workload trace motivates revisiting it. The confirmed optimal geometry (24/8) and the MRU re-entry policy together form the new compound baseline from which the next round of probes should launch.

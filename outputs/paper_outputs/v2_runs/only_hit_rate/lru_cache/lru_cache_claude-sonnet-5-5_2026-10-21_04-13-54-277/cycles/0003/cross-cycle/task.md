# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three single-axis changes were tried against the baseline: a compact node layout (metadata-size), a hot-path rewrite (per-op-time), and a segmented LRU (hit-rate). Only v003, the segmented LRU, was a Winner. It was the only variant that moved a dimension the evaluator actually recorded.

## Patterns

- Each hypothesis targeted a different dimension, so the cohort did not compete on one axis.
- The fitness vectors held only hit-rate. Metadata-size and per-op-time were missing for every variant, so the v001 and v002 gains are predictions, not results.
- v001 and v002 preserved strict LRU semantics (hit-rate matched the parent). That made them neutral on the one measured axis and unable to win.
- v003 gained hit-rate by departing from strict LRU. The invariant gate accepted it. The gain came from one trace and one cache size.
- v003's added bookkeeping (a per-node flag, a second list, demotion work) probably costs memory and latency, but nothing measured it.

## Open questions

- Is the missing metadata-size and per-op-time data a harness gap or a configuration issue? It decides whether the next cycle can judge any variant.
- Does v003's hit-rate gain hold across other traces and cache sizes, or is it specific to the scan-heavy trace?
- How much metadata and latency does v003 pay for its gain?
- Do the v001 compact layout and v002 fast paths combine with segmentation without cancelling each other?

## Continuation

First, get metadata-size and per-op-time measured, or the cycle's selection signal is hit-rate alone. Then build on v003 by combining it with v001's compact links and v002's cheap promotion, so the segmented design recovers the size and speed it gives up. Do not propose more strict-LRU micro-optimizations until those dimensions are measured.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Three variants were tried: two tweaks to the segmented LRU (a ghost list for direct-to-protected re-admission, and a smaller protected share) and a fresh CLOCK second-chance rewrite of the baseline. Only the CLOCK variant (v001 in the winners list, per its hit-rate gain over strict LRU) won. Both segmented-LRU refinements slightly lowered hit-rate.

## Patterns

- Both tweaks to the segmented design lost hit-rate. Each one weakened the protected segment, either by letting ghost-hit keys displace proven entries or by shrinking it.
- Evidence in this cycle points toward a large, stable protected set. The useful direction is at or above the current split, not below it.
- The CLOCK variant won a small hit-rate gain through a different mechanism. Unreferenced insertion plus a second-chance bit gives scan resistance without list manipulation.
- Metadata-size and per-op-time were again missing from every fitness vector. All size and speed claims, including CLOCK's cheaper hits and smaller nodes, are still unverified.
- All results come from one trace (the scan-heavy LIRS workload) at one cache size. The gains and losses are small.

## Open questions

- Is the measurement gap for metadata-size and per-op-time still unresolved? It has now blocked judgment for two cycles.
- Does the CLOCK gain hold on other traces and cache sizes, or is it specific to this one?
- Would a gentler ghost use (insert at probation MRU, or adaptive segment sizing as in ARC) help where the direct-to-protected version hurt?
- Does a protected share above 80% help further?

## Continuation

Fix or confirm metadata-size and per-op-time measurement before proposing more variants, because otherwise CLOCK's main claimed advantage cannot be judged. Build on the CLOCK winner next, for example with a CLOCK variant that keeps frequency or ghost information, and check it on more traces. Drop the smaller protected share and unconditional ghost-to-protected admission; if the segmented line is revisited, try a larger protected share or a gentler ghost policy.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

Three variants were tried: a larger protected share for the segmented LRU (90%), a gentler ghost list that re-admits at probation MRU, and a frequency-aware CLOCK (2-bit counter) rewrite of the baseline. Only the protected-share increase (v001) was a Winner. It gave a small hit-rate gain. The ghost variant matched its parent exactly, and the frequency-CLOCK variant showed a modest gain but did not win.

## Patterns

- The segmented line improved by tuning a single constant. Two cycles of evidence now agree: a smaller protected share loses hit-rate and a larger one gains it.
- The gentler ghost policy removed cycle 0002's regression but produced no gain. Its hit-rate was identical to the parent's, so the ghost path likely rarely changed a decision on this trace.
- Frequency-aware CLOCK reports scan-resistance gains similar to the earlier one-bit CLOCK, which suggests that adding frequency bits is incremental, not transformative.
- Every gain this cycle is small and rests on one scan-heavy trace at one cache size.
- Metadata-size and per-op-time were again absent from all fitness vectors. This is the third cycle with that gap, so the claimed size and speed advantages of CLOCK and the cost of the ghost list remain unverified.

## Open questions

- Does the protected-share gain keep growing, plateau, or reverse past 90%? Probation must still absorb scans and give new keys time for a second hit.
- Do the segmented and CLOCK gains hold on other traces and cache sizes, particularly ones with more reuse at long distances where a ghost list might matter?
- Is the missing metadata-size and per-op-time measurement a harness fault? It has now blocked judgment for three cycles.
- Can the protected-share tuning combine with a CLOCK or frequency mechanism?

## Continuation

Resolve the metadata-size and per-op-time measurement gap before any further variants, or the loop is selecting on hit-rate alone. Next, probe the protected share around 90% (for example slightly above, or adaptive sizing) and test the winner on more traces and cache sizes. Drop the probation-MRU ghost list unless a trace with long-distance reuse is added, and stop spending cycles on further single-constant tweaks without a second trace to validate them.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>

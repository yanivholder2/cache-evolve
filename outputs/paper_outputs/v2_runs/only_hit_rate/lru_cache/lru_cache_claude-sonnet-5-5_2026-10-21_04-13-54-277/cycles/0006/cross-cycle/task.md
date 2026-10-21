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


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

Three variants refined the segmented LRU: a larger protected share (95%), a second-chance bit on protected eviction, and a stamp-based fast path that skips relinking on hits. Only the protected-share increase (v001) won, with a marginal hit-rate gain. The second-chance variant matched its parent, and the relink-skip variant lost a little hit-rate.

## Patterns

- The protected-share curve is nearly flat past 90%. The gain is marginal, and probation is close to its minimum useful size.
- Refinements to protected-segment eviction or recency tracking did not help. The second-chance bit sits on a fallback path that rarely runs at this share. The approximate-recency shortcut gave up some hit-rate.
- Cheaper-per-op ideas (skipping relinks) cannot be judged. Metadata-size and per-op-time are absent from every fitness vector for the fourth cycle in a row.
- Gains are again small and rest on one scan-heavy trace at one cache size.

## Open questions

- Is the measurement gap for metadata-size and per-op-time a harness fault? It has now blocked judgment for four cycles.
- Do the segmented gains hold on other traces and cache sizes, especially ones with long-distance reuse or less scan pressure?
- Would adaptive segment sizing (ARC-style) beat any fixed share when the workload changes?
- Does the relink-skip shortcut pay for itself in time, if per-op-time were measured?

## Continuation

Stop tuning the protected share and drop protected-eviction refinements such as the second-chance bit, because both are at a plateau on this trace. Before any new variants, fix the metadata-size and per-op-time measurement and add at least one more trace or cache size. After that, pursue changes that act on probation or promotion, such as adaptive sizing or combining the segmented design with CLOCK or frequency ideas. Revisit the relink-skip only if per-op-time can show its benefit.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Three variants targeted probation on the 95%-protected segmented LRU: a TinyLFU-style sketch admission gate, a change to where demoted protected nodes land in probation, and a frequency-aware probation eviction scan. Only the admission gate (v001) won, with a marginal hit-rate gain. The other two matched their parent exactly.

## Patterns

- Probation-side tweaks are nearly inert at this protected share. Demotions are rare, so where demoted keys land in probation changes nothing.
- The frequency-scan variant was dead by construction. Frequency was reset or never persisted in the segment being scanned, so every candidate tied and the scan fell back to plain LRU order.
- Admission filtering gave a small gain, but segmentation already keeps one-hit scan keys in the small probation segment. The sketch's extra work (hashing, counters, periodic decay) buys little. Counting rejected misses lets them be admitted on retry, and the gate can reject new hot keys.
- Hit-rate gains remain marginal and rest on one scan-heavy trace at one cache size.
- Metadata-size and per-op-time were not used to judge anything this cycle, and the cost of the sketch and the extra freq byte per node is unverified.

## Open questions

- Is the sketch gate worth its time and memory cost, if per-op-time and metadata-size could be measured?
- Does admission filtering matter more on traces with long-distance reuse or heavier scan pressure?
- Would frequency help if it persisted across demotion, or covered protected entries?
- Is the measurement gap on metadata-size and per-op-time (now five cycles) a harness fault?

## Continuation

Stop varying probation placement and probation-only eviction scans; the segmented design is saturated on this trace. Keep the admission gate only if cost can be measured, and prefer cheaper variants such as not counting rejected misses or a tiny doorkeeper. Before more hypotheses, fix the cost measurement and add a second trace or cache size. Then try ideas that change a different axis, such as adaptive sizing, CLOCK hybrids, or frequency that persists across segments.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

Three variants worked on the sketch admission gate or its CLOCK alternative: no longer counting rejected misses, a one-bit doorkeeper in front of the sketch, and a CLOCK + sketch-gate rewrite of the strict LRU. Only the doorkeeper (v002) won, with a very small hit-rate gain. Not counting rejected misses lost a little hit-rate.

## Patterns

- Admission-gate tuning is saturated. The segmented LRU at 95% protected share already filters most scan keys, so cleaner frequency estimates leave little to gain.
- Counting rejected misses helps. It lets recurring keys build frequency across retries and earn admission. Removing it hurt.
- The doorkeeper's benefit (less scan pollution in the counters) is mostly offset by its costs. A new hot key estimates only 1 and loses the gate more often, and each halving forgets recent first sightings.
- CLOCK plus a sketch gate reports a hit-rate gain over strict LRU, but it did not win. The comparison base is unclear. This is the same "CLOCK is incremental" pattern seen in cycle 0003.
- Metadata-size and per-op-time were again missing from the fitness vectors, now for six cycles. The cost of the sketch, doorkeeper and CLOCK hand sweep is unverified.
- All gains are marginal and rest on one trace at one cache size.

## Open questions

- Is the doorkeeper's tiny gain worth its added state and logic, given that costs cannot be measured?
- Would the CLOCK + sketch design beat the current best segmented line if compared head to head, and on what cost profile?
- Does admission filtering matter on traces with long-distance reuse or heavier scan pressure?
- Is the missing cost measurement a harness fault?

## Continuation

Stop tuning the admission gate: the sketch update rule, the doorkeeper and the threshold are all at a plateau on this trace. Before more hypotheses, fix metadata-size and per-op-time measurement and add a second trace or cache size, since the loop is selecting on hit-rate alone with only marginal differences. Then pursue a different axis, such as adaptive segment sizing, frequency that persists across segments, or a direct cost-aware comparison of the CLOCK + sketch design against the segmented winner.


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

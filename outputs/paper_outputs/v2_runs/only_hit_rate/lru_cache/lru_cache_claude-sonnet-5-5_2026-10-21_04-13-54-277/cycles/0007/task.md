# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0007

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0007/hypotheses/H03.md

## Prior cycles digest

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

Exactly one file: `cycle-insight.md` in the output dir.

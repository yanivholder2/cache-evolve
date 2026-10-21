# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0006

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0006/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.

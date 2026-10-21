# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0005

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0005/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0005/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0005/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0005/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0005/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.

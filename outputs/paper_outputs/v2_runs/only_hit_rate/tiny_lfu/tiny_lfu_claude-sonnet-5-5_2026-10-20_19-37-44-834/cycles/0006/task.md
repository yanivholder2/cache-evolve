# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0006

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0006/hypotheses/H03.md

## Prior cycles digest

### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

Three changes were tried: online hill-climbing of the window size (v001), a looser window-to-probation admission duel that tie-admits repeat keys (v002), and a 4-bit packed sketch with a shared hash (v003). v002 won with a very small hit-rate gain. v001 was roughly neutral, and v003 lost hit-rate.

## Patterns

- The gains are all marginal and near noise on workload-a. The workload is stable and the parent is already near its optimum for window size and admission, so structural tweaks keep returning roughly nothing.
- The only change that helped kept frequency information intact and loosened admission slightly. The change that degraded information (v003: saturating counters, no aging, correlated row indices) lost hit-rate. This matches cycle 2: preserve frequency counts.
- v003 contradicts the cycle-1 reading that a compact sketch costs no accuracy. That reading held for u8 counters with conservative update. It does not hold for 4-bit counters with no aging and a shared hash.
- For the third cycle running, the fitness vectors report hit-rate only. The metadata-size and per-op-time goals of v003 and the cost risks of v001 and v002 are still unmeasured, so none of the results can be judged on cost.
- Adaptive window tuning cannot show its value on a single stable trace. Its added machinery is unjustified without a phase-changing benchmark.

## Open questions

- Would v003 recover its hit-rate with periodic halving or with better-decorrelated row indices? Do its size and speed savings justify any remaining loss?
- Is v002's win real? It is tiny, on one trace and one cache size.
- Does v001's hill-climbing pay off on a trace whose best window size shifts?

## Continuation

Keep v002's tie-admit duel as the new base. Before more hit-rate ideas, fix the fitness output to report metadata-size and per-op-time, and add a second trace with shifting popularity or a different cache size. Then rerun v003's compact sketch with aging and a decorrelated hash, and judge it on cost. Drop further hit-rate-only tuning of window size and admission on workload-a.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

Three cost-oriented or structural tweaks were tried: halving the sketch row width (v001), growing the protected segment share of main (v002), and deriving all row indices from one 64-bit hash (v003). v002 was named Winner but matched its parent exactly on hit-rate. v001 lost hit-rate, and v003 held hit-rate level.

## Patterns

- The Winner is a tie, not a gain. Its status comes from not regressing, so the protected/probation split looks irrelevant on workload-a at this cache size.
- Changes that leave frequency estimates intact (segment split, single-hash row derivation) were neutral. Changes that degrade estimate quality (narrower rows causing more counter collisions) lost hit-rate. This matches the cycle 2 and 3 theme: preserve frequency information.
- v001 contradicts the cycle 1 reading that the sketch is over-provisioned. Halving width is too aggressive, while hash-sharing (v003) appears safe.
- For the fourth cycle running, the fitness vectors report hit-rate only. Metadata-size and per-op-time, the stated goals of v001 and v003, are still unmeasured, so no cost trade can be judged.
- Variant authors again could not always read overlay source, so explanations rest on hypothesis text and fitness vectors.

## Open questions

- Does the single-hash change actually reduce per-op-time? That is unverified.
- Would a gentler width reduction than 2x keep hit-rate while saving size?
- Is workload-a simply saturated, so that no structural tweak can move hit-rate on it?

## Continuation

Fixing the evaluation is now the blocker: add metadata-size and per-op-time to the fitness output, plus a second trace (shifting popularity or a different cache size), before generating more hypotheses. Keep v003's single-hash indexing as a candidate cost win to verify on speed. Drop protected-segment sizing and further 2x width cuts. If tuning sketch size, try a smaller step and judge it on size.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Three changes were tried: a further-loosened admission rule gated on a window hit (v001), a gentler 25% cut in sketch row width (v002), and reusing precomputed sketch indices across estimate and increment (v003). v001 was named Winner, but its own insight reports a marginal hit-rate loss. As in cycle 2, it looks like the least-bad variant, not a real gain.

## Patterns

- Same recurring split: changes that degrade frequency information (looser admission, narrower rows) lost a little hit-rate. The behavior-preserving change (index reuse) matched the parent exactly.
- Admission loosening has hit its limit. The tie-admit duel already captured the benefit, and going past ties swapped in colder entries. In-window recency adds nothing beyond the sketch estimate here.
- Even a gentle width cut cost hit-rate, so the sketch is already near the width where collisions matter. Combined with cycle 4, width reduction of any size looks unproductive on this workload.
- For the fifth cycle running, the fitness vectors report hit-rate only. The cost goals of v002 and v003 (metadata-size, per-op-time) remain unmeasured, so the Winner choice rests on hit-rate among near-ties.
- Variant authors again could not always read overlay source, so explanations rest on hypothesis text and fitness vectors.

## Open questions

- Does v003's index reuse actually reduce per-op-time, and does the cached index array cost metadata?
- Does v002's size saving justify its small hit-rate loss? That cannot be judged without a size measurement.
- Is workload-a saturated, so that no admission or sketch tweak can move hit-rate on it?

## Continuation

Do not generate more hypotheses until the evaluation is fixed: add metadata-size and per-op-time to the fitness output, plus a second trace (shifting popularity or a different cache size). Drop further admission loosening past the tie-admit rule and drop sketch width cuts. Keep v003's index reuse as a candidate cost win and verify it on speed first.


## Write

Exactly one file: `cycle-insight.md` in the output dir.

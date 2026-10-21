# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0005

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0005/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0005/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0005/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0005/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0005/hypotheses/H03.md

## Prior cycles digest

### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Three single-lever changes were tried: sketch aging on top of the windowed design (v001), a larger window (v002), and a doorkeeper in front of the baseline sketch (v003). None improved hit-rate over its parent. v002 was named Winner, but its own insight reports a small hit-rate loss. It looks like the least-bad variant, not a real gain.

## Patterns

- Every variant moved hit-rate slightly the wrong way on workload-a at the tested cache size. The workload looks stable and frequency-driven, with little recency or popularity drift for these mechanisms to exploit.
- Each change that discards or hides frequency information cost hit-rate. Aging decays history, a larger window dilutes main-region protection, and the doorkeeper hides first sightings from the counters.
- Metadata-size and per-op-time are still missing from the fitness vectors. This is the second cycle with that gap, so the cost side of every idea is still unmeasured.
- Variant authors could not always inspect overlay source, so the explanations are plausible readings and not confirmed causes.

## Open questions

- Is the Winner status of v002 an artifact of ranking among losers, or is it a real gain on some dimension that wasn't reported?
- Would aging pay off with a longer sample period, or on a trace with real popularity shifts?
- Would a doorkeeper help if admission were adjusted for its +1 offset, or on a workload with real sketch pollution?
- Do the cycle-1 and cycle-2 changes differ in metadata or per-op cost even though hit-rate barely moves?

## Continuation

Stop tuning window size and stop adding decay or filtering on workload-a alone, since both lost hit-rate. First fix the fitness output so it includes metadata-size and per-op-time, and add a second trace with shifting popularity so aging and doorkeeper ideas can be judged fairly. Then pursue cost-oriented changes, such as v003-from-cycle-1's compact sketch, or admission tuning that preserves frequency information, and judge them on cost as well as hit-rate.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.

# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Two structural variants were evaluated: a window plus segmented main region (v002) and a compact u8 sketch with single-hash indexing and conservative update (v003). The aging variant (H01, v001) was rejected at fitness because the simulator build failed on an HTTP 429 while downloading the Caffeine jar, so it never produced a result. v002 won with a marginal hit-rate gain, and v003 was essentially neutral.

## Patterns

- Both evaluated variants moved hit-rate only negligibly on workload-a, so the baseline's sketch-gated admission already captures most of the available benefit.
- Both fitness vectors reported hit-rate only. The metadata-size and per-op-time effects, which were the stated goal of H03 and the main risk of H02, are unmeasured.
- Narrower counters and cheaper hashing cost no accuracy, so the sketch looks over-provisioned.
- The only rejection was infrastructure-related (network rate limit), not a flaw in the idea. Aging remains untested.

## Open questions

- Does periodic halving (aging) help? It was never evaluated.
- Do the v002 window/segmentation and v003 sketch compaction help or hurt on metadata size and per-op time?
- Is the small v002 win real, or noise? Other traces or cache sizes might show a different result.

## Continuation

Re-attempt aging (H01) first, since the simulator failure was environmental, and check that the simulator's dependencies are cached or retried before the run. Make sure the fitness output includes metadata-size and per-op-time, so v002 and v003 can be judged on cost. Consider combining v003's compact sketch with aging or the window. Don't keep tuning hit-rate on structure alone until the missing dimensions are measured.


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


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

Three changes were tried: blocking all sketch rows into one cache-line block (v001), choosing the protected-overflow demotion victim by sketch estimate (v002), and replacing pointer links with u32 slab indices (v003). v002 was named Winner, but it is effectively a tie with its parent, within noise. v001 lost hit-rate, and v003 preserved behavior.

## Patterns

- Same split as earlier cycles: changes that weaken frequency-estimate quality (blocked rows with correlated slots) lost hit-rate. Changes that leave estimates and admission untouched (demotion choice, slab links) were neutral.
- The Winner is again a near-tie. Segment internals (protected size in cycle 4, demotion choice here) have now been flat twice, so they do not move hit-rate on workload-a.
- Locality and cost optimizations (v001, v003) show only their downside, or nothing, because the fitness vectors still report hit-rate only. This is the sixth cycle running with metadata-size and per-op-time unmeasured.
- Variant authors again could not always read overlay source, so explanations rest on hypothesis text and fitness vectors.

## Open questions

- Does v003's slab layout shrink metadata or speed up ops? It is behavior-preserving, so it is a free win if it does.
- Could a blocking scheme with better row decorrelation (for example, a different hash per row within the block) keep hit-rate and still gain locality?
- Is workload-a saturated for hit-rate? Six cycles of tweaks have produced only noise-level movement.

## Continuation

Stop generating hit-rate hypotheses until the evaluation reports metadata-size and per-op-time and includes a second trace (shifting popularity or a different cache size). Keep v003's slab links and the earlier single-hash and index-reuse changes as candidate cost wins, and verify them on size and speed first. Drop segment-internal tuning (protected size, demotion choice) and simple sketch blocking. Revisit blocking only with decorrelated rows, and only if per-op-time is measured.


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

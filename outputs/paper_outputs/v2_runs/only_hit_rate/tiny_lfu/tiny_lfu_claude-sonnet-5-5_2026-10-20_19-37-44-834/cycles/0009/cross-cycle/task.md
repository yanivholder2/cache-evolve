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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Three changes were tried: widening the sketch rows 4x (v001), counting each access once instead of twice on a miss-then-put (v002), and choosing the colder of the two oldest probation entries as victim (v003). v001 was the Winner with a small hit-rate gain in the predicted direction. It is the first width change to help, after several cycles of narrowing that lost. v002 lost slightly and v003 was a no-op within noise.

## Patterns

- Wider rows helped slightly, and narrower rows lost in cycles 4 and 5. Width therefore moves hit-rate monotonically but with sharply diminishing returns. The sketch is only mildly collision-limited.
- The Winner's gain is small and is bought with a metadata cost that is still unmeasured. Its value as a Winner depends on a size measurement that does not exist.
- The double count on a get-miss followed by a put acts as a useful admission bias toward new keys. Removing it made the unchanged `>= 2` tie-admit threshold too strict. Count semantics and the threshold are coupled and cannot be tuned independently.
- Victim selection inside probation (sampling two) is flat, like the earlier segment-internal tweaks of cycles 4 and 6. Probation order is mostly recency order, so the second-oldest entry is rarely colder.
- For the seventh cycle running, the fitness vectors report hit-rate only. The costs of v001 (size) and v003 (extra sketch estimates per miss) are invisible, and variant authors again could not read overlay source.

## Open questions

- Is v001's hit-rate gain worth four times the sketch metadata? An intermediate width such as 8x might capture most of it.
- Would the single-count change work if the tie-admit threshold were retuned together with it (for example to `>= 1`)?
- Does the never-decaying sketch limit hit-rate, so that aging or reset period is the real lever?
- Is workload-a saturated, so that only small gains remain?

## Continuation

Fix the evaluation first: add metadata-size and per-op-time to the fitness output, plus a second trace (shifting popularity or a different cache size). Then judge v001's width against its size cost, trying an intermediate width. Pursue sketch aging or reset-period changes, and retune count semantics and the tie-admit threshold together if revisiting single-counting. Drop probation victim sampling, further segment-internal tuning, and standalone single-count changes.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

Three changes were tried: single-counting with a retuned (lower) tie-admit threshold (v001), very infrequent sketch halving on the wide sketch (v002), and an intermediate sketch width of 8 on the baseline (v003). v001 was the Winner by matching its parent with a marginal, noise-level gain. v002 lost slightly, and v003 gained slightly but was not measured on size.

## Patterns

- Count semantics and the admit threshold are coupled, as cycle 7 suspected. Retuning them together restored parity, and parity is all it did. Admission-rule tuning has now been flat across several cycles.
- Even a very gentle aging period cost a little hit-rate. A never-decaying sketch is not the limiter on workload-a, so aging joins the dropped levers.
- The intermediate width gained slightly over the baseline, which fits the earlier picture of width helping with sharply diminishing returns. Its hypothesis text also described the change as a reduction when the factor goes up, so the direction and the size claim were muddled and cannot be trusted.
- Every dimension that could separate these variants on cost (metadata-size, per-op-time) is still unmeasured, for the eighth cycle running. The Winner is again a near-tie chosen on hit-rate alone.
- Variant authors again could not read overlay source, so explanations rest on hypothesis text and fitness vectors.

## Open questions

- Does single-count with a lower threshold simplify the code or speed up ops enough to count as a cost win?
- Does width 8 keep most of the width-16 gain at a lower sketch size? That cannot be judged without a size measurement.
- Is workload-a saturated for hit-rate, given eight cycles of noise-level movement?

## Continuation

Fix the evaluation before generating more hypotheses: add metadata-size and per-op-time to the fitness output, plus a second trace (shifting popularity or a different cache size). Then judge the sketch width/size trade-off using v003 and the earlier wide variant. Treat single-count with a lower threshold as a clean base, but drop further admission-rule tuning, sketch aging and halving, and segment-internal tweaks. State explicitly in any width hypothesis which direction the factor scales the sketch.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Three changes were tried: a fifth sketch row at unchanged width (v001), a recency-gated tie-break in admission using a "touched in window" flag (v002), and repacking the sketch as a flat u8 array with single-hash indexing (v003). v002 was named Winner, but all three moved hit-rate only at noise level, so the pick is effectively a tie with the parent.

## Patterns

- Both hit-rate-oriented changes (extra row, tie-break gating) landed within noise. This extends the earlier finding that estimation quality and admission rules are saturated on workload-a.
- The tie-break only matters on equal estimates, and it was nearly neutral. Ties are probably a small share of admission duels.
- v003 was a cost change (size and speed), but its hypothesis text was muddled: it mentioned width 8 while the described change did not use it. Its intended benefits were never measured, because the fitness vectors again report hit-rate only. This is the ninth cycle running.
- This cycle partly repeated earlier ground: sketch depth and admission tuning had already been flat. The hypothesis writer also could not read the overlay or baseline source, so the proposals were generic.
- Variant authors again could not read overlay source, so explanations rest on hypothesis text and fitness vectors.

## Open questions

- Does the flat u8 single-hash sketch actually shrink metadata or speed up ops? It looks behavior-neutral, so it may be a free cost win, but that is unverified.
- Is hit-rate on workload-a exhausted for sketch and admission changes? Eviction or segment structure, or a different trace, may be the only remaining lever.
- Does the v002 flag cost per-entry memory that outweighs its tiny gain?

## Continuation

Do not generate more sketch-depth, tie-break, or admission-rule hypotheses; all are flat. Fix the evaluation first: add metadata-size and per-op-time to the fitness output, plus a second trace (shifting popularity or a different cache size). Then judge the flat u8 packing (v003) and the earlier cost changes (slab links, single-hash) on size and speed. Any width or packing hypothesis must state its parent and scaling direction consistently.


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

# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0009

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0009/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.

# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0010

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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

Exactly one file: `cycle-insight.md` in the output dir.

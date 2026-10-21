# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0008

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0008/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.

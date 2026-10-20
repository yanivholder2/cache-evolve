# Task: cycle-insight-writer

Cycle: 0004
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0004

## Winners

- v002
- v001

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0004/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0004/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0004/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0004/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0004/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0004/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

The cohort tried three kinds of change: recency rotation of rejected admission victims (v001), a constant-factor guard that skips relinks at the MRU tail (v002), and a probation/protected segmentation of the resident list (v003). The two Winners are v002, a cost-only change with an unchanged hit rate, and v003, a small hit-rate gain that adds metadata.

## Patterns

- Rotating a rejected victim to the MRU tail hurt both hit rate and per-op time. The frequency-based admission gate already did the protective work, so recency rotation added cost without improving admission.
- Changes that leave eviction order alone and only trim pointer work on repeated accesses to the newest key are a safe way to gain speed.
- Segmentation produced the only hit-rate movement, but it is small and paid for with per-node and per-cache metadata growth.
- Every variant's evidence comes from one trace (lirs workload-a), so none of the Winners' gains has been shown to generalize.

## Open questions

- Does v003's marginal hit-rate gain hold on other traces, or is it specific to lirs workload-a?
- v003's latency drop is unexplained by its added bookkeeping and may be measurement variance. Is it real?
- How much of v002's saving depends on the share of accesses that hit the newest key?

## Continuation

Pursue further constant-factor, eviction-order-preserving guards like v002, and re-test v003's segmentation on at least one additional trace before building on it. Drop recency rotation of rejected victims (v001) as a direction.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

The cohort tried three kinds of change: hashing each key once and reusing it across the admission check and sketch update (v001), packing the protected flag into an existing node field while cutting resident-list anchors (v002), and a one-hit doorkeeper in front of the frequency sketch (v003). All three Winners kept hit rate close to their parent, and the gains were mostly in per-op time or metadata size, not in hit rate.

## Patterns

- Constant-factor work was the main source of gains. Hoisting the hash out of `put()` produced a clear put-path speedup, but it also moved hit rate slightly, so it is not behaviour-preserving as written.
- Metadata reduction and hit rate decoupled. Packing the protected state into existing fields shrank metadata without changing hit rate, so the bookkeeping cost of segmentation can be trimmed independently of the policy.
- The doorkeeper barely moved hit rate on this workload. One-hit-wonder pollution of the sketch does not appear to limit the admission gate here, and the change mostly added handle state and per-put work.
- Several gains and regressions are unverified attributions. Speedups were attributed to shared helpers and regressions to specific accessors or the doorkeeper without inspecting the source, and all evidence comes from one trace.

## Open questions

- Do the precomputed-hash helpers produce the same counter indices as the public sketch methods on the same keys? This determines whether v001's hit-rate drop is a bug or a real trade-off.
- Is the doorkeeper's memory cost fully reported? Its storage does not appear in the metadata breakdown.
- Does any of this hold on a trace where one-hit keys dominate, or on a second trace at all?

## Continuation

Pursue the hash-reuse direction from v001, but first confirm equivalence of the hashed helper indices so the hit-rate drop is resolved. Keep the metadata-packing approach from v002, since it cut cost without moving hit rate. Drop the doorkeeper (v003) as a direction unless a trace where one-hit keys dominate shows a real hit-rate effect.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

The cohort tried three changes to the admission and aging machinery: capping the protected segment without demotion (v001), switching the sketch's row indexing to the public per-row seeded scheme (v002), and halving the sketch's aging period (v003). The three Winners were each listed for a single trade-off rather than a clear net gain: v002 recovered hit rate at a cost in per-op time, and v001 and v003 mostly reported hit-rate or metadata costs.

## Patterns

- Hit rate and per-op time moved in opposite directions. v002 recovered admission fidelity by matching the public indexing, but gave back most of the speedup that the hoisted-hash design from the prior cycle had provided.
- Bounding the protected segment cost hit rate. The segmentation gain from Cycle 0001 seems to depend on letting the protected set turn over, so caps are the wrong lever.
- Shortening frequency aging hurt hit rate. Stale counts were not what limited the admission gate, which is consistent with the doorkeeper result in Cycle 0002.
- Several attributions are unverified. Source diffs could not be inspected, struct growth and some timing shifts do not match the single-constant changes that were intended, and every result still comes from one trace (lirs workload-a).

## Open questions

- Can the seeded per-row indexing from v002 be made cheaper while keeping the same counter indices, so the fidelity gain survives without the speed loss?
- Is the struct growth in v003 from unrecorded edits, and do the timing shifts in v001 and v003 survive repeat runs?
- Do any of these hit-rate effects hold on a second trace?

## Continuation

Pursue the index-equivalence work from v002: keep the seeded indexing semantics but cut its per-row cost, and verify it against the public path. Drop protected-segment capping (v001) and shorter sketch aging (v003) as directions, since both lost hit rate on this workload.


## Write

Exactly one file: `cycle-insight.md` in the output dir.

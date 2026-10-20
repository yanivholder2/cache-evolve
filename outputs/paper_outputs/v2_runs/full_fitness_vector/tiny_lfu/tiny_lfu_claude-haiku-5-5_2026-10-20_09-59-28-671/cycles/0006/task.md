# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0006

## Winners

- v002
- v003
- v001

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0006/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

The cohort tried three changes: caching per-row sketch seeds at construction instead of re-deriving them (v001), inlining the protected-flag accessors on the get hit path (v002), and a small W-TinyLFU-style window in front of the frequency gate (v003). The Winners are v001 and v002, both of which kept hit rate at their parent's value; v003 did not move hit rate and was not a Winner.

## Patterns

- Per-op cost is the main lever again. Seed caching recovered constant-factor time lost to the Cycle 0003 per-row indexing change, with no hit-rate change, which fits the claim that counter indices are unchanged.
- Caching the seeds added fixed per-instance metadata. The struct grew, which the hypothesis had tried to prevent. Storing the same values in a `const` table would likely keep the speed without the growth.
- Accessor inlining on the get path helped gets but not puts. The put-side regression was not predicted and may be run-to-run noise, so the get gain should not be read as a general win yet.
- A recency window did not help. Hit rate did not move, and the added list work on both get and put made latency worse. This matches earlier cycles, where changes that touched admission or aging did not raise hit rate on this workload.
- Several attributions remain unverified. The overlay source was not inspected in any of the three variants, so the links between changes and timing shifts are inferred, and all evidence comes from one trace (workload-a).

## Open questions

- Do the cached seeds produce exactly the same counter indices as the per-access derivation for every key? Hit-rate parity is consistent with yes but does not prove it.
- Can the seed table live in a `const` item so the speedup holds without struct growth?
- Does the put-side latency change from inlining survive repeat runs, and does the get gain hold on a second trace?
- Does the window's bookkeeping appear in the reported metadata at all? The reported node size did not change, so that figure may undercount it.

## Continuation

Pursue seed caching from v001, but move the seeds into a `const` table and verify index equivalence against the public path before building on it. Keep the get-path accessor inlining from v002 as a candidate, and re-run its put timing before accepting it. Drop the W-TinyLFU admission window (v003) as a direction on this workload.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

The cohort tried three access-pattern changes: splitting the put-side unlink and promotion path into a separate helper (v001), moving the per-row sketch seeds from a per-instance field into a module-level constant table (v002), and restoring the public per-row seeded indexing on top of a single precomputed hash (v003). All three are Winners, but for different reasons: v001 and v002 are per-op-time gains with hit rate unchanged, and v003 is a hit-rate correction that gives back speed.

## Patterns

- Seed storage was the cost behind the earlier size and speed regressions. Keeping the seeds in a `const` table, not per instance, removed the struct growth and kept the speedup, with hit rate unchanged.
- Index fidelity and speed trade off directly. Matching the public per-row seeded mixing restored the admission decisions, but reintroduced per-row work on both get and put. The speed win and the correct indices cannot both be kept by the current formulation.
- Put-side latency responds to the shared unlink and promotion path. Splitting it out improved put timing, but this rests on a single recorded run and has not been separated from noise.
- Every Winner kept hit rate at or near its parent's value except the index correction, and none of the variants changed the metadata reported for the policy itself. Source diffs were not inspected, so the mechanisms are inferred from hypotheses and fitness results.

## Open questions

- Can the public per-row indices be produced with less per-row work than the seeded mixing function, so the fidelity fix does not cost the speed gain?
- Do the `const`-table indices match the per-access derivation for every key, or only in aggregate hit rate?
- Does the put-side improvement from v001 survive repeat runs, and does it hold on a second trace?

## Continuation

Pursue the `const`-table seed layout from v002 and verify its index equivalence against the public path before building further on it. Keep the index-correctness direction from v003, but make the per-row derivation cheaper rather than accepting the speed loss. Hold the put-path split from v001 as a candidate until repeat runs confirm the gain.


## Write

Exactly one file: `cycle-insight.md` in the output dir.

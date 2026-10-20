# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0007

## Winners

- v002
- v003
- v001

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0007/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort tried three changes to the sketch's indexing and the protected segment: premixing the per-row seeds into a module-level `const` table (v001), demoting the protected tail to probation MRU (v002), and grouping a key's sketch counters into one cache-line block (v003). Each Winner is a single-axis trade-off rather than a clean net gain: v002 showed a timing gain with hit rate unchanged, v003 showed a clear per-op speedup at a cost in hit rate, and v001 kept hit rate and metadata flat while its timing read as a regression.

## Patterns

- Per-op time is again the only axis that moved decisively, and it moved in both directions. The block-grouped layout in v003 gave the clearest speedup, but it changed which keys collide in the sketch and cost hit rate. Index changes that alter collisions look like the main source of hit-rate movement this cycle.
- Premixed seed tables kept hit rate and metadata unchanged, which fits the claim that counter indices are preserved. They did not make the per-row path faster, and the timing signal from the single run was unstable, so the Cycle 0005 `const`-table layout should not be treated as a speed win yet.
- Protected-tail demotion produced no hit-rate change at all. Either the protected segment never reached its capacity limit on this trace, or demoted entries were not re-referenced. This is a test-coverage question as much as a result.
- Several attributions remain unverified. Source diffs were not inspected, the timing results come from single runs, and all evidence comes from one trace (workload-a). One metadata movement in v003 is not explained by the hypothesis.

## Open questions

- Can the block-grouped counter layout from v003 keep its per-op gain while preserving the original collision pattern, or does the speedup require the collision change?
- Does the protected-capacity branch fire at all on workload-a? If not, the demotion hypothesis was untested rather than refuted.
- Do the timing shifts in v001 and v003 survive repeat runs, and do any of these hit-rate effects hold on a second trace?
- What caused the reported metadata growth in v003 when the hypothesis forbade struct changes?

## Continuation

Pursue the cache-line grouping from v003, but keep the speed gain and restore collision-preserving block selection, or pair it with a compensating counter-width or aging change, before accepting it. Do not spend another cycle on protected-segment turnover until it is confirmed the branch fires on this trace. Hold the seed `const`-table premix from v001 as a neutral layout option, not a speed direction, until repeat runs settle its timing.


## Write

Exactly one file: `cycle-insight.md` in the output dir.

# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0008

## Winners

- v002
- v001
- v003

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0008/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

The cohort tried three changes: halving the protected-segment share so the demotion branch fires more often (v001), breaking sketch-estimate ties in favour of the candidate at admission (v002), and choosing the counter block from the key hash and the column from the per-row seed (v003). All three are labelled Winners, but the variant evidence does not support a clear gain: none raised hit rate on workload-a at size 2048, and each reports a per-op-time or hit-rate regression.

## Patterns

- Hit rate did not rise in any variant. v001 and v002 both dipped slightly, and v003 recovered only a trivial amount of the parent's loss. Admission tie-breaking and protected-capacity changes are not where this trace loses hit rate.
- Protected-segment capacity matters more than protected-tail turnover. Halving the protected share made the demotion branch reachable, and that cost hit rate and per-op time. This partly answers the Cycle 0006 question: the branch can fire, and firing it more often did not pay off.
- Per-op time regressed in every variant, at both get and put. The causes are inferred (extra list moves, extra per-row lookup work, extra evictions on ties), not measured from source.
- The speed gain from Cycle 0006's cache-line grouping depended on the simpler column mapping. Taking the column from the per-row seed did not restore collision spread and gave up the get-path benefit, so the speed-versus-collision trade-off is still unresolved.
- Metadata is essentially flat. The one size movement, a slightly smaller TinyLfuCache in v003, is unexplained.
- Every result comes from one run on one trace (workload-a), and no variant's overlay source was inspected.

## Open questions

- Is there a protected capacity that still exercises the demotion path without losing hit rate, or does any reduction cost hit rate on this trace?
- Does the admission tie case matter on a second trace, or is it inert on workload-a?
- Can the cache-line grouping keep its get-path gain without a per-row column lookup?
- Do the put tail regressions in v001 and v002 survive repeat runs?
- What reduced the TinyLfuCache size in v003?

## Continuation

Drop the tie-break change (v002) and the block-plus-seeded-column split (v003) as directions on this workload: neither moved hit rate, and both added per-op cost. Keep the protected-capacity lever (v001) as the one open thread, but test whether the demotion path can run at a capacity that holds hit rate before exploring protected-tail turnover again. Revisit cache-line grouping only if a collision-preserving block selection can be found that does not add a per-row column lookup.


## Write

Exactly one file: `cycle-insight.md` in the output dir.

# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0009

## Winners

- v001
- v003
- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0009/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

The cohort tried three changes aimed at frequency-sketch collisions: a conservative counter update that increments only the minimum-valued rows (v001), a put-path change that avoids re-reading the victim's frequency (v002), and reshaping the sketch from one fewer row to more columns at the same counter budget (v003). The two sketch-layout changes produced the only hit-rate movement, both small; the put-path change was flat on hit rate.

## Patterns

- Collision spread is the lever that moved hit rate. Both v001 (conservative update) and v003 (fewer rows, more columns) reduce how much colliding keys inflate each other's counts, and both showed small hit-rate gains on workload-a at size 2048. This is the first cycle since Cycle 0005 where a Winner gained hit rate.
- The gains carry a cost in different places. v001 adds an extra pass over a key's rows on every increment and shows slower get and put timings. v003's timing improvement is not explained by its stated change and may be noise or a layout effect.
- The put-path change (v002) left hit rate and metadata flat. Its only signal is a small put p99 shift from a single run, with a get p99 shift it cannot explain, so it is not a usable speed result yet.
- Metadata did not move in any variant. The v003 sketch stays within the baseline memory budget.
- Every result comes from one run on one trace and one cache size. Source overlays were not inspected in this cycle, so mechanisms are inferred from hypotheses and fitness results, and the v003 row and column counts are not recorded.

## Open questions

- Does the v003 row-for-column trade hold on a second trace and at other cache sizes, or is the gain specific to workload-a at size 2048?
- Can the conservative update in v001 keep its collision benefit without the extra per-increment pass over all rows?
- Is the v003 timing gain real, or is it measurement noise?
- Does the put-path victim-read change in v002 show a repeatable put p50 or p99 gain across at least three runs?

## Continuation

Pursue the sketch-collision direction: the row-versus-column trade from v003 is the stronger candidate, so confirm its hit-rate gain on another trace and size before building on it. Hold the conservative update from v001 as a hit-rate option only if it can be made without the extra per-row pass. Drop the put-path victim-read change (v002) as a speed direction until repeat put timings show a consistent gain.


## Write

Exactly one file: `cycle-insight.md` in the output dir.

# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0010

## Winners

- v002
- v001
- v003

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

The cohort tried three changes: collapsing the parent's two-pass conservative sketch update into one loop that reuses admission-check counts (v001), inlining the packed protected-flag accessors on the get path (v002), and narrowing sketch counters to 8 bits to fit more columns in the same byte budget (v003). Only v003 showed a hit-rate movement, and it was small; v001 and v002 left hit rate flat.

## Patterns

- Hit rate moved in only one variant. The narrower-counter, wider-sketch layout (v003) nudged hit rate up slightly, which fits the Cycle 0008 collision-spread direction. Its saturating counters also add a possible admission-resolution cost that was not isolated.
- The single-loop conservative update (v001) improved tail latency for both get and put, which is the first sign that the extra per-increment pass flagged in Cycle 0008 can be removed without losing the collision benefit. The put median went the wrong way, so the gain is not yet clean.
- Inlining the get-side flag accessors (v002) did not produce a get-side gain, and put timings rose in the same run. An edit that should not touch the put path moved it, which points to run noise or an overlay that differs from the description.
- Timing signals are weak across the cohort. Each comes from a single run, and the sources were not inspected, so no timing result here should be read as a speed win.
- Metadata stayed flat in all three variants, so the byte budget held under the column-for-width trade.

## Open questions

- Does the v003 hit-rate gain hold on a second trace and at other cache sizes, and does saturation at narrow counter width cost admission accuracy?
- Does the v001 tail improvement survive repeat runs, and what drives its put median regression?
- Is the v002 get and put slowdown noise, or does the overlay differ from the described change?
- Are the v003 timing drops real, given the hypothesis made no timing claim?

## Continuation

Pursue the v003 counter-width and column trade: test it on another trace or size, and check whether narrow saturating counters hurt admission before building further on it. Keep the single-loop conservative update from v001 as the preferred way to get collision spread without the extra pass, but require repeat runs that clear the put median before accepting it. Drop the get-side accessor inlining in v002 as a speed direction on this workload.


## Write

Exactly one file: `cycle-insight.md` in the output dir.

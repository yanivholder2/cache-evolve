# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0010

## Winners

- v001
- v003

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Three variants on the doorkeeper-style admission incumbent: a larger two-hash aged doorkeeper (v001), a one-slot ghost fingerprint per segment to re-admit the last evicted key (v002), and a deletion-only cleanup of dead writes and reads (v003). All were Winners, but none improved latency. Only v001 raised hit-rate, and v002 and v003 were flat on hit-rate.

## Patterns

- Per-op-time got worse in all three variants, including get latency where get was not meant to change. This is the sixth cycle in a row in which a hot-path or admission edit regressed latency, and the pattern points to layout effects or noise more than real cost.
- v001 confirms the cycle 0006 trade-off. A bigger, aged doorkeeper recovers the lost hit-rate, but it costs put latency through extra probes, cache footprint and periodic resets.
- v002's one-slot ghost is almost free in metadata but almost never matches a reusable key, so it added work for no real hit-rate gain.
- v003 shows that deleting more dead bookkeeping does not help. The cycle 0004 gain most likely came from the filter itself and not from dead-code removal.
- Winner status is again weak evidence, because flat hit-rate and metadata are enough to qualify.
- Variant authors again could not read the source, so the mechanisms above are inferred from fitness vectors.

## Open questions

- What is the run-to-run noise floor for get and put latency on the unchanged incumbent? This has now been asked for four cycles and still has no answer.
- Can a doorkeeper keep v001's hit-rate while staying cheap, for example by fitting in a single cache line?
- Would a larger or better targeted ghost history gain hit-rate, or is it not worth its cost on this trace?

## Continuation

Stop making latency-motivated code edits and drop further dead-store deletion, one-slot ghost and generic doorkeeper-size variants. The next cycle must first measure the noise floor by repeating baseline runs, and profile or use a second trace, before crediting any latency change. If a policy variant is tried, make it a cache-line-sized aged doorkeeper that holds v001's hit-rate at lower put cost.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

Three admission-filter variants on different parents: a one-cache-line aged two-probe doorkeeper (v001), unconditional admission while a segment has free slots (v002), and a tiny direct-mapped fingerprint doorkeeper (v003). All were Winners. Only v001 was a clean improvement: put latency fell and hit-rate held or rose slightly, with metadata flat. v002 gained nothing and regressed latency. v003 cut latency but lost hit-rate.

## Patterns

- Shrinking the doorkeeper to one cache line fixed the cycle 0007 trade-off. Footprint and reset method, not the filtering itself, drove put cost, so a small filter aged often enough to stay sparse can match a large one.
- Cheap put-side wins come from skipping insert and eviction on one-hit keys. A lossy filter without aging (v003) pays for it in hit-rate, as in cycle 0006. Aging and sparsity are what keep a small filter accurate.
- Warm-up loss is not the hit-rate limit. Admitting first touches into free space did nothing, since the cache fills quickly and the filter works almost entirely in the full-segment regime.
- v002's large get and put regression on an untouched or trivially changed path is again most likely layout or measurement noise. The noise floor is still unmeasured, and v001's small get gain and hit-rate gain may be noise too.
- Variant authors for v001 and v002 could not read the source, so the mechanisms are inferred from fitness vectors and the hypotheses.

## Open questions

- What is the run-to-run noise floor for get and put latency on the unchanged incumbent? This has now been asked for five cycles.
- Is v001's hit-rate gain real or within noise?
- Would a cache-line-sized filter with a different aging threshold, or a fingerprint filter with aging, push the hit-rate and put cost trade-off further?
- Does v001's gain hold on a second trace with scans or frequency skew?

## Continuation

Adopt v001 (the one-line aged doorkeeper) as the new incumbent for policy work. Drop warm-up-admission variants and unaged fingerprint tables. Re-measure v001 and its parent with repeated runs to set the noise floor before crediting any latency gain, and check it on a second trace. If a further variant is tried, tune v001's aging threshold or probe count, and avoid hot-path edits to get.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Three small tweaks to the one-line aged doorkeeper incumbent: a lower aging threshold (v001), a single-probe filter with a higher threshold (v002), and an early return in put for resident keys (v003). All were Winners, but none improved latency. Only v001 gained hit-rate, and the gain was marginal. v002 lost a little hit-rate, and v003 held hit-rate flat. Metadata stayed flat in all three.

## Patterns

- Per-op-time got worse in all three, including get latency where get was not meant to change. This is now the seventh cycle in a row in which an edit to the hot path or the admission filter regressed latency. Layout effects or noise are the most likely cause.
- The doorkeeper is tuned out. Aging threshold and probe count are weak levers with only small hit-rate trade-offs. Once both probes share a cache line, the second probe costs almost nothing, so dropping it saves no time and weakens selectivity.
- Skipping filter bookkeeping on resident-key puts bought nothing, so that bookkeeping is not a meaningful cost.
- Winner status is again weak evidence, since flat hit-rate and metadata are enough to qualify.
- Variant authors for v002 and v003 could not read the source, so the mechanisms are inferred from fitness vectors.

## Open questions

- What is the run-to-run noise floor for get and put latency on the unchanged incumbent? This has now been asked for six cycles and is still unanswered.
- Is v001's small hit-rate gain real or within noise?
- Would changes to eviction or promotion, rather than admission, move hit-rate further?
- Do these conclusions hold on a second trace with scans or frequency skew?

## Continuation

Stop tuning the doorkeeper (aging threshold, probe count) and stop editing put or get control flow for latency. Next cycle should first measure the noise floor with repeated baseline runs, comparing against the parent within the same run. After that, aim hypotheses at eviction or promotion policy, and check them on a second trace. Do not credit any latency change without repeats.


## Rejected variants

- v002 — implement:adapter produced no overlay files under /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0010/variants/v002/attempts/03/overlay (3/3 attempts)
  Write at least one changed subject file under overlay/; an empty overlay is indistinguishable from the baseline.

## Write

Exactly one file: `cycle-insight.md` in the output dir.

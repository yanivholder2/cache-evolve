# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0009

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

Three structural changes to the incumbent: a tag-first bucket scan (v001), shared-then-upgrade reader-writer locking (v002), and a small fixed-size doorkeeper ring replacing the in-item second-hit filter (v003). All were Winners. v001 and v002 held hit-rate and metadata flat and made latency worse. v003 showed a large latency drop but lost hit-rate.

## Patterns

- Cycle 0005's finding held for a fifth cycle: structural edits to probing and locking did not pay off. Tag compares cost as much as full u64 key compares, and the upgrade lock paid for two acquisitions on hits that need promotion.
- The bucket scan and the lock mode are now both unlikely to be where hit-path time goes. The benchmark looks effectively uncontended, so reader-writer sharing has nothing to exploit.
- v003 is the only latency gain, and it came with a hit-rate loss. The ring is lossy, so recurring keys were probably evicted from it before their second touch.
- v003's get-latency gain is implausible on its own, because get was not changed. This again points to measurement noise or layout effects, and the noise floor is still unmeasured.
- Winner status is again weak evidence. Flat hit-rate and metadata make a variant a Winner even when latency regresses.
- Variant authors still could not read the source and reasoned from fitness vectors alone, so the mechanisms above are inferred.

## Open questions

- What is the run-to-run noise floor for get and put latency on the unchanged incumbent? This has been asked for three cycles and not answered.
- Is v003's put gain real (skipped insert and eviction work on declined first touches), and could a larger or aged doorkeeper keep that gain without the hit-rate loss?
- Where does hit-path time actually go, if not the scan, the lock or the hashing?
- Does a different trace, such as scans or frequency skew, change any of these conclusions?

## Continuation

Drop further bucket-scan tagging and reader-writer lock variants. Do not edit the hot path blind again. First establish the noise floor by repeating baseline runs, and profile where get and put time go, or evaluate the incumbent on a second trace. If a policy variant is tried, make it a better-sized or aged doorkeeper, judged by whether it keeps v003's put-side savings at the incumbent's hit-rate. Re-measure latency with repeats before crediting any gain.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.

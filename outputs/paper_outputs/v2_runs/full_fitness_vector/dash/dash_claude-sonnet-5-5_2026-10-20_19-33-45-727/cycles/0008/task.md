# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0008

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Three latency-only variants on the minimal bit-packed second-hit filter: deleting more write-only bookkeeping (v001), skipping promotion on a flagged hit (v002), and hoisting hash/index/tag computation out of the locked region (v003). All were Winners because hit-rate and metadata stayed flat. None delivered a convincing latency gain. v001 was at the noise floor, and v002 and v003 made median latency slightly worse.

## Patterns

- Hit-rate and metadata were invariant in every variant, as designed. These variants only changed mechanics, so they say nothing new about policy.
- Subtractive changes ran out of room. The remaining dead stores are too few to measurably help, so the cycle 0004 "subtract only" direction looks exhausted.
- Restructuring the hot path (skipping promotion, hoisting arithmetic out of the lock) again made latency slightly worse. This is the fourth cycle in a row where a hot-path edit to the incumbent did not pay off.
- Regressions again appeared on paths the change did not touch, and tail movements went in both directions. Noise or code layout is the likeliest explanation, and the noise floor has never been measured.
- Second-hit promotion and hash arithmetic are not where hit-path time goes.

## Open questions

- What is the run-to-run noise floor for get and put latency on the unchanged incumbent?
- Where do get and put time actually go (lock acquisition, bucket scan, memory access)? Nothing has profiled this yet.
- Does a different trace, such as one with scans or frequency skew, reveal differences these levers cannot show on the LIRS trace?

## Continuation

Stop editing the hot path blind. Drop further dead-store deletion, promotion-skip and hoisting variants. The next cycle should first establish the noise floor by repeating baseline runs, and profile where get and put time goes. Alternatively, it should evaluate the incumbent on a different trace. Only try structural changes to probing, bucket layout or locking if profiling points to them.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.

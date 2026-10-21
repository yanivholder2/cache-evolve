# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0007

## Winners

- v001
- v003
- v002

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

Three variants targeted per-op-time through mechanics rather than policy: a get-hit lookup/locking cleanup (v001), a put-path free-slot fast path (v002), and a from-baseline re-derivation of the bit-packed second-hit filter with unread bookkeeping stripped (v003). All were Winners, but only v003 genuinely improved. v001 and v002 held hit-rate and metadata flat and made latency worse.

## Patterns

- Hand-optimizing the hot paths of an already-fast incumbent backfired twice. Atomics, early returns, and merged scans or bitmask checks added work or branches that did not pay off. This is the same failure mode cycle 0003 saw with conditional guards.
- Both latency regressions also showed up on paths the change did not touch (put for v001, get for v002). This suggests code-layout effects or measurement noise, so the regressions cannot be firmly attributed.
- v003 re-derived the second-hit filter from baseline and improved both hit-rate and latency at flat metadata. That independently confirms cycle 0001/0002's finding that this filter is the one real win.
- v003's gain mixes the filter policy with removal of baseline per-get statistics and timestamp writes. Those two effects cannot be separated from these results.
- Removing work that the policy never reads helped. Adding mechanisms to speed up existing work did not.
- Insert-path fast paths look weak, probably because buckets are mostly full under this workload.

## Open questions

- Are the v001 and v002 latency regressions real, or run-to-run and layout noise?
- How much of v003's latency gain comes from dropping dead bookkeeping, and how much from skipping the first-hit remove-and-promote?
- Where do get and put time actually go? Nothing has measured this yet.
- Does the cleaned-up baseline route (v003) match or beat the cycle 0002 v001 incumbent?

## Continuation

Treat v003 (minimal bit-packed second-hit filter with dead bookkeeping removed) as the new base, and compare it directly with cycle 0002's v001. Pursue subtractive changes only, such as deleting other unread writes or stats and removing redundant work. Drop further lookup-atomics, early-return and free-slot fast-path variants. Re-measure latency, with repeats, before trusting any result, and consider a different trace for hit-rate questions.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.

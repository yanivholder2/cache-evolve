# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0006

## Winners

- v002
- v001
- v003

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0006/hypotheses/H03.md

## Prior cycles digest

### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

Three variants built on the bit-packed second-hit filter: a 2-bit hit counter in place of the 1-bit flag, a skip-if-already-hot shortcut on the get-hit path, and a CLOCK-style second-chance ref bit for victim selection. All were Winners, but only by negligible hit-rate gains or none. All three gave up latency.

## Patterns

- Metadata stayed flat everywhere. Spare bits can hold a counter or a ref bit, so footprint is no longer the constraint.
- Richer admission or recency signals (counter, ref bit) added almost no hit-rate. The second-hit filter already screens out most one-touch items, so there is little left for them to catch.
- Every variant added hot-path work (per-hit read-modify-write, a conditional guard, or an eviction scan) and every one lost latency. The shortcut meant to remove work also cost latency, because a branch that only sometimes succeeds is not free.
- Hit-rate gains here are within plausible trace noise. The Winner status says little about real improvement.
- Admission and eviction-order levers now look saturated on this trace, after the position levers in cycles 0001 and 0002.

## Open questions

- Is the remaining latency regression real or run-to-run noise? Several explanations rest on inference, because the overlay source was not readable.
- If the hit path is already cheap, where does get and put time actually go (lookup, locking, bucket scan)?
- Does a different trace, such as one with scans or a frequency skew, expose gains these levers cannot show on the LIRS trace?

## Continuation

Treat cycle 0002's v001 (bit-packed second-hit filter, baseline item size) as the base. Drop further admission-signal, recency-bit and hit-path-shortcut variants, because hit-rate is saturated and each costs latency. Aim the next cycle at latency work on the lookup, locking and bucket-scan paths, or at evaluating on a different trace. Repeat the latency measurement before trusting any result.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.

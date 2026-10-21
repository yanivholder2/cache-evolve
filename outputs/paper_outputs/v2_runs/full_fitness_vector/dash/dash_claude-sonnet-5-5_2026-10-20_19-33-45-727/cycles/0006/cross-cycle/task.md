# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three variants probed promotion policy in the DASH segment: two moved only the insert position (uniform MRU, and split cold/hot by tier), and one changed admission by requiring a second stash hit. All three won, each with a modest hit-rate gain on the recency-skewed LIRS trace.

## Patterns

- Every gain was small. Insert position and admission tweaks both help, but none looks like a main driver.
- The position-only variants cost no metadata. The admission-filter variant added a per-item flag that grew the item struct, because there was no spare padding.
- Latency shifts were mixed. They are plausibly noise in the position-only variants, where no work was added or removed. The only mechanically explained speedup came from the admission filter, which skips the remove-and-promote work on a first hit.
- Scan resistance (keeping one-hit-wonders out of the hot tiers) was the shared theme. Uniform MRU (v001), cold first admission (v002) and a second-hit gate (v003) all pursue it in different ways.

## Open questions

- Is the latency change real or noise? v001 and v002 changed no work, so re-measure before trusting it.
- Does the gain from v001's uniform MRU come from stash→target promotion or from the demotion cascades? The two were changed together.
- Can the second-hit filter work without growing the item, for example by packing the flag into spare bits or encoding it in bucket position?
- Do the position and admission gains stack?

## Continuation

Combine the second-hit admission filter (v003) with the tiered insert positions (v002 or v001), and make the filter's flag footprint-free so metadata stays flat. Separate the promotion and demotion position choices to attribute v001's gain. Drop further single-position-only variants unless they are paired with a latency re-measurement.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Three variants followed up on cycle 0001: one packed the second-hit flag into spare bits, one made first stash admissions cold on top of uniform MRU, and one stacked cold demotions on the second-hit filter. All three won. Only the bit-packing variant (v001) gained cleanly. The two position-based stacking attempts gave weak or mixed results.

## Patterns

- Bit-packing the admission flag (v001) was the clear win. It kept the filter's hit-rate benefit, restored the baseline item size, and improved latency. The flag does not need its own field.
- Position and admission levers overlap rather than add. Cold first admission on top of uniform MRU (v002) gave a small hit-rate gain at a latency cost. Cold demotions plus a flag reset on top of the second-hit filter (v003) gave faster operations but slightly lower hit-rate. Each pair targets the same thing, keeping weakly reused items out of the hot region.
- Hit-rate and latency traded off in the position variants. Whichever of the two improved, the other tended to give ground. The latency changes in v002 and v003 are explained only by inference, because the overlay source was not read.
- Insert position alone looks close to exhausted as a lever.

## Open questions

- Is v002's small hit-rate gain real, or within trace noise?
- Does v001's bit-packed filter keep its latency gain when it is combined with a milder form of demotion penalty, such as cold demotion without the flag reset?
- Does the packed flag have enough spare bits to hold a small counter, such as a 2-bit or 3-bit hit count, instead of a single bit?

## Continuation

Treat v001 (bit-packed second-hit filter, baseline item size) as the base for the next cycle. Explore richer admission signals that cost no metadata, such as a multi-bit hit counter or an age or recency bit in the same spare bits, instead of more insert-position tweaks. Drop further stacking of cold-demotion rules on the filter, and drop cold first admission, unless a latency re-measurement shows the effect is real.


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

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>

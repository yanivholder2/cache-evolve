# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three single-point tweaks to the baseline S3-FIFO were tried: a lower small-to-main promotion threshold, a smaller small-queue ratio, and ghost recycling of main evictions. The first was lost to an infrastructure failure. The two Winners were the small-queue ratio change and the ghost-recycling change.

## Patterns

- The v001 rejection came from a Maven 429 while downloading the Caffeine jar. It says nothing about the promotion-threshold idea, which is still untested.
- Both Winners report large per-op latency drops, but each variant's own analysis says the change has no mechanism for that. Treat the drops as run-to-run noise.
- Only the small-queue ratio change (v002) has a plausible hit-rate gain, and it is modest. It is a constant-only change that adds no work and leaves metadata unchanged.
- The ghost-recycling change (v003) lowered hit rate. Main evictions crowd out small-queue demotions in the shared ghost, which weakens its one-hit-wonder filter. It likely won through the noisy latency numbers, not on merit.
- All evidence comes from one trace (LIRS workload-a) at one cache size, so the gains are not shown to generalize.

## Open questions

- Is the latency improvement reproducible, or is the fitness signal noisy enough to promote regressions?
- Is the best small-queue ratio workload-dependent? Is 10% better than other values such as 5% or 15%?
- Does the lower promotion threshold (freq>=1) help, now that v001's infrastructure failure has hidden it?

## Continuation

Build on v002's 10% small queue and retry the promotion-threshold hypothesis from H01, since it never got evaluated. Drop ghost recycling of main evictions unless it is redesigned, for example with a separate ghost or without the freq=1 head start. Next cycle should sweep the small ratio and use more traces or repeated runs to separate real latency effects from noise.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Three variants were tried: a lower promotion threshold (freq>=1) on the 10% small queue (v001), a 15% small-queue ratio (v002), and a separate bounded main-eviction ghost (v003). Only v003 shows a real, modest hit-rate gain. v001 and v002 were slight hit-rate regressions, so they probably won through noisy latency numbers.

## Patterns

- Both constant-level tweaks to the small-queue path (looser promotion, larger small queue) lost hit rate against the 10% parent. This points to 10% with the baseline threshold being a local optimum on this trace.
- The small queue's filtering role appears to matter more than quick promotion or extra residence time. Anything that lets more one-hit-wonders into main, or takes capacity from main, hurts.
- v003 supports the diagnosis from Cycle 1. Main-eviction recycling failed because it diluted the shared demotion ghost, and a separate ghost keeps the filter intact and gives a small positive gain. The cost is a struct-level metadata increase and a slower typical put.
- Latency shifts appeared again in all three variants, including edits that add no hot-path work. Every variant insight attributes them to noise. Latency is still not a trustworthy selection signal.
- All evidence still comes from one trace (workload-a) at one cache size.

## Open questions

- Is v003's hit-rate gain larger than run-to-run noise, and does it hold on other traces and sizes?
- Is the 25% main-ghost size a good choice? Is it worth the metadata and put-latency cost?
- Is the small-ratio optimum below 10% (the 5% point was not tested this cycle)?

## Continuation

Treat v003's separate main-eviction ghost as the leading direction and tune it: ghost size, and whether it needs a freq head start on re-entry. Drop the freq>=1 promotion threshold and larger small-queue ratios (15% and up), since both regressed. If anything is swept further, test 5% and below. Evaluate on more traces and repeated runs, and judge on hit rate rather than latency.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

The cycle tried a behavior-preserving scan refactor (v001), a smaller 5% small queue (v002), and a cap on main-eviction rotation (v003). v001 is the only change with a credible latency gain. v002 gives a tiny hit-rate gain. v003 slightly lowered hit rate and has no measurable speed benefit, so it probably won through noisy latency numbers.

## Patterns

- The cycle moved away from the ghost-tuning direction that Cycle 0002 recommended. None of the three variants touched the main-eviction ghost.
- v001 is the only change with a mechanism for its latency gain: it removes redundant linear scans, and hit rate and metadata are unchanged. The cost is that it deleted several tests on promotion and eviction behavior.
- The small-queue ratio shows diminishing returns. Moving from 10% to 5% helped only marginally, so the curve is flattening.
- The rotation cap in v003 is unnecessary because the frequency counter saturates at a low value, so rotation is already effectively bounded. It trades hit rate for nothing.
- Latency noise continues. The `get` path moved even in variants that did not touch it. Only changes with a clear mechanism, like v001's, should be credited.
- All evidence still comes from one trace (workload-a) at one cache size.

## Open questions

- Is v002's gain real, or noise? Does 5% hold on other traces?
- Can membership checks use an index such as a hash map? Linear scans still dominate `put` cost.
- Can the tests v001 removed be restored without losing its gains?

## Continuation

Keep v001's scan consolidation, restore the deleted tests, and pursue index-based membership to cut `put` cost. Keep 5% as the small-queue ratio only if repeated runs and other traces confirm it, and stop sweeping that knob. Drop the main-eviction rotation cap. Go back to tuning the separate main-eviction ghost from Cycle 0002, and judge on hit rate over multiple traces rather than on latency alone.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

The cycle tried hash-index membership lookups (v001), a main-eviction ghost that re-admits keys into main with a freq head start (v002), and a doubled ghost bound (v003). Only v001 has a real gain: lower typical `put` and `get` cost with hit rate unchanged. v002 won with no improved dimension. Its extra ghost scan and its lost hit rate point to noise as the reason it was picked.

## Patterns

- v001 is the only variant with a mechanism for its gain. The index removes linear scans and preserves the hit/miss sequence exactly. Its cost is a much larger whole-cache footprint that the fixed per-entry `metadata-size` figure does not show.
- v001's `put` tail barely improved. The remaining tail cost likely comes from eviction and ghost-trimming work, not membership checks.
- Both ghost-side changes (v002, v003) lowered hit rate on workload-a. A freq head start for main-evicted keys and a longer ghost history both let marginal objects into main. This is the third cycle in which anything that loosens main admission has hurt.
- v002 contradicts Cycle 0002's reading of the separate main ghost as a small gain. Its head-start variant regressed, and the extra linear scan undid scan savings. The earlier gain may have been noise or may depend on details of the design.
- Baseline ghost sizing looks near a sweet spot. Reuse beyond main capacity is rare on this trace.
- Latency shifts that have no mechanism, such as v003's tail, are noise again. Latency is still unreliable for selection unless the change has a mechanism.
- Everything still comes from one trace (workload-a) at one cache size.

## Open questions

- Does v001's index hold its gain across traces? What is its real memory cost?
- Can the `put` tail be reduced by making eviction and ghost trimming cheaper?
- Is any form of special handling for main-evicted keys worthwhile? If so, it needs an O(1) tagged-ghost design with evidence of faster reuse.
- Do the earlier small-ratio and ghost results survive repeated runs and other traces?

## Continuation

Keep v001's index-based membership as the new base. Next, target `put` tail cost in eviction and ghost trimming, and account for the index's memory footprint explicitly. Drop the main-ghost freq head start, the larger ghost bound, and further ghost-size tuning. Revisit main-evicted-key handling only as an O(1) tagged variant of the existing ghost. Validate on multiple traces and repeated runs, and judge on hit rate.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Three independent experiments from different parents: an O(1) ring-buffer ghost (v001), a CLOCK-style pop/push rewrite of main eviction (v002), and a 5% small queue from baseline (v003). Only v001 has a mechanism for its gain (lower `put` cost at unchanged hit rate). v003 shows a modest hit-rate gain, but its large latency drop is implausible for a one-constant edit. v002 was neutral and probably won on noise.

## Patterns

- Both behavior-preserving rewrites (v001, v002) kept hit rate and eviction decisions identical. Only the one that targeted ghost maintenance (v001) moved `put`. The other, on main eviction (v002), did not.
- Together these confirm that the `put` tail comes from ghost maintenance, not from main-queue eviction or membership checks.
- The per-op-time headline tracks `get` p50, so a variant that improves `put` can still look worse on the headline (v001). `get` shifts appeared in variants that never touched `get` (v001, v002), so they are noise.
- The 5% small ratio now has a second hit-rate gain on workload-a, from a clean baseline. It is consistent in direction but still small, and it is single-trace.
- The latency drop in v003 has no mechanism and should not be credited to the ratio change.
- Whole-cache footprint grows slightly with each new structure (ring bookkeeping here), and the per-entry `metadata-size` metric does not show it.

## Open questions

- Does the 5% small-ratio gain survive repeated runs and other traces and sizes?
- Does v001's ring ghost combine with the index and scan work from earlier cycles without a `get` regression?
- What is left in the `put` tail now that ghost trimming is O(1)? Candidates are hash-map operations and the small-to-main move.
- Is the `get` p50 shift real or a layout effect?

## Continuation

Adopt v001's ring-buffer ghost as the base for `put` work, and stop reworking main-queue eviction (v002 shows nothing to gain). Treat 5% as the small ratio and stop sweeping it unless repeated runs on other traces disagree. Next, re-measure `get` p50 with repeated runs, and look for the remaining `put` tail cost outside ghost trimming and eviction. Judge on hit rate and on latency changes that have a clear mechanism.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cycle tried three behavior-preserving speed edits: a cheaper hasher for the index (v001), a single location-tagged index replacing separate small/main/ghost lookups (v002), and a small micro-optimization of the frequency store, small-to-main move, and deque pre-sizing (v003). Hit rate was identical for all three. Only v002 has a large, mechanism-backed gain, a big drop in `put` cost.

## Patterns

- Cycle 0005 left the `put` tail unexplained. This cycle shows it was hash-map traffic from multiple membership probes, not queue manipulation. Cutting map operations per `put` (v002) was the first change to move it substantially.
- v001's cheaper hasher gave a small, plausible `get` gain and a smaller footprint. Its `put` tail did not improve, which is consistent with hashing not being the `put` bottleneck.
- v003 produced only small shifts within noise. The frequency store, small-to-main move, and deque growth are not significant costs, so the micro-optimization direction is exhausted.
- v002 has two costs. The fixed struct footprint roughly doubled, and the `get` p99 got worse while p50 improved. Both likely come from one larger index that now also holds ghost keys.
- Pure hit-rate-neutral rewrites keep winning on latency while hit rate stays flat across all cycles. Hit-rate improvement has not been tried since the 5% small ratio.
- The evidence is still one trace at one cache size. Small `get` shifts remain unreliable without a mechanism.

## Open questions

- Can v002's footprint and `get` tail be reduced, for example by keeping ghost keys out of the main index or by better table sizing?
- Does the Fx-style hasher stack on top of the tagged index, and does it help or hurt the `get` tail there?
- What remains in the `put` tail after the tagged index?
- Does the gain hold on other traces and repeated runs?

## Continuation

Adopt v002's tagged index as the new base for `put`. Next, try combining it with v001's cheap hasher and pre-sizing, and look for ways to cut its memory overhead and `get` p99. Drop further micro-optimizations like v003's, and drop more membership or hasher work unless it follows from the tagged index. Then validate on multiple traces and repeated runs, and consider returning to hit-rate ideas that don't loosen main admission.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

The cycle followed up on the tagged index from Cycle 0006. It tried a separate ghost map to keep ghost keys out of the main index (v001), a one-retry "recent-main" second chance in small-queue eviction (v002), and an Fx-style hasher with map pre-sizing on the tagged index (v003). v001 and v003 are behavior-preserving speed wins with mechanisms. v002 is nearly neutral on hit rate and slightly slower, so it probably won on noise.

## Patterns

- Both useful variants cut hash-map cost, the lever Cycle 0006 identified. v001 helped `put` mainly. v003 helped `get` mainly. They touch different parts of the index and look complementary, but they have not been combined.
- v003 confirms that hashing cost dominates `get` on the tagged index. It also answers the open question from Cycle 0006: the cheap hasher stacks on the tagged index and improves the `get` tail there.
- v003 traded a better `put` median for a much worse `put` tail. The likely causes are oversized pre-sizing and weak key mixing, but the cause was not isolated, because the two changes were bundled.
- v001's expected footprint reduction could not be checked. The metadata metric counts only inline struct bytes, not heap. Every added structure (second map, ring) shows up as small struct growth, and heap effects stay invisible.
- v002 is a third loosening-style tweak to main-related admission that gave almost nothing. Few small victims were both freq-0 and recently evicted from main. This reinforces that eviction-policy tweaks have little room on this trace.
- Hit rate was flat for every variant again. Hit-rate gains have not appeared since the 5% small ratio.
- Evidence is still one trace at one size.

## Open questions

- Does v001's separate ghost map combine with v003's hasher without bringing back the `get` tail or `put` tail problems?
- Which part of v003 caused the `put` tail regression: the pre-size or the weak hash mixing?
- What is the real heap footprint of the tagged index plus separate ghost map?
- Do the gains hold on other traces and repeated runs?

## Continuation

Adopt v001's split ghost map and v003's hasher as the combined base, but separate v003's pre-sizing: keep the hasher and test a tighter pre-size or a better-mixing finalizer to fix the `put` tail. Drop the recent-main second-chance filter and further small-queue tie-breaks. Measure heap footprint explicitly, and validate on multiple traces and repeated runs before more latency work. Then consider hit-rate ideas that do not loosen main admission.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

The cycle tried two hasher/pre-sizing follow-ups on the tagged index with split ghost map (a multiply-xorshift hasher in v001, and a tight capacity pre-size plus an avalanche finalizer in v002). It also tried a smaller small-queue ratio, 3% instead of 5% (v003). The only Winner with a clear signal is v003, which gave a tiny hit-rate gain. v001 and v002 hold hit rate flat and trade latency, so their wins are weak and probably noise-assisted.

## Patterns

- Hasher work is exhausted. A cheaper hasher (v001) made everything slightly slower. A stronger finalizer with tight sizing (v002) fixed the `put` tail but gave back much of the `get` gain. Together they show a hash-quality versus hash-cost tradeoff with no free win, and hashing is no longer the main bottleneck.
- Tail and median trade off in v002. Tight sizing and mixing remove cold-insert outliers, while the added finalizer and fuller table raise the median. The two effects were bundled again, so their split is still unknown.
- The small-ratio lever is saturating. The step from 5% to 3% gave a much smaller hit-rate gain than the earlier steps, and the latency shifts in v003 have no mechanism, so they are noise.
- Hit-rate-neutral latency tweaks now produce mixed results. Variants selected as Winners without a mechanism-backed gain are increasingly likely to be noise.
- Evidence is still one trace at one size.

## Open questions

- Does the 3% small ratio hold up on other traces and sizes, or does it hurt workloads with more mid-reuse objects?
- Is a lighter finalizer or modest slack enough to keep the `get` gain and still fix the `put` tail?
- Do the earlier latency gains (tagged index, split ghost map, Fx hasher) survive repeated runs, now that the base is more stable?

## Continuation

Stop hasher, finalizer, and small-ratio sweeps. Pick one hasher config (the Cycle 0007 Fx hasher with moderate slack, or at most a light finalizer) and keep it. Keep 3% only if it holds on other traces. The next cycle should move to hit-rate ideas that change promotion or ghost policy without loosening main admission, and validate on multiple traces with repeated runs, including a heap footprint measurement.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

The cycle tried a keyed-lookup refactor that replaced a linear queue scan with one map probe (v001), ARC-style adaptive small-queue sizing (v002), and a lower main frequency cap (v003). Only v001 has a mechanism-backed gain, a tail-latency win with hit rate unchanged. v002 and v003 were selected as Winners without a real gain.

## Patterns

- v001 won by removing an O(n) scan, so the gain shows in the `get`/`put` tails and not in the medians. The earlier cycles' map-cost findings are consistent with this.
- v002's feedback loop did nothing on this trace. Hit rate matched the fixed parent, and the added counters, window check and drain loop only added state and latency. Adaptive sizing is not worth its cost without evidence that the ghost-hit ratio leaves its dead band.
- v003 traded a slightly lower hit rate for a marginally shorter eviction tail. The frequency cap of 3 protects reused main entries, so lowering it is not free.
- Two of three Winners are likely noise-assisted. This continues the Cycle 0008 trend: selection without a mechanism-backed gain is unreliable.
- Policy-parameter tweaks (small ratio, frequency cap, adaptive sizing) are now saturated or harmful on this trace.
- Evidence is still one trace at one cache size. v001's analysis was done without reading the sources, so it is weaker than usual.

## Open questions

- Does v001's single-lookup structure conflict with, or duplicate, the tagged index and split ghost map from Cycles 0006–0007? The two lines of work need to be reconciled.
- Does the adaptive-sizing signal ever leave its dead band on other traces or sizes?
- Do the latency gains survive repeated runs and other traces, and what is the real heap footprint?

## Continuation

Keep v001's keyed single-lookup as the base, after confirming it is compatible with the tagged index and Fx hasher. Drop adaptive small sizing and frequency-cap sweeps, and stop parameter tuning generally. Next cycle should try promotion or ghost-policy changes that do not loosen main admission, and validate on multiple traces with repeated runs and an explicit heap measurement.


### Cycle 0010

# Cycle 0010 Insight

## Cohort outcome

The cycle tried three small refinements: zeroing frequency on small-to-main promotion (v001), age-gating ghost re-admission to the newer half of the ghost ring (v002), and reusing one computed hash across the resident, ghost and insert probes in `put` (v003). None has a mechanism-backed gain. All three were selected as Winners, but v001 is neutral, v002 gained only a tiny amount of hit rate at a latency cost, and v003 hurt.

## Patterns

- All three Winners are probably noise-assisted. This continues the trend since Cycle 0008 that selection without a mechanism is unreliable.
- Two variants targeted main-admission quality (freq reset, stale-ghost gating). Both showed that weak main entries are not a significant source of loss on this trace. Hit rate barely moved.
- Both latency-oriented or bookkeeping changes (v002's sequence counter, v003's hash carrying) added per-entry state and slowed things, mostly in the `put` tail. This repeats the Cycle 0009 lesson that added state costs more than it returns here.
- v003 shows hashing small integer keys is not the `put` tail cost. That closes the hashing line, which Cycle 0008 had already called exhausted.
- Analyses for all three variants were written without reading the sources, so the mechanisms are inferred and weaker than usual.
- Evidence is still one trace at one cache size.

## Open questions

- Is the trace simply near its ceiling for this policy family, so that small policy tweaks cannot show up?
- Is the `get` slowdown seen in v002 real, or just run-to-run variance on this harness?
- Do any of the earlier latency gains (tagged index, split ghost map, keyed lookup) survive repeated runs?

## Continuation

Drop freq-reset, ghost age-gating, and hash reuse. Do not add per-entry state or counters. Stop micro-tweaks to admission and hashing. The next cycle should first establish a repeated-run noise floor and test on other traces and cache sizes, with an explicit heap measurement, then pick only hypotheses with a clear mechanism, such as a structural change to promotion or ghost policy that can move hit rate on a different workload shape.


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

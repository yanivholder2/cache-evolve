# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0007

## Winners

- v001
- v003
- v002

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0007/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.

# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0008

## Winners

- v003
- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0008/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.

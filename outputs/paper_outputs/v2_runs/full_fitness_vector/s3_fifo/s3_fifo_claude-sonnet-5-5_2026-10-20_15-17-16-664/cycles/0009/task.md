# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0009

## Winners

- v001
- v003
- v002

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0009/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.

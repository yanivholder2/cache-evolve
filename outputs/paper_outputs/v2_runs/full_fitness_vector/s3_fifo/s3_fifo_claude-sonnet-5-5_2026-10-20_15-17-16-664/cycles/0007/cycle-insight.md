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

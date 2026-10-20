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

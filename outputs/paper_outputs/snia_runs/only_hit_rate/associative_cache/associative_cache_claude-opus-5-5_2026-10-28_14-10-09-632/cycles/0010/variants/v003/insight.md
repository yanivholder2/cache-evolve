# Variant v003 Insight

## Helped

- (none) Neither 131072 cell improved. The 16384 cells are bit-identical to v001, which is the expected null result: the `halving`-gated `crp_window` still gives 4 slots in 128-slot buckets, so the size gate isolates the change exactly as intended.

## Hurt

- hit-rate (web_0 @ 131072): Widening the correlated-reference window from 16 to 32 slots in 1024-slot buckets moved this cell the wrong way, so the hypothesis's main claim is refuted. Slots 16–31 of the 256-slot probation ring evidently hold a meaningful share of web_0's *real* reuse. A re-reference that arrives that soon after insertion was enough to earn promotion under v001. Now it is discarded as a burst, so those blocks age out of probation with counter 0 and are evicted.
- hit-rate (hm_0 @ 131072): The drop here is tiny, a rounding-level move, but it lands just under the refutation threshold. That suggests 32 slots is also at or past the useful edge for hm_0 in large rings. In practice the window is neutral-to-harmful for hm_0, not a source of gain.

## Takeaway

The large-bucket correlated-reference window is already at or past its optimum at 16 slots (1/16 of the probation ring). Doubling it to 1/8 filters out genuine early reuse on web_0 instead of more bursts, so the cycle 0008 gain does not keep growing with window width. Together with cycle 0009 v002's result for small buckets, this means window width is used up at both sizes: about 16 absolute slots is the most either bucket size tolerates. The open option is a *narrower* large-bucket window (e.g. 8–12 slots), not a wider one.

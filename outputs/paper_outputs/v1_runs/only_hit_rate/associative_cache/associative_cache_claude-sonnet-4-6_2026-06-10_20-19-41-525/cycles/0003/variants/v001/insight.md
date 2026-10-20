# Variant v001 Insight

## Helped

- hit-rate: Inserting new items at the midpoint (`len / 2`) rather than the MRU tail keeps fresh cold entries out of the fully-protected back of the vector, nudging them slightly closer to the eviction front and producing a small incremental gain on the scan-heavy workload-a trace.

## Hurt

## Takeaway

Stacking mid-point insertion on top of cold-first ClassicLRU eviction produced only a marginal hit-rate improvement, revealing that the two mechanisms are not strongly additive: cold-first eviction already targets counter==0 items regardless of their position in the vector, so insertion site has little influence over which items actually get evicted. Mid-point insertion's scan-pollution benefit is largely pre-empted by the counter-based eviction selection inherited from the parent, with only a residual gain from edge cases where all bucket items are cold and LRU position becomes the tiebreaker.

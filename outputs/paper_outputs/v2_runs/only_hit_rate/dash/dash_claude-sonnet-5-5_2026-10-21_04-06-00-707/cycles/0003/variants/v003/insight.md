# Variant v003 Insight

## Helped

- (none) No dimension improved. Widening the probe to the next and previous buckets before using the stash did not retain more useful keys.

## Hurt

- hit-rate: Slightly below the parent. Placing keys in a second neighbour bucket delays eviction, so colder entries stay resident longer and crowd out hotter ones. The eviction decision becomes less tied to recency.
- per-op-time: Not reported in this variant's fitness vector. The extra bucket probe on insert, `get` and `promotion_get` adds work on the miss path, but this change cannot be confirmed from the eval output.
- metadata-size: Not reported in this variant's fitness vector, so no effect can be claimed either way.

## Takeaway

In this Dash segment the hit-rate bottleneck is the eviction policy, not bucket-capacity pressure. A wider probe neighbourhood only postpones eviction, and it lets stale entries survive. Any later attempt to use the neighbourhood should keep recency ordering, and it should record the per-op-time and metadata-size dimensions so the lookup cost of the extra probes is visible.

Caveat: I could not read the overlay source in this session, so the mechanism above is inferred from the hypothesis and the fitness vectors.

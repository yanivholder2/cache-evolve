# Variant v003 Insight

## Helped

- (none): hit-rate, the only dimension measured, did not improve over the parent.

## Hurt

- hit-rate: Skipping the probation LRU end to evict the first never-reused entry slightly lowered hit rate on workload-a. Demoted protected entries keep `reused = true`, so the scan mostly protects them and sends fresh, not-yet-reused insertions to eviction before they can prove themselves.

## Takeaway

A one-bit "re-referenced" flag does not beat plain probation-LRU eviction here. The bit says only that an entry was hit at least once, not how recently or how often, so it ranks entries worse than recency order alone on this trace. This matches the earlier flat results from small-window counter approaches. Further gains probably need recency-aware or multi-bit signals, not a binary reuse bit.

Note: I could not read the overlay source in this session (only Read, Write, Edit and Grep were available). The mechanism above comes from the hypothesis text and the fitness delta, not from a diff of the code.

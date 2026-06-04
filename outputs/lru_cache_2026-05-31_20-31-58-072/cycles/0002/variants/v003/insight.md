# Variant v003 Insight

## Helped

- hit-rate: Splitting the single arena-list into an 80/20 probationary/protected pair lets second-touch keys sit in protected while the lirs:multi1 scan churns only the small probationary tail, so working-set entries accumulate hits that a pure LRU at the same capacity would have evicted on the next miss.

## Hurt

- hit-rate: Carving the arena into a 20% probationary slice means the very first hit on a key has only `capacity / 5` slots of grace before the scan flushes it — keys touched once and re-referenced just outside that window never reach protected at all, which caps the upside on traces whose re-reference distance exceeds the probationary capacity.

## Takeaway

This variant proves the SLRU mechanism is structurally sound on the baseline arena: promotion, demotion, and probationary-only eviction all hold while preserving in-place slot reuse, and the policy registers a meaningful hit rate on the scan-heavy lirs:multi1 trace. However, the parent's fitness vector was sampled at sizes 16/32 while this run measured size 512, so the matched-size SLRU-vs-LRU delta the hypothesis set out to settle is still not actually established from these two vectors — what's been shown is that the implementation runs and yields hits, not that 80/20 SLRU beats pure LRU at the same capacity.

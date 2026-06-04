# Variant v001 Insight

## Helped

- metadata-size: The third pass reuses the existing `lfu_counter` field and adds no per-item or per-bucket state, so per-item bytes stay identical to v003.

## Hurt

- hit-rate: Sacrificing every `lfu_counter == 1` slot before falling back to LRU-front turns single-hit items into second-class victims even when they sit at the back of the bucket; on this trace the once-hit "lukewarm" tier contains genuinely useful re-references, and demoting them below stale-but-twice-hit items at the front evicts the wrong slot more often than v003's simpler two-pass.

## Takeaway

A stricter "must survive two hits to be protected" filter is not a free refinement of v003: inside a fixed-capacity bucket the `lfu_counter == 1` tier overlaps heavily with recently-re-referenced items at the back, and skipping them in eviction order overrides ClassicLRU's recency signal in cases where recency was still the better predictor. Finer counter-based gradations on top of ClassicLRU need to respect bucket position, not just counter value, or they erode the recency information that the back-of-bucket move was supposed to preserve.

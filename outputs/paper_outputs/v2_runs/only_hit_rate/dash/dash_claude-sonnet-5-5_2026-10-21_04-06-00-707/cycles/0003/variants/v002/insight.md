# Variant v002 Insight

## Helped

- hit-rate: The admission filter in the DashSegment insert path (skip a key on its first miss once the segment is full enough to need eviction, admit on the second sighting) improved hit-rate on workload-a over the parent. One-hit-wonders no longer evict entries that get reused.

## Hurt

- None measured. hit-rate is the only reported dimension. The added per-segment filter state (up to 256 bytes) and the extra hashing on the miss path were not evaluated, because no metadata-size or per-op-time dimension was reported. I could not read the overlay sources, so this rests on the hypothesis text and the two fitness vectors.

## Takeaway

Gating eviction-causing inserts behind a second-sighting check pays off on this trace, so a meaningful share of its misses are single-use keys. This is the first variant in this lineage that changed which keys get admitted, as opposed to where they are placed, and it moved hit-rate where the parent's insert-position change did not. The cost side, filter memory and per-miss work, is unmeasured, so the gain is not yet shown to be worth it.

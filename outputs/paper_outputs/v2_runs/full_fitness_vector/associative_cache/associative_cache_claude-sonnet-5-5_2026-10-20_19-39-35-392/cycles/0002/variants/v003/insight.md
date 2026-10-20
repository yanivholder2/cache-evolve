# Variant v003 Insight

## Helped

- hit-rate: Inserting new keys near the LRU end (about index len/4) while promoting to MRU only on a hit makes one-hit-wonders the first eviction candidates, so re-referenced entries in the wide buckets survive scans.
- per-op-time: Both get and put latency fell relative to v001. The likely cause is that the insertion point changes how far elements shift or move within a bucket, but I could not inspect the source diff, so this is unconfirmed and may be partly measurement noise.

## Hurt

- metadata-size: No change. The hypothesis added no per-slot metadata, so this dimension neither helped nor hurt.

## Takeaway

Position-only scan resistance (LIP/BIP-style insertion) on top of v001's wide buckets gave a clear hit-rate gain without extra state. This supports the hypothesis that the insertion path, not the hit path, is where recency policy gains are found. The gain cost nothing in metadata and did not slow the cache down.

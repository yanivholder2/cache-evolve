## Recurring wins

- Lean per-op mechanics keep paying off independently of policy: hash once per op, contiguous per-bucket storage, in-place policy updates. Every winner in both cycles has built on them.
- Merged buckets with one shared eviction policy are the base that works. Each key may use the whole bucket, which recovers hit rate and removes partner-path tail latency.
- Taking work out of the put path pays off. Contiguous storage first, then removing wall-clock reads and shrinking `Item`, each gave clear put and tail-latency gains.
- Shrinking metadata (logical ticks instead of `Instant`) was the first change to move metadata-size, at no hit-rate cost.

## Recurring failures

- Placing a key outside its single hashed bucket (two-choice, cuckoo, cross-bucket victim areas) always fails P2 slot_legality. Do not propose it again.
- Split home/partner halves inside a bucket lowered hit rate in both cycles. The bookkeeping cancels the flexibility, and hot halves push out their partner's items.
- Get p50 keeps getting worse as policy and layout change. Full-bucket scans and memmoves of large items on hit and evict push cost onto the median read. No cycle has fixed this yet.
- Explaining latency by guesswork has been wrong before. Removing the per-hit `Instant::now()` was expected to fix get p50 and did not. Attribute regressions only with care.

## Open questions

- Plain LRU vs SLRU on the same combined base (merged bucket + 24-byte `Item`): which wins on hit rate? They have never been measured side by side.
- Is SLRU's 3/4 protected fraction close to optimal?
- Can a compact tag/fingerprint array, or a ring/offset layout that avoids front-of-Vec shifts, bring back the get median without losing hit rate?
- Can the timestamp field be dropped entirely under position-ordered policies to shrink metadata further?
- Would frequency-aware or admission-filtered policies inside the single-bucket rule add hit rate beyond SLRU?

## Per-policy notes

- Split-half LRU (home/partner): passes gates only when pairs are merged into one physical bucket. Hit rate stays below baseline. Retired.
- Plain LRU over merged bucket: hit rate above baseline. Get p50 cost comes from scans and shifts. Does not need the timestamp.
- SLRU (probation/protected): the clearest hit-rate gain so far, which suggests the trace rewards scan resistance. Segment membership is stored in the existing boundary, so it adds no metadata. Same get-median cost as plain LRU. Does not need the timestamp.
- Timestamp-based policies: wall-clock reads on the hot path hurt put and tail latency. Use logical ticks if any policy needs recency stamps.

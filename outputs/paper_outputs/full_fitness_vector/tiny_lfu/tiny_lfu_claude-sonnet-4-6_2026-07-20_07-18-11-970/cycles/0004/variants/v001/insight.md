# Variant v001 Insight

## Helped

- `hit-rate`: Splitting rows 0–1 and rows 2–3 across two independently seeded hashes reduced cross-row collision correlation in the count-min sketch, making frequency estimates slightly more accurate and nudging the admission filter toward better eviction decisions.

## Hurt

- `per-op-time`: Adding a second `seeded_hash` call doubled the hashing work compared to v002's single-hash scheme; while the p50 median held steady, both get and put p99 tail latencies climbed, suggesting the extra hash invocation shows up under contention or in cache-miss heavy paths rather than on warm hits.

## Takeaway

Introducing a second seeded hash recovered the sketch independence that v002's single-hash bit-slice approach had sacrificed, producing a small but real hit-rate gain. The cost is that p99 tail latency regressed: the two-hash scheme sits between the original four-hash baseline (best independence, most hashing) and v002 (least hashing, weakest independence), confirming that row independence and hash count are genuinely in tension and that the p50 median is too coarse to capture where the extra hash overhead actually lands.

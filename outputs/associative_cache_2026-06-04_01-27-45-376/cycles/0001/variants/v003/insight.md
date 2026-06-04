# Variant v003 Insight

## Helped

- hit-rate: The two-pass `eviction_candidate_index` evicts `lfu_counter == 0` slots first, so one-shot keys that arrived in a burst get killed before genuinely re-used items at the back of the bucket are pushed out.

## Hurt

## Takeaway

Inside a fixed-capacity bucket, recency order alone wastes slots on never-touched keys — overlaying a "must survive one hit to be protected" filter on top of ClassicLRU's existing ordering reuses the already-present `lfu_counter` field (no metadata cost) and turns the front of the bucket into a churn-absorbing zone. Bucket-movement in Dash benefits from any cheap signal that distinguishes one-shot arrivals from re-referenced items, since within a bucket the policy has no other escape valve.

## Recurring wins

- Intra-bucket policy changes — victim selection and admission position —
  have been the high-leverage surface; the strongest gain so far came from
  replacing pure-LRU victim choice with a frequency-first rule.
- Levers that shield the reusable working set from one-hit-wonders and scan
  traffic (frequency-aware eviction, probationary front admission) have all
  lifted hit rate.

## Recurring failures

- Coarsening bucket geometry (widening buckets) gave the smallest gain and
  appears collision-bound — diminishing returns from further widening.

## Open questions

- Do the orthogonal levers compose (frequency-based victim selection plus
  probationary admission) or do they overlap?
- Is the O(bucket_size) min-frequency scan affordable once per-op time is
  measured, or does its cost erode the hit-rate win?
- Is the dominant workload pressure scan traffic or capacity collision?
- Fitness has tracked only hit-rate; the per-op-time / scan cost side of
  every trade remains unmeasured.

## Per-policy notes

- LRU: pure-LRU victim choice is beatable; repurposing the dormant
  `lfu_counter` as a live hit count to drive frequency-first eviction was the
  winning move.
- Resident set, routing, capacity invariants, and the `EvictionPolicyKind`
  enum / JNI ordinal mapping have stayed untouched in winning variants —
  gains so far live entirely inside the bucket.

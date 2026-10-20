# Variant v002 Insight

## Helped

- **hit-rate**: Replacing the parent's conditional swap (only promote when `lfu_counter` beats the neighbor) with an unconditional single-step swap toward the head converts the policy from frequency-gated promotion to pure recency promotion, ensuring every accessed item immediately advances one slot — this produces colder tail candidates on workload-a than the soft-LFU ordering, which could strand recently-accessed items mid-list when their counter was still below their neighbor's.

## Hurt

## Takeaway

The primary hypothesis — that removing `lfu_counter` writes would lower per-op-time — did not materialize; latency stayed pinned at the 100ns measurement floor, suggesting the benchmark's timer resolution masks any cache-line write savings at this bucket size. What the change did reveal is that unconditional recency promotion (always advance one step) outperforms conditional frequency-gated promotion (advance only when hotter than neighbor) on this workload: the soft-LFU ordering in v003 left recently-reused items stranded below cold neighbors, whereas v002's always-swap rule keeps the tail genuinely cold.

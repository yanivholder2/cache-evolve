# Variant v002 Insight

## Hurt

- **hit-rate**: Tail-preferential eviction with simple recency flags failed to preserve hot items; the 64-operation reset interval is too coarse-grained to capture temporal locality patterns in the Zipfian workload, causing the backward scan to evict items that would otherwise survive under pure LRU ordering. Hit-rate fell from 0.664 to 0.6137, contradicting the hypothesis that recently-inserted items are inherently "hotter."
- **metadata-size**: The 8-byte `recent` boolean added to each Item struct increased metadata overhead from 40 to 48 bytes without corresponding performance gains, making the space-time trade-off unfavorable.

## Neutral

- **per-op-time**: The RefCell-based operation counter and backward tail scan in eviction_candidate_index did not measurably increase latency; get p50/p99 remained at 100 ns, and put operations stayed at 100 ns p50 / 200 ns p99. The added bookkeeping complexity was absorbed without tail-latency impact.

## Takeaway

The hypothesis that a simple recency flag plus tail-preferential eviction would approximate frequency awareness failed in practice. The parent's (v003's) position-based LRU with early-exit outperforms this approach despite the intuition that preserving recently-inserted items would help under Zipfian access patterns. The backward scan trades predictable eviction order for worse cache behavior—items evicted from the tail are not necessarily cold, and the 64-operation reset interval creates artificial recency boundaries that misalign with actual working-set reuse distances. This variant demonstrates that naive recency tracking is an insufficient proxy for the frequency information that more sophisticated policies (like v003's parent) implicitly capture through pure position-based ordering.

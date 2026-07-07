# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Re-admitting ghost-hit keys at `lfu_counter = 2` and the MRU back position gives premium warm-pool protection to recently evicted keys, but on workload-a those keys do not carry enough actual future reuse to justify displacing items that cold-first eviction would otherwise have preserved.

## Takeaway

The ARC-style ghost buffer rests on the assumption that re-access shortly after eviction signals a hot key deserving warm-pool status. Workload-a does not exhibit that pattern strongly enough: ghost hits fire, but the re-admitted keys turn out to be no warmer than ordinary new arrivals, so elevating them to counter 2 at the MRU back causes the eviction path to oust genuinely warm items one step earlier. The mechanism adds cost (O(bucket_size) ghost scan per put, extra evict_item call on every ghost hit) without a compensating hit-rate gain, yielding a net regression. Ghost-based re-admission needs a trace with frequent post-eviction re-access to pay off; the counter cap and mid-point insertion inherited from the parent are the more load-bearing elements on this workload.

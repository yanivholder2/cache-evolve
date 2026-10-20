# Variant v002 Insight

## Helped

## Hurt

- **hit-rate**: Adding `lfu_counter` increments on each get and switching eviction to prefer the two-tail item with the lower counter caused the policy to occasionally evict the penultimate slot (positionally more protected) over the true tail, overriding the clean recency ordering that made the parent's pure-LRU tail eviction effective on workload-a.

## Takeaway

The hypothesis that writing `lfu_counter` and applying two-tail frequency comparison would lift hit rate above the parent backfired: the frequency signal accumulated through single-step recency promotion is too noisy to improve eviction decisions at the tail, and the two-tail selection can eject a recently-promoted item at `len-2` in place of the genuine LRU victim at `len-1`. Workload-a responds better to strict positional recency — always evict the tail — than to any frequency overlay applied only at the two-tail boundary.

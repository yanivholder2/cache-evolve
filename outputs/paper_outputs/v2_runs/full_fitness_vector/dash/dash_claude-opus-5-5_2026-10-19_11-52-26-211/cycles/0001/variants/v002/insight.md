# Variant v002 Insight

## Helped

- hit-rate: Sending a stash hit to its probing bucket when the target is full and the probing bucket has room skips the target → stash demotion, so the cache holds a bit more data. The gain is so small it may just be noise, which suggests this case (full target next to a non-full probing bucket on a stash hit) rarely comes up on workload-a.
- per-op-time: get and put both got much faster at p50 and p99, but this change can't explain a drop that large. It only adds two O(1) `is_full()` checks and skips one eviction cascade, and only on the stash-hit path. Most of the speedup is probably a difference in the benchmark environment or run, not the code, and should be re-measured before anyone relies on it.

## Hurt

## Takeaway

Balanced placement on stash → normal promotion is safe: slot legality and capacity hold, and the probing insert can't evict anything. It barely affects hit rate, though, because it only triggers when the target is full while its neighbour has room. With classic-LRU and promote-to-probing already spreading load, that combination seems to be rare. To gain more hit rate, look elsewhere: for example, balance the initial put/stash placement, or let target-hit promotions use spare capacity in the same way. Don't treat this variant's timing improvement as a result of its change.

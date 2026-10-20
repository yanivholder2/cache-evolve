# Variant v003 Insight

## Helped

- `metadata-size`: Replacing the 48-byte linked-list `Node<K,V>` (which carried two `Option<usize>` prev/next link fields) with the flat 24-byte `Slot<K,V>` halved per-entry memory overhead exactly as the hypothesis predicted.

## Hurt

- `per-op-time`: Both `get` and `put` latency rose significantly instead of falling — the CLOCK hand's forward scan through the slot ring during `put` strides across cache lines, and subsequent `get` calls then incur higher latency re-reading those evicted lines, negating the savings from removing list-pointer writes.
- `hit-rate`: CLOCK's single reference bit is a coarser approximation of recency than the exact LRU ordering it replaced; the algorithm makes slightly worse eviction decisions on this workload, producing a small but measurable drop in hit rate.

## Takeaway

The CLOCK redesign delivered its structural win — slot size halved — but the hypothesis that eliminating list-pointer writes would lower `get` latency did not materialise. The CLOCK hand's traversal of the slot array during eviction appears to be the new bottleneck: it pollutes the slot ring's cache lines on every `put` that triggers a sweep, raising `get` latency rather than reducing it. The coarser recency signal (one bit versus full LRU ordering) also carried a real hit-rate cost, suggesting that for this workload the precision of exact LRU outweighs the pointer-write overhead the hypothesis targeted.

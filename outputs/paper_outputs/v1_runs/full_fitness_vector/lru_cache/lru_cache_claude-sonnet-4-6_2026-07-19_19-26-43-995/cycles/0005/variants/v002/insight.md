# Variant v002 Insight

## Helped

- **metadata-size**: Removing `prev` and `next` from `Node<K,V>` and storing them in a parallel `Vec<Link>` shrank `Node<u64,u64>` from 24 to 16 bytes, eliminating the cross-cache-line straddle that had affected every third node during linked-list traversal and raising link-pair density from ~2.67 to 8 per 64-byte cache line.

## Hurt

_(no tracked dimension summary degraded)_

## Takeaway

The link-array split successfully achieved its structural goal — `Node` size dropped by one third and the link traversal paths now operate on a denser, better-aligned array. However, the predicted latency improvement from L1-residency of the compacted link array did not register in `per-op-time`: at the 100 ns timer resolution used by the benchmark, the cache-line density gain is invisible, and the `LruCache` struct itself grew by 24 bytes (from 120 to 144) to accommodate the `Vec<Link>` fat pointer, partially offsetting the per-entry footprint win at the struct level. The change proves that separating hot link metadata from cold key/value data is structurally sound, but surfacing a latency benefit will require either a finer-grained timer or a workload with enough arena span to stress link-traversal cache misses.

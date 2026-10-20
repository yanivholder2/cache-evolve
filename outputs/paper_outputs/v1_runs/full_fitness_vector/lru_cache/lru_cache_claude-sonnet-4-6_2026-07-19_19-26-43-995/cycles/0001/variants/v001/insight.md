# Variant v001 Insight

## Helped

- hit-rate: Shielding confirmed-hot items in a protected LRU queue prevents one-time-access flood keys from displacing them, visibly lifting hit rate on workload-a's mixed-reuse pattern.

## Hurt

- metadata-size: Adding the `protected: bool` field to `Node<K, V>` increased per-node size from 48 to 56 bytes due to struct padding, with the `LruCache` struct itself growing from 112 to 168 bytes to hold the six extra queue-pointer and length fields.
- per-op-time: Every probationary hit now requires unlinking from one chain, relinking to another, and conditionally demoting the protected head — roughly 2.5× more pointer chasing per operation than the single-chain baseline, pushing p50 get latency from 41 ns to 100 ns.

## Takeaway

The two-queue design delivers a meaningful hit-rate gain on workloads with repeated accesses, but it pays a steep latency and memory price: the conditional demotion path on every probationary hit adds significant pointer manipulation that the single-chain LRU never performs. Any follow-on variant targeting this structure should look at reducing cross-queue relinking frequency or tightening the `Node` layout before the per-op slowdown becomes the binding constraint.

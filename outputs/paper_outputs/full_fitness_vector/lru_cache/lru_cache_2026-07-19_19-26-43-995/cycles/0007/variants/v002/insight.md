# Variant v002 Insight

## Helped

_(no tracked dimension summary improved)_

## Hurt

- **metadata-size**: Adding `freq_sketch: [u8; 64]` to `LruCache<K, V>` grew the per-cache-instance struct by 64 bytes (one full cache line), increasing `LruCache<u64,u64>` from 144 to 208 bytes — though `Node<u64,u64>` is unchanged at 16 bytes, so the tracked summary metric did not move.

## Takeaway

Gating probationary graduation on a persistent CountMin sketch (freq ≥ 2) produced no change in hit rate on workload-a compared to the immediate one-hit graduation policy of v001. The sketch's core promise — that items evicted with freq=1 would re-accumulate to freq=2 on their very next probationary visit and graduate faster than a fresh item — does not translate into measurable benefit, suggesting that on this Zipfian trace the moderate-frequency items that cycle in and out of probationary are already served well by the existing policy, and the added discrimination against true one-hit items does not materially reduce protected-queue churn. The 64-byte sketch is therefore overhead without payoff on this workload.

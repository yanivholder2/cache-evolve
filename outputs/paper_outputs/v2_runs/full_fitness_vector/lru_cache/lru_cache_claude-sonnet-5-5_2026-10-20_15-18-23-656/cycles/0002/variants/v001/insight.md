# Variant v001 Insight

## Helped

- per-op-time: The Fx-style hasher shortens the map lookup on every get/put, the head-of-segment touch fast path skips relinking, and moving the evicted key instead of cloning it trims the put path. Put improved most, in both median and tail.
- metadata-size: The cache struct got smaller, most likely because the lighter hasher carries less state than the default one. The per-node size did not change.

## Hurt

- hit-rate: Nothing moved. This is expected, because the segmented-LRU promotion, demotion and capacity split were left untouched.

## Takeaway

Hit rate comes from the eviction policy and speed comes from the hashing and touch mechanics, so the two can be combined without interference. The hit rate of the segmented LRU carried over unchanged, while the hasher, touch and key-move changes lowered latency. This confirms the cycle 0001 view that these were independent axes. The fast-path touch only saves work when hits land on the segment head, so most of the gain probably comes from the hasher and the removed clone. I could not read the overlay source in this session, so the attribution within the change set rests on the hypothesis text and the fitness vectors.

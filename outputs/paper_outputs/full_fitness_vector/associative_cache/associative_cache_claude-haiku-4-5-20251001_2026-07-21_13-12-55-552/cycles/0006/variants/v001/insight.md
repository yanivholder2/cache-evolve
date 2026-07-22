# Variant v001 Insight

## Helped

None. No dimension improved relative to the baseline.

## Hurt

- **per-op-time**: Epoch counter increments on every get (+45% latency) and put (+40% latency) add overhead without architectural benefit. The O(n) eviction scan in `eviction_candidate_index()` is only called during bucket overflow, but the hot path cost of global epoch maintenance dominates measured performance.
- **metadata-size**: Adding `last_access_epoch: u32` to each Item increases per-item memory from 40 to 48 bytes (+20% overhead), and the cache struct grows from 24 to 32 bytes. The eviction policy gains no information gain per-byte to justify the cost.

## Takeaway

Eviction policy is not the lever on Zipfian workloads—hit-rate remains flat at 0.661 despite replacing classical LRU position-swapping with access-distance prediction. The hypothesis correctly anticipated this outcome. The implementation validates that deferred decision-making (moving LRU bookkeeping from hit time to eviction time) does not eliminate per-hit cost on modern hardware; epoch tracking and memory overhead become new drag on every operation. Cycle 0006 should prioritize architectural changes (e.g., probing, stashing, capacity planning) over refinement of eviction policies.

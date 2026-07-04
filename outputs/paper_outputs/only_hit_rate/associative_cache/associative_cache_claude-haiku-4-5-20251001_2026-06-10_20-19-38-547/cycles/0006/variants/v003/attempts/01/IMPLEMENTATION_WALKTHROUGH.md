# Variant v003: Phase-Aware Collision Management - Detailed Walkthrough

## Concept Overview

The hypothesis proposes that cache hit rate can be improved by detecting when a workload's access pattern changes (a "phase transition") and then accelerating the cache's adaptation to the new pattern.

**Key Insight**: When a workload changes its access pattern, the cache's eviction policy (e.g., LFU with frequency counters) has "learned" the old pattern. These learned statistics (stored in lfu_counter and timestamp fields) can interfere with the cache's ability to quickly adapt to the new pattern. By "resetting" these counters during transitions, we allow the cache to re-learn what matters in the new pattern.

## Phase Transition Detection Mechanism

### Collision Distribution as a Pattern Signature

A workload's access pattern can be characterized by how evenly (or unevenly) items are distributed across buckets:

```
Workload A (uniform access):
Bucket 0: [item1, item2]        (2 items)
Bucket 1: [item3, item4]        (2 items)
Bucket 2: [item5]               (1 item)
Bucket 3: [item6, item7, item8] (3 items)

Collision counts: [1, 1, 0, 2]
Mean: 1.0, StdDev: 0.87, CV: 0.87

Workload B (concentrated access - same cache, different phase):
Bucket 0: [item1, item2, item3, item4] (4 items)
Bucket 1: []                           (0 items)
Bucket 2: []                           (0 items)
Bucket 3: [item5, item6]               (2 items)

Collision counts: [3, -1, -1, 1]  // After saturation_sub: [3, 0, 0, 1]
Mean: 1.0, StdDev: 1.49, CV: 1.49
```

The coefficient of variation (CV = std_dev / mean) captures how "concentrated" the access pattern is:
- Low CV: items evenly distributed across buckets (uniform access)
- High CV: items concentrated in few buckets (hot spot access)

### Detecting the Transition

Every 2000 operations:
1. Compute current CV from bucket collision distribution
2. Compare with prior phase's CV
3. If |current_cv - prior_cv| > 0.20: **Phase transition detected**

This threshold of 20 percentage points is chosen to distinguish genuine pattern changes from normal variance.

## Adaptive Reset Strategy

### Normal Operation (No Transition)
```
Operations:    0---500--1000--1500--2000--2500--3000
Reset events:           |              |              
                    reset_1        reset_2       (every 1000 ops)
```

Every 1000 operations, the cache resets:
- All items' lfu_counter → 0
- All items' timestamp → now

This allows the cache to maintain a "rolling window" of recent access patterns while forgetting very old history.

### Transition Detection Occurs
At operation 2000, CV calculation detects a phase change:
```
Prior CV:   0.80 (old phase)
Current CV: 1.15 (new phase detected!)
Difference: 0.35 > 0.20 → TRANSITION DETECTED
```

### Accelerated Adaptation (Phase Transition)
```
Operations:  2000--2250--2500--2750--3000--3250--3500--3750--4000--4250
Reset freq:  500 (accelerated)
Reset events:  |     |     |     |     |     |     |     |     |
Transition lasts until operation 4000
```

During transition (2000 operations):
- Reset frequency increases to every 500 operations
- More frequent resets = faster "forgetting" of old pattern
- Cache adapts more rapidly to new access characteristics
- LFU counters reset 4 times instead of 2 times (0.5x vs 1x frequency)

### Exit Transition
At operation 4000 (2000 ops into transition):
```
Current time: 4000
Transition started: 2000
Duration: 2000 → Exit transition mode
Reset frequency: 500 → 1000 (revert to normal)
Next reset: at 5000 (not 4250)
```

## Impact on Eviction Policies

### For LFU (Least Frequently Used)
- **Before**: Old items with high lfu_counter evicted last (still "remembered")
- **On transition**: Counter reset → all items treated equally
- **Effect**: Cache can evict "yesterday's hot items" to make room for "today's hot items"

### For TimestampLRU
- **Before**: Old items with old timestamps remain in cache
- **On transition**: Timestamp reset → all items appear equally recent
- **Effect**: New items compete fairly with old items for eviction

### For Other Policies (FIFO, Lifo, Swap, ClassicLRU)
- Reset affects only lfu_counter and timestamp (not used by these policies)
- Behavior remains unchanged, but timestamp field is reset

## Expected Behavior on Different Workload Types

### Workload with No Phase Changes
- Transition never detected (CV doesn't change by > 20%)
- Cache operates in normal mode: reset every 1000 ops
- Behavior similar to baseline, with added statistical overhead

### Workload with Distinct Access Phases
```
Phase 1 (ops 0-2000):   Uniform access → CV ≈ 0.8
Phase 2 (ops 2000-4000): Hot spot access → CV ≈ 1.2
Transition detected!
  → Reset frequency 500
  → Cache adapts faster
  → Hit rate improves
```

### Workload with Frequent Phase Changes
```
Multiple transitions detected
Each transition:
  → 2000 ops at accelerated 500-op resets
  → Then back to normal 1000-op resets
Cache keeps adjusting to changes
```

## Performance Characteristics

### Overhead
- **Per operation**: Increment 2 counters, 1 modulo check
- **Every 1000-2000 ops**: CV computation (128 iterations for bucket count)
- **Every 1000-500 ops**: Reset all items' counters (linear in cache size)

### Expected Hit Rate Improvement
- Best case: Workloads with distinct phases → 2-5% improvement
- Neutral case: Uniform workloads → <0.5% change
- Worst case: Adversarial patterns → 0-1% potential decrease

## Debugging/Monitoring

The implementation includes debug/info logging at key points:
```rust
// Logged at every phase transition check:
"AssociativeCache.check_phase_transition: op_count={} mean={:.3} std_dev={:.3} cv={:.3} prior_cv={:.3}"

// Logged when transition detected:
"AssociativeCache: phase transition detected at op_count={}. CV diff={:.3}"

// Logged when exiting transition:
"AssociativeCache: exiting phase transition at op_count={}"

// Logged at every reset:
"AssociativeCache: resetting collision counters at op_count={} phase_window={}"
```

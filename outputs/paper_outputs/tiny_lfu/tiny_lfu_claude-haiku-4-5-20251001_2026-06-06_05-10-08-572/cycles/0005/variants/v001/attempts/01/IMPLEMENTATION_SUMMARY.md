# Variant v001 Implementation Summary

## Hypothesis
Replace LRU-only victim selection with frequency-aware candidate selection to improve hit-rate on the latency-optimized depth-3 baseline.

## Parent Variant
- **Cycle**: 0002
- **Variant**: v002
- **Key Achievement**: Reduced sketch depth from 4 to 3, improving latency (get: 132→90 ns, put: 166→117 ns) but reducing hit-rate (0.7362 → 0.7334)

## Changes Made

### File Modified
- `src/tiny_lfu/mod.rs`

### Key Modifications

#### 1. Maintained Depth-3 Sketch Geometry
- `SKETCH_DEPTH = 3` (inherited from parent v002)
- Preserves the latency gains from the parent variant

#### 2. Frequency-Aware Victim Selection (lines 98-126)
Replaced pure LRU victim selection with a frequency-aware candidate window approach:

**Old Logic (lines 98-109 in baseline):**
```rust
let victim_idx = self.head.expect("...");
let victim_freq = self.sketch.estimate(&victim_key);
let incoming_freq = self.sketch.estimate(&key);
if incoming_freq > victim_freq {
    // evict head unconditionally
}
```

**New Logic:**
1. Start with `self.head` as initial candidate
2. Scan up to **5 items** from the head of the linked list
3. For each item in the window, compute estimated frequency using `sketch.estimate()`
4. Track the candidate with **minimum estimated frequency**
5. Use that candidate as the victim for eviction
6. Keep admission condition unchanged: `incoming_freq > victim_freq`

### Implementation Details

```rust
// Scan up to 5 items from head, find minimum frequency
let mut victim_idx = self.head.expect("TinyLfuCache: full cache has no victim");
let mut min_freq = self.sketch.estimate(&self.nodes[victim_idx].key);
let mut candidate = victim_idx;
let mut current = self.nodes[victim_idx].next;

// Scan up to 4 more items (5 total) and find the one with minimum frequency
for _ in 0..4 {
    if let Some(idx) = current {
        let freq = self.sketch.estimate(&self.nodes[idx].key);
        if freq < min_freq {
            min_freq = freq;
            candidate = idx;
        }
        current = self.nodes[idx].next;
    } else {
        break;
    }
}

// Evict the candidate with minimum frequency
victim_idx = candidate;
let victim_key = self.nodes[victim_idx].key.clone();
let victim_freq = self.sketch.estimate(&victim_key);
let incoming_freq = self.sketch.estimate(&key);
if incoming_freq > victim_freq {
    // perform eviction
}
```

## Expected Behavior

This change targets hit-rate improvement by:
1. **Avoiding LRU-only evictions**: Instead of blindly evicting the oldest item, we consider a small window of candidates
2. **Frequency-aware selection**: Among the window, we evict the truly cold items (minimum estimated frequency) rather than purely time-based items
3. **Low overhead**: A fixed window of 5 items provides frequency awareness without O(n) overhead per operation
4. **LFU alignment**: Better respects the LFU principle while maintaining the depth-3 sketch's latency benefits

## Invariants Preserved

- ✓ Linked list structure (unlink/push_back operations unchanged)
- ✓ Capacity bounds (still enforces capacity limit)
- ✓ Admission control (incoming_freq > victim_freq condition unchanged)
- ✓ Frequency estimation (sketch operations unchanged)
- ✓ Framework Cache<K,V> contract (interface unchanged)

## Expected Outcome

By combining:
- **Parent's latency gains**: Depth-3 sketch reduces hashing work
- **This variant's selectivity**: Frequency-aware victim selection improves hit-rate decisions

We expect to recover some or all of the hit-rate loss from parent v002 while maintaining the latency improvements.

Target: hit-rate > 0.7334 (parent), ideally approaching or exceeding 0.7362 (baseline v002 parent)

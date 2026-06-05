# Variant v003: Threshold-Based Frequency Amnesia

## Hypothesis
Threshold-based frequency amnesia will improve hit-rate above 0.7334 by cleanly separating working-set items from ephemeral accesses through sharp forgetting boundaries.

## Implementation Summary

### Key Changes to LruCache

1. **Frequency Tracking**
   - Added `freq: u8` field to Node structure (3-bit quantized, range 0-7)
   - On `get()`: increment frequency (capped at 7)
   - On `put()`: new items start with frequency 0
   - Frequency is NOT incremented on updates (put of existing key)

2. **Periodic Decay Mechanism**
   - Added `operation_count: u64` to track cumulative get/put operations
   - Added `decay_interval: u64` = capacity × 16
   - Decay is triggered after every capacity × 16 operations

3. **Threshold-Based Amnesia**
   - Replaces smooth multiplicative decay (0.75×) with sharp threshold forgetting
   - On decay: reset all freq < 2 to 0
   - Keep frequencies ≥ 2 unchanged
   - Creates clear separation between working-set items (freq ≥ 2) and ephemeral accesses (freq < 2)

4. **Two-Pass Victim Selection**
   - First pass: find minimum frequency among all cached items
   - Second pass: among items with minimum frequency, select the LRU (head) item
   - Evicts the LRU item with minimum frequency, protecting high-frequency items from eviction

### Design Rationale

**Sharp vs. Smooth Decay**: The hypothesis replaces smooth multiplicative decay (0.75×) with a sharp threshold-based approach. This creates a discontinuous boundary at frequency=2:
- Items with freq < 2 are treated as ephemeral and reset to 0
- Items with freq ≥ 2 are treated as part of the working set and protected

**Two-Pass Selection**: By finding minimum frequency first, then selecting LRU among those, we protect hot items (high frequency) from eviction while still evicting older items among the cold set.

### Compatibility

- Maintains the same `Cache<K, V>` trait interface
- Preserves arena-based slot reuse (no free-list management)
- Maintains HashMap + doubly-linked list structure
- All structural invariants (capacity bound, value fidelity, no phantoms) are preserved

### Test Coverage

Added `frequency_tracking` test to verify:
- Multiple accesses to a key increase its frequency
- High-frequency items are protected from eviction
- New items with frequency 0 are evicted before high-frequency items

## Expected Outcomes

Threshold-based amnesia should improve hit-rate by:
1. Clearly distinguishing working-set items (freq ≥ 2) from one-time accesses (freq < 2)
2. Aggressively evicting ephemeral accesses after periodic decay
3. Providing better protection for items with demonstrated reuse patterns

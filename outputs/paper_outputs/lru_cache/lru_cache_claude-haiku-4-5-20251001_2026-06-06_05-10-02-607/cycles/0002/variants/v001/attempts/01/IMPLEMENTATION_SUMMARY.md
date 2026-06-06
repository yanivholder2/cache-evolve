# Variant v001 Implementation Summary

## Hypothesis
Bound eviction search to the first N nodes to prevent pathological O(N) scans on high-frequency clusters, reducing put p99 latency.

## Changes Made

Modified `find_evictable_node()` in `src/lru_cache/mod.rs` to implement bounded search:

### Key Modifications

1. **Search Window Bound**: Limited the scan to `min(capacity / 4, 256)` nodes from the head (LRU end), preventing pathological full-list scans.

2. **Search Logic**:
   - Calculate search limit: `search_limit = std::cmp::min(self.capacity / 4, 256)`
   - Track nodes scanned with a counter
   - Return immediately upon finding the first node with `freq <= 1` within the window
   - Break the loop once the search limit is reached

3. **Fallback Strategy**: If no node with low frequency is found within the search window, fall back to evicting the LRU (head) node in that window, even if its frequency is higher.

### Preserved Characteristics

- **Frequency decay mechanism**: Still decays all frequencies every `capacity` evictions
- **Hit-rate benefits**: Hybrid frequency-LRU strategy remains intact; low-frequency items are preferentially evicted when found
- **Scanning strategy for eviction**: Changed only the depth/window, not the logic or order

### Expected Impact

- **Reduces put p99 latency**: Bounds the eviction search to O(min(capacity/4, 256)) instead of O(capacity)
- **Maintains hit rate**: Still preferentially evicts low-frequency items; only changes the search depth
- **Graceful degradation**: Under high-frequency scenarios, falls back to LRU eviction within the search window

## Testing

Added a new test `bounded_search_within_window()` that verifies:
- Eviction only searches within the bounded window
- Low-frequency items within the window are evicted before high-frequency items outside it
- High-frequency items are protected from eviction unless all items in the window have high frequency

# Variant v001: Adaptive Bucketing + Deferred-Promotion Queue

## Overview

This variant implements **H01**: aggressive tuning of deferred-promotion queue parameters to reduce get p50 latency regression while preserving p99 improvements from cycle/0007/v001.

## Key Changes from Parent (v003)

### 1. Deferred-Promotion Queue with Two-Batch Swap

Added batching mechanism for frequency bucket updates to amortize bookkeeping overhead:

- **`deferred_batch`**: Main batch accumulating pending node indices
- **`swap_batch`**: Secondary batch awaiting flush
- **`max_batch_size`**: 8 items (tuned parameter)
- **`swap_threshold`**: 4 items (50% of max_batch_size)

### 2. Mechanism

Instead of immediately updating frequencies on every `get()` and `put()` call:

1. Node indices are accumulated in `deferred_batch`
2. When `deferred_batch.len() >= swap_threshold` (reaches 4 items), we:
   - Flush the residual `swap_batch` from the previous cycle
   - Swap the batches
   - Immediately process the now-secondary batch to frequency buckets
3. This batches 4 get/put accesses' frequency updates into one processing pass

### 3. Latency Reduction Strategy

**Goal**: Reduce per-get cost by batching expensive unlink/relink operations.

- In bucketing mode, updating frequency requires:
  - Unlinking node from old frequency bucket
  - Incrementing frequency counter
  - Linking to new frequency bucket
  
- By batching 4 such updates, the amortized cost per operation is reduced
- More swap operations (50% threshold vs full) = faster amortization

### 4. Safety Mechanisms

- **Eviction handling**: When a node is evicted, it's removed from both deferred and swap batches to prevent updating a reused slot
- **Mode switching**: When transitioning between simple and bucketing modes, all deferred updates are flushed first
- **Simple mode**: Continues with immediate promotions (no batching, since it's less expensive than bucketing)

## Preserved Features

- **Adaptive bucketing**: Mode switching at 75% saturation
- **Hit rate tracking**: 65% threshold for bucketing activation
- **Frequency decay**: Periodic decay every N evictions
- **Bounded-window eviction**: In simple mode for underutilized caches

## Expected Outcome

**Hypothesis**: More frequent swaps (every 4 accesses) with smaller batch size (8 vs 16) reduces per-get bookkeeping overhead while maintaining the p99 improvements from cycle/0007/v001.

Target metrics:
- Get p50: Closer to v003's 34 ns (vs 42 ns regression in v001)
- Get p99: Maintain ~42% improvement (from v003's 97 ns baseline)
- Put p99: Maintain ~50% improvement (from v003's 177 ns baseline)

## Implementation Details

### Deferred Batch Processing

```rust
flush_deferred_batch_if_needed() {
    if deferred_batch.len() >= swap_threshold {
        // Flush residual from swap_batch
        for each item in swap_batch {
            update_node_frequency_direct(item)
        }
        // Swap main into secondary
        swap(deferred_batch, swap_batch)
        // Flush now-secondary batch
        for each item in swap_batch {
            update_node_frequency_direct(item)
        }
        clear(swap_batch)
    }
}
```

### Eviction Safety

When evicting node at index `evict_idx`:
```rust
deferred_batch.retain(|&idx| idx != evict_idx)
swap_batch.retain(|&idx| idx != evict_idx)
```

This prevents attempting to update frequencies of reused slots.

## Testing

The implementation maintains all existing test cases from the parent variant, ensuring correctness of:
- Get miss/hit
- Put with overwrites
- LRU eviction order
- Capacity edge cases
- Mode transitions (implicit through saturation/hit_rate tests)

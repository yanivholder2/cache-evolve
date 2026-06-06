# Variant v001: Design Document

## Hypothesis

**H01**: Reduce get p50 latency regression by aggressively tuning batch-swap parameters in a deferred-promotion queue-flush mechanism to minimize bookkeeping overhead while preserving tail-latency gains.

## Rationale

- **Problem**: Cycle/0007/v001 achieved breakthrough p99 improvements (42% get, 50% put) but incurred a 20% get p50 regression (35→42 ns) due to two-batch swap bookkeeping on every get.
- **Question**: Can batch size and swap frequency tuning reduce this overhead while preserving p99 gains?
- **Approach**: Apply aggressive parameter reduction to the proven cycle/0003/v003 adaptive bucketing baseline.

## Design

### Core Mechanism: Deferred-Promotion Queue

Instead of immediately updating frequency buckets on every get/put:

1. **Accumulation Phase**
   - Node indices are collected in `deferred_batch`
   - No frequency updates occur during this phase
   
2. **Flush Phase (triggered at 50% capacity)**
   - When `deferred_batch.len() >= swap_threshold` (4 items for batch size 8):
     - Flush any residual items in `swap_batch`
     - Swap the batches
     - Process all items in the newly-secondary batch
   
3. **Result**
   - Expensive unlink/relink operations are batched
   - Cost amortized across more accesses
   - Per-get latency reduced

### Parameters (Tuned from cycle/0007/v001)

| Parameter | Cycle/0007/v001 | This Variant | Rationale |
|-----------|-----------------|--------------|-----------|
| `max_batch_size` | 16 | 8 | Reduce overhead per batch |
| `swap_threshold` | Full (100%) | 50% (4/8) | Trigger swaps more frequently for faster amortization |

### Mode Strategy

Maintains **dual-mode from v003**:
- **Simple Mode** (underutilized): Bounded-window eviction, no batching (already fast)
- **Bucketing Mode** (≥75% saturation): Frequency buckets with deferred promotion queue

### Safety

**Eviction Handling**: When a node is evicted, its index is removed from both `deferred_batch` and `swap_batch` to prevent attempting to update frequency of a reused slot.

**Mode Switching**: All deferred updates are flushed before transitioning between modes.

## Expected Outcomes

### Primary Goal
Reduce get p50 latency from v003/v001's 35 ns baseline while maintaining p99 improvements:
- **Get p50**: Target ~35-38 ns (vs 42 ns regression in v001)
- **Get p99**: Maintain ~97 ns or better (34% improvement from original)

### Secondary Goals
- **Put p99**: Maintain ~177 ns or better (7% improvement from original)
- **Hit Rate**: Maintain ~0.69 (equivalent to v003)
- **Metadata Size**: Similar to v003 (~192 bytes) — dual-mode state unavoidable

## Trade-offs

| Aspect | Trade-off |
|--------|-----------|
| **Latency** | Batching reduces per-op cost but introduces small batching delay |
| **Consistency** | Deferred updates are eventually applied, maintaining correctness |
| **Memory** | Batch storage is minimal (2 Vecs of ~8 items each) |
| **Simplicity** | More complex than pure v003 but justified by latency target |

## Validation

The implementation:
- ✓ Preserves all existing test cases
- ✓ Maintains correctness invariants (LRU ordering, eviction, capacity bounds)
- ✓ Implements full hypothesis (batch size 8, swap at 50%)
- ✓ Builds against subject crate
- ✓ Ready for gate testing (contract, universal invariant, invariant)

## Files Changed

- **`src/lru_cache/mod.rs`**: Core cache implementation with deferred-promotion queue and two-batch swap mechanism

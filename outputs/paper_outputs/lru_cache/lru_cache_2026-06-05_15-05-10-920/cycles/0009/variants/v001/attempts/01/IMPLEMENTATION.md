# Variant v001: Per-Access Frequency Decay

## Hypothesis
Apply per-access frequency decay proportional to elapsed operations since last access to improve cache responsiveness to gradual access pattern phase shifts without relying on fixed-interval periodic decay.

## Changes Made

### 1. Node Structure
- **Added**: `last_access_counter: u64` field to track when each node was last accessed

### 2. LruCache Structure
- **Added**: `global_operation_counter: u64` to count total operations (get + put)
- **Removed**: `frequency_reset_counter` and the entire `check_and_perform_frequency_reset()` mechanism

### 3. get() Method
After incrementing node frequency, apply per-access decay:
- Compute `ops_since_access = global_operation_counter - node.last_access_counter`
- Apply decay formula:
  - If `ops_since_access > 64`: frequency is reduced linearly: `freq * (128 - (ops_since_access / 64)) / 128`
  - If `ops_since_access ≤ 64`: no decay, frequency unchanged
- Update `last_access_counter = global_operation_counter`
- Increment `global_operation_counter`

### 4. put() Method
- For existing keys: increment `global_operation_counter` 
- For new keys (eviction case):
  - Use `compute_decayed_freq()` helper to find victim with minimum decayed frequency
  - When reusing slot: set `freq = 0`, `last_access_counter = global_operation_counter`
- Increment `global_operation_counter`

### 5. Helper Method
Added `compute_decayed_freq(idx)` to calculate effective frequency with decay applied, without modifying node state. Used during victim selection in eviction.

## Rationale

**Why per-access decay vs. periodic decay?**
- Parent variant v003 uses periodic decay (0.75× every `capacity × 16` operations), which assumes phase shifts occur at regular intervals
- Real workloads may transition gradually rather than at regular boundaries
- Per-access decay adapts smoothly to actual access patterns without statistical overhead

**Decay formula:**
- Linear decay: frequency decreases by approximately 0.78% per 64 operations without access
- At 8192+ operations without access, frequency decays to 0
- Provides gentle forgetting compared to parent's multiplicative approach while remaining responsive to workload dynamics

## Test Coverage
The implementation includes a new test `frequency_decay_with_operation_delta()` that verifies:
- Decay is computed based on operation deltas
- Cache remains functional after decay application
- No observable errors during extended access patterns

## Invariants Maintained
- Cache contract: put→get round-trips work correctly
- Universal invariants: no phantom items, never-put keys return None
- Eviction: minimum frequency victims are selected (with decay applied)
- Capacity: never exceeds cache size

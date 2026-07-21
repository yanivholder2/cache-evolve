# Variant v001 Insight

## Helped

(no dimension moved)

## Hurt

(no dimension moved)

## Takeaway

Replacing `Vec<bool>` with a packed `Vec<u64>` bitset shrank the heap allocation behind `prot_flags` by 8× at capacity=2048 (2048 bytes → 256 bytes), but none of the three fitness dimensions captured this saving: `metadata-size` tracks the `Node` struct size and `TinyLfuCache` struct size, neither of which changes because a `Vec<u64>` and a `Vec<bool>` occupy identical space inside the struct itself (pointer + length + capacity). `hit-rate` was correctly unchanged — the change is purely representational — and `per-op-time` was also flat because the extra bit-shift and mask operations cost the same as a direct boolean array index at these workload sizes. The fitness model needs a metric that measures off-struct heap allocations (e.g., the sum of all Vec backing stores) before this class of compaction change can register a signal.

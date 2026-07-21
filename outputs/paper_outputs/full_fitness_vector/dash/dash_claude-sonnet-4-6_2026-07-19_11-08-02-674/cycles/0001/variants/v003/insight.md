# Variant v003 Insight

## Helped

- **metadata-size**: Dropping the `timestamp: Instant` field from `Item` shrank the struct from 40 to 24 bytes exactly as predicted, cutting per-item overhead by 40%.
- **per-op-time**: Removing `Instant::now()` from `Item::new()` eliminated a syscall on every `put`, and degrading `TimestampLRU.get_from_position` from a timestamp-stamping operation to a simple vector move eliminated a second syscall on every `get`, yielding large latency reductions for both operations.

## Hurt

_(none)_

## Takeaway

The `timestamp` field in `Item` was a dual cost: it bloated every cached entry by 16 bytes and injected an `Instant::now()` syscall into both `put` (at allocation) and `get` (via `TimestampLRU`). Removing it improved memory footprint and operation latency simultaneously with no hit-rate penalty, showing that TimestampLRU's wall-clock ordering provided no eviction advantage over position-based ordering on this workload.

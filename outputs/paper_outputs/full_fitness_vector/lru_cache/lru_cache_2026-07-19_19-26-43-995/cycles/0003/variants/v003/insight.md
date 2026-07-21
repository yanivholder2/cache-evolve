# Variant v003 Insight

## Helped

_(no dimensions improved)_

## Hurt

- **per-op-time**: Both `get` and `put` latencies increased despite the change being limited to the eviction branch of `put()` — including `get`, which was not modified at all — suggesting the benchmark conditions differed rather than the field-split borrow introducing real overhead; the optimization had no measurable benefit for the `u64` key type used in the trace.

## Takeaway

The field-split borrow was applied correctly and the `evicted_key` clone was eliminated, but the change produced no latency improvement because the benchmark key type is `u64`, a `Copy` type for which `.clone()` is a trivial register copy with no heap allocation to remove. The hypothesis's performance claim is only valid for non-trivial key types with owned heap data; measuring it against a `u64` workload cannot confirm or deny the optimization's value for its intended target.

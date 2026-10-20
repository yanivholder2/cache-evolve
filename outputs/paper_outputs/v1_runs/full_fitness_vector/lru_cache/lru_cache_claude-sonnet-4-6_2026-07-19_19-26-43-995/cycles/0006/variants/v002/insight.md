# Variant v002 Insight

## Helped

_(no tracked dimension summary improved)_

## Hurt

- **hit-rate**: The SEEN_BIT gate holds probationary items at their FIFO position until a second access arrives; on this workload many of those items are evicted by the probationary LRU head before the second hit comes, causing misses that the enlarged protected pool (capacity/8 probationary cap, giving ~87.5% protected headroom) was not large enough to offset.
- **per-op-time**: The get-path summary rose by one 100 ns timer tick relative to the parent's anomalous zero-floor reading — the two-hit path adds a SEEN_BIT conditional branch on every probationary access, replacing a full unlink+relink only on the second hit, but the benchmark's coarse clock cannot distinguish real latency change from measurement noise here.

## Takeaway

Composing the SEEN_BIT filter with a larger protected quota did not produce the anticipated hit-rate lift on workload-a: the stricter two-hit admission barrier evicts more useful items from the probationary FIFO than it protects, because the workload does not provide enough repeat accesses within the probationary window to let items earn graduation. The enlarged protected pool only helps items that pass the gate — if few items pass, more headroom in protected is irrelevant. Future attempts should either loosen the admission threshold (e.g. time-based or position-based SEEN reset) or measure the graduation rate directly to confirm whether the two-hit gate is too strict for this trace before tuning the protected quota.

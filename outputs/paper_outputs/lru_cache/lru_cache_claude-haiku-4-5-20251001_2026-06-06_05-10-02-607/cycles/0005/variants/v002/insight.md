# Variant v002 Insight

## Helped

- **per-op-time (put p99)**: Incremental draining of the promotion queue during get() reduces the batch size that put() must flush at once, cutting the tail latency spike by 42% (297 ns → 173 ns).
- **per-op-time (put p50)**: Smaller pending queue at put time reduces typical put latency overhead from bucket manipulations, improving median latency by 2% (88 ns → 86 ns).

## Hurt

- **per-op-time (get p99)**: The drain-check and batch processing of promotions when the queue reaches threshold (8 items) adds work to the get path itself, increasing tail latency by 32% (114 ns → 151 ns).

## Takeaway

This variant demonstrates that incremental draining successfully distributes the deferred promotion work across both operations, substantially reducing put p99 from a spike of 297 ns to 173 ns. However, the amortization comes at the cost of making some get operations slower when the queue is full, since they must now process part of the accumulated work. The trade-off favors systems where put tail latency is more critical than get tail latency, but reveals the fundamental constraint: deferral spreads cost but doesn't eliminate it—the total work remains constant, just redistributed.

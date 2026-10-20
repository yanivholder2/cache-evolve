# Variant v001 Insight

## Hurt

- **hit-rate**: Stricter promotion threshold (freq > 2 instead of freq > 1) reduces items graduating to main queue, causing marginal misses on moderately-reused items that don't reach the higher frequency bar.

## Helped

- **per-op-time**: Fewer items in the main queue due to stricter admission creates significantly lower latency on both get and put operations, with get p50 improving from 248ns to 138ns and put p99 dropping from 4150ns to 1502ns.

## Takeaway

Raising the small-queue promotion threshold from freq > 1 to freq > 2 trades a negligible hit-rate decline for substantial per-operation latency wins. The stricter filter keeps the main queue less populated, reducing lookup contention and eviction overhead. This variant reveals that the admission threshold is a latency lever: tighter filtering buys speed at the cost of precision in hotness detection. For workloads where operation latency matters more than maximal cache hits, this trade-off is valuable.

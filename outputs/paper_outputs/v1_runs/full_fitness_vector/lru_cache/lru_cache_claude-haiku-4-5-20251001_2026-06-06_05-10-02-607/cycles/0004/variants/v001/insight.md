# Variant v001 Insight

## Hurt

- **per-op-time (get p50)**: Median latency regressed 56% (34→53 ns) despite the intended optimization, likely due to counter increment and modulo overhead dominating the cost of the decision logic being skipped.
- **per-op-time (get p99)**: Tail latency degraded 162% (97→254 ns); the cache's fast path through mode transitions in v003 was faster than the added counter-management overhead even with periodic sampling.
- **per-op-time (put p50)**: Median put latency increased 26% (72→91 ns) from counter increments and sampling checks outweighing the savings of skipped decision logic.
- **per-op-time (put p99)**: Tail latency worsened 94% (177→344 ns); periodic sampling didn't prevent mode-transition overhead when transitions do occur, and the counter bookkeeping adds fixed cost to every operation.
- **metadata-size**: LruCache struct grew 8 bytes (192→200 bytes) to hold the operation_counter field.

## Helped

- No metrics improved in this variant.

## Takeaway

Periodic sampling (every 16 operations) to reduce mode-decision overhead backfired badly. On this underutilized workload, the lightweight decision logic in v003 was fast enough that skipping it 15/16 times does not save wall-clock time—the counter increment and modulo check on every operation dominate the per-op cost instead. The workload never saturates, so mode transitions are rare; the hypothesis's assumption that decision overhead was a significant bottleneck turned out to be wrong. This teaches that counter maintenance and branch overhead can overwhelm any savings from deferred work when the original work is already O(1) and cheap, and that sampling strategies tuned for high-contention scenarios may actively hurt latency-sensitive code on underutilized paths.

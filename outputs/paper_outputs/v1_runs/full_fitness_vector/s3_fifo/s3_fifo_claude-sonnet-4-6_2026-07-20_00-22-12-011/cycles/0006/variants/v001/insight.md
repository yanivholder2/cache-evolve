# Variant v001 Insight

## Helped

- hit-rate: Shrinking the small queue to 1/80 of capacity gave the main queue a slightly larger resident footprint, allowing the hot Zipfian working set to stay resident a bit longer and raising hit-rate marginally above the parent's 1/40 result.

## Hurt

- per-op-time: With a smaller small queue, entries turn over faster and `evict_small()` is triggered more frequently per `put()` call; this additional eviction work pushed both get and put latencies well above the parent's measurements, reversing the near-zero p50 that cycle/0005 achieved.

## Takeaway

Tightening the admission filter from 1/40 to 1/80 delivered the predicted (tiny) hit-rate improvement on the Zipfian workload, but the cost was a significant per-operation latency regression: more frequent small-queue evictions added overhead that outweighed the benefit. The result suggests that the 1/40 sizing from cycle/0005 is near the practical floor for this workload — going tighter trades a negligible quality gain for meaningful throughput loss.

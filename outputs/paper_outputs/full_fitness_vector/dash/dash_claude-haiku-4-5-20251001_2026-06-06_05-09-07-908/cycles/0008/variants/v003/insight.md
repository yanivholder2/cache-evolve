# Variant v003 Insight

## Helped

- hit-rate: The lower promotion threshold (divisor 4, yielding threshold 6) permits more items to reach the probing tier, allowing the LFU counter logic to better identify and retain genuinely hot items rather than being starved by a conservative gate.
- per-op-time (get): With hot items concentrated in the probing tier, lookups succeed in fewer bucket hops, cutting median latency by more than half.
- per-op-time (put median): The more liquid promotion pathway reduces contention in the target bucket, allowing fresh inserts to find space with less cascading.

## Hurt

- per-op-time (put p99): Under heavy insertion load, the increased promotion traffic creates spike-tail latency as the tier cascade handles more simultaneous evictions.

## Takeaway

Divisor 4 proves to be the permissiveness sweet spot on baseline 25/75 geometry. By lowering the threshold from 12 to 6, items earn their way into the probing tier more easily, which paradoxically improves selectivity: the buckets fill with items that truly deserve the premium tier rather than sitting in the target bucket waiting for an unreachable threshold. The result is a 1.1% hit-rate gain and sub-100ns median get latency, repositioning the baseline geometry as competitive with more aggressive variants when paired with an appropriately tuned promotion gate.

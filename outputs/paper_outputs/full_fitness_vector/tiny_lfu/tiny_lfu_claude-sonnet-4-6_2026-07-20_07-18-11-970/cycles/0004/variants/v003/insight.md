# Variant v003 Insight

## Helped

- hit-rate: Requiring an incoming item to beat the victim by more than one count reduced marginal-frequency evictions where CMS noise could flip the true ordering, keeping genuinely hot incumbents resident longer on the Zipfian trace.

## Hurt

- per-op-time: Both get and put p50 latencies rose substantially relative to the baseline despite the change being a single `saturating_add(1)` in the admission branch; the stricter filter reduces successful evictions per put call, shifting the branch-prediction and memory-access pattern in the hot `put` path in a way that outweighs the trivial arithmetic cost.

## Takeaway

Admission stability via a one-count incumbent advantage is a real lever on a stable Zipfian working set: the tighter threshold filters out the ambiguous marginal-frequency cases where sketch noise is most likely to cause unnecessary churn, and hit rate improves as a result. The per-op-time regression is a caution — changing how often eviction actually fires affects downstream memory access patterns enough to measurably shift latency, so future attempts coupling a stricter admission gate with reduced sketch overhead or batched eviction paths may be needed to keep throughput intact.

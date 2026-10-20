# Variant v002 Insight

## Helped

- (none) No dimension improved. metadata-size held at 24 as predicted, since the edit touched only `get`'s body and left the `Node` layout untouched — but holding steady is not a gain.

## Hurt

- hit-rate: Dropping the per-hit `unlink`+`push_back` degrades the list from strict LRU to insertion order, so the sampled-coldest eviction walk now scans oldest-*inserted* entries; re-accessed-but-old hot keys are never pulled to the tail and fall into the victim sample, and the frequency gate alone does not fully protect them — a large regression that disproves the "hit-rate is admission-bound, not recency-bound" premise.
- per-op-time: get p50 *rose* despite removing ~7 pointer writes from the hit path, and put p50/p99 also rose; the reorder was never on the critical path, so deleting it produced no speedup and the measurement moved the wrong way.

## Takeaway

The per-`get` LRU reorder is load-bearing for hit-rate on this trace: recency does feed eviction, because the move-to-tail is what keeps re-accessed entries out of the oldest-inserted sample window the victim walk scans. Removing it is a double loss — the predicted get-p50 win never materialized (the list surgery wasn't the bottleneck), while hit-rate fell sharply. This quantifies that the reorder buys real hit-rate and costs effectively nothing measurable, so it should stay.

# Variant v001 Insight

## Helped

- hit-rate: Cutting the small queue from 25% to 10% of capacity (512/1536 → 204/1844 at capacity 2048) moves residency to the main queue. Keys that prove reuse, through `freq > 1` graduation or a ghost hit, now get a longer second-chance lifetime there. The ghost queue is sized to `main_size`, so it also grows and remembers more demoted keys to re-admit straight into main. On workload-a, these gains outweigh the cost of a shorter probation window.

## Hurt

(No dimension regressed. This variant's fitness vector reports only `hit-rate`. `metadata-size` and `per-op-time` were not measured, so this change says nothing about them either way.)

## Takeaway

On workload-a at capacity 2048, a bigger main queue matters more than a longer probation window. The paper's 10% split beats the baseline's 25% even though the graduation rule still needs two hits inside a now much shorter small-queue window. A key that misses graduation is not lost: it falls into the larger ghost queue and returns to main on its next access. The shorter probation therefore seems to cost little, but this needs a follow-up measurement. The larger ghost is also scanned linearly on every `put`, so `per-op-time` should be measured before this split is adopted.

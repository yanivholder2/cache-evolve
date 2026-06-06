# Variant v001 Insight

## Helped

- per-op-time (get): Negligible latency improvement from tracking frequency in ghost queue structure.

## Hurt

- hit-rate: Loosening retention to allow freq ≥ -1 and boosting re-admission to freq+2 caused the ghost queue to fill with low-confidence candidates that crowded out high-frequency entries in main, reducing hit-rate from 0.7164 to 0.6497.
- per-op-time (put): Ghost queue growth from preserved low-frequency entries increased the linear search cost of ghost hits from 577ns to 827ns (43% increase).

## Takeaway

The hypothesis to recover hit-rate by relaxing the ghost-queue retention threshold proved counterproductive. While preserving items with negative frequencies seemed likely to retain re-access opportunities, the strategy instead inflated the ghost queue with marginal candidates that competed for main-queue slots when re-admitted. The boosted re-admission weight (freq+2) was insufficient to reflect their actual utility, and the resulting main-queue pollution displaced higher-frequency entries. The variant demonstrates that ghost-queue retention policy and re-admission strategy are tightly coupled—simply loosening one without deeper analysis of candidate quality destroys performance.

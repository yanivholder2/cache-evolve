# Variant v001 Insight

## Helped

- **hit-rate**: Incrementally tightening the small-queue admission ratio from 1/10 to 1/11 improves selectivity further, demonstrating that the boundary-discovery search has not yet reached saturation and that the workload's sparse reuse patterns continue to benefit from even stricter frequency-based filtering at the gate.

## Hurt

(none)

## Takeaway

Advancing from 1/10 to 1/11 capacity for the small queue sustains the improvement trajectory established across cycles 0002–0005, confirming that S3-FIFO's performance on this workload remains sensitive to tighter admission control. The modest but consistent gain indicates that items in the small queue continue to be filtered effectively by the freq > 1 threshold, preserving cache diversity while blocking the vast majority of single-access pollution. This variant reinforces that the saturation point lies beyond 1/11 and warrants continued exploration at even tighter ratios in future iterations.

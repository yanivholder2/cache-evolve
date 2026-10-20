# Variant v003 Insight

## Helped

## Hurt

## Takeaway

Feeding main-queue victims with frequency of at least 1 into the ghost queue left hit-rate on lirs workload-a at capacity 2048 exactly where the baseline sits, so the warm-key half of the cycle-0009 ghost feed contributes nothing on this trace. That is consistent with the H03 prediction that the earlier gain came from cold keys, but a no-op implementation would look the same, so the overlay diff should be checked to confirm the ghost push actually fires before treating this as evidence.

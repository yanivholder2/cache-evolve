# Variant v002 Insight

## Helped

## Hurt

## Takeaway

Halving the main-queue frequency decay step (decrementing by 2 instead of 1 on each reclaim pass) left hit-rate on lirs workload-a at size 2048 exactly equal to the baseline, so this lever did not move the one measured dimension. The variant's fitness vector reports only hit-rate, so metadata-size and per-op-time, which the hypothesis asked to be reported alongside it, have no evidence here. I could not inspect the overlay source in this session, so I cannot confirm from the code whether the decrement change landed as specified.

# Variant v003 Insight

## Helped

- (none) No dimension improved. Lowering the freq cap from 3 to 2 did not produce the hoped-for faster demotion of stale main-queue entries.

## Hurt

- hit-rate: Capping freq at 2 gives hot entries fewer reinsertion chances in main eviction, so some reused items are evicted too early. The result is slightly below the baseline on workload-a at size 2048.

## Takeaway

Reducing the freq cap is a small change that cost a little hit-rate on this trace. It joins the cycle-2 halving experiment as evidence that this workload gains from keeping reuse history, not from ageing it faster. Only hit-rate was measured for this variant, so the cap's effect on metadata-size and per-op-time is unknown. The 2-bit counter does not shrink at a cap of 2, so little change would be expected there.

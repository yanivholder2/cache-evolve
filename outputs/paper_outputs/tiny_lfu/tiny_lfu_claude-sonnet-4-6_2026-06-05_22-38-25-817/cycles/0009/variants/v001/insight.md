# Variant v001 Insight

## Helped

- hit-rate: Lowering `PROMO_THRESHOLD` from 8 to 6 allowed items with true frequencies 6–7 — which the accurate 8×-width sketch now estimates correctly rather than inflating past 8 — to enter the protected segment instead of stalling in probationary.

## Hurt

## Takeaway

The hypothesis was directionally correct: the cohort of medium-hot items previously blocked by the threshold=8 gate does exist and does benefit from the lower threshold under the collision-reduced sketch, producing a small but positive hit-rate gain. The gain is marginal rather than substantial, indicating that the frequency band 6–7 represents only a thin slice of the workload-a access distribution — the gate miscalibration was real, but the affected cohort is not large enough to deliver a meaningful uplift on its own.

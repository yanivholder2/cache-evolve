# Cross-Cycle Insights

## Recurring wins

- **Conservative, modest adjustments aligned with observed access patterns**: Cycle 0001's single-level frequency increase (MAX_FREQ 3→4) succeeded by matching the tuning to moderate-reuse workload behavior. This contrasts with aggressive re-weighting, suggesting the system's parameters are tightly calibrated and small, deliberate shifts in the right direction compound.

- **Structural selectivity in admission gates**: Cycle 0002 pivoted from frequency depth to small-queue gate tightening (1/4 → 1/5 capacity), delivering clear hit-rate gains. This suggests that controlling *who competes* for main-queue slots is now a higher-leverage lever than absolute frequency depth.

## Recurring failures

- **Ghost-hit frequency crediting**: Both cycles consistently rejected approaches that grant frequency protection based on prior ghost-hit survival. v002 failed in Cycle 0001 (freq=2 on re-admission) and again in Cycle 0002 (freq=1 on re-admission). The pattern is clear: prior survival does not predict future reuse in this workload. The baseline approach—treating ghost hits as cold and requiring them to earn frequency through actual re-access—aligns better with access dynamics.

## Open questions

- **Structural selectivity saturation point**: How far can small-queue reduction go (1/6, 1/7) before hitting a utilization cliff where the queue becomes too constrained to absorb working-set diversity?

- **Frequency-depth diminishing returns**: Why does the frequency-depth axis show rapid saturation (3→4 was significant; 4→5 marginal)? Is this inherent to the workload or do promotion thresholds *within* the small queue remain unexplored?

- **Workload-informed tuning**: Can reuse-depth distribution, access burstiness, or other workload characteristics predict optimal frequency depth, gate sizing, and promotion thresholds a priori, reducing future search space?

## Per-policy notes

- **S3-FIFO baseline**: Frequency balance is tightly calibrated; items must *earn* protection through reuse evidence, not queue history. Conservative increases in frequency depth show sustainable gains, while blanket frequency credits destabilize the system. The admission gate (small-queue capacity as a fraction of total) emerged in Cycle 0002 as the dominant tuning lever for this workload, suggesting the policy's selectivity window is the key bottleneck.

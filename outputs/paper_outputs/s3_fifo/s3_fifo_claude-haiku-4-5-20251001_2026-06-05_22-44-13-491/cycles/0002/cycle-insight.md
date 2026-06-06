# Cycle 0002 Insight

## Cohort outcome

The cohort tested whether frequency-depth scaling and structural admission-gate tuning could build on Cycle 0001's MAX_FREQ success. The winner (v003) pivoted from depth to structure: tightening the small queue admission gate from 1/4 to 1/5 of capacity. This structural selectivity outperformed continuing the frequency-depth axis (v001's marginal 4→5 gain) and confirmed that ghost-hit frequency credits remain a false lead (v002).

## Patterns

- **Admission selectivity now outperforms frequency depth**: v003's tight small-queue gating (1/5 capacity, enforcing stricter frequency filtering to promote to main) delivered clear hit-rate gains, while v001's extension of MAX_FREQ from 4 to 5 showed only marginal improvement. This inverts the Cycle 0001 lesson: frequency depth was the winning lever then, but structural parameters controlling *who gets to compete for main-queue slots* now deliver more value.

- **Frequency-depth dimension exhibits diminishing returns**: The gap between v001's improvement (4→5) versus Cycle 0001's (3→4) is markedly smaller, suggesting the frequency-depth axis is approaching saturation. The system may be approaching a regime where absolute reuse depth matters less than the selectivity of item admission.

- **Ghost-hit heuristics remain counterproductive**: v002's attempt to grant modest earned protection (freq=1 on re-admission) again failed to improve hit-rate, just as Cycle 0001's more aggressive variant (freq=2) did. The pattern is consistent: prior survival does not predict future reuse. The baseline approach—treating ghost hits as cold (freq=0) and letting them earn frequency through actual re-access—aligns better with the workload's access dynamics.

## Open questions

- How far can small-queue reduction go? Does the selectivity gain continue below 1/5, or is there a utilization cliff where the queue becomes too constrained to absorb working-set diversity?
- Why do ghost-hit frequency credits consistently fail? Is it that ghost hits in workload-a lack correlation with future reuse, or that the frequency mechanism cannot distinguish between "earned" and "gifted" protection?
- Can small-queue sizing be adaptive or workload-aware (e.g., scaled by reuse-depth distribution) rather than fixed structural constants?

## Continuation

Drop ghost-hit frequency crediting entirely—two cycles of evidence suggest it misses the workload's access patterns. Shift focus to structural selectivity: test further small-queue reductions (1/6, 1/7) to find the admission-gate sweet spot, and consider whether reuse-depth distribution or burstiness could inform optimal sizing *a priori*. Retire the frequency-depth axis (MAX_FREQ beyond 4-5 appears to yield diminishing returns); any remaining improvements likely come from admission gating or frequency promotion thresholds *within* the small queue, not absolute depth expansion.

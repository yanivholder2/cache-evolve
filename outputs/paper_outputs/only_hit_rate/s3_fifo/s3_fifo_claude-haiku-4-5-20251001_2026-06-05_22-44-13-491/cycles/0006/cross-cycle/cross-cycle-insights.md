# Cross-Cycle Insights

## Recurring wins

- **Structural selectivity via small-queue reduction dominates**: Seven consecutive cycles (0002–0006) show monotonic hit-rate improvement as the small-queue admission gate tightens from 1/4 → 1/12. This is unambiguously the highest-leverage optimization axis, outperforming both incremental steps and orthogonal frequency-depth compounding at every stage. The workload exhibits exceptional tolerance for extreme selectivity without utilization cliff.

- **Conservative frequency-depth scaling compounds orthogonally**: MAX_FREQ increases (3→4→5) deliver measurable but diminishing gains across cycles. Frequency-depth and structural selectivity are genuinely independent levers; they compound but remain unequally weighted, with structural selectivity outweighing frequency-depth by a wide margin in absolute impact.

- **Frequency-based promotion (freq > 1) filters effectively at scale**: Even at extreme selectivity (small-queue ratio 1/12), the frequency threshold separates genuine reusers from noise. Items demonstrating sustained access patterns successfully graduate to main-queue storage through this mechanism, indicating the workload's access patterns are skewed enough that coarse-grained frequency signals remain discriminative under severe under-provisioning.

## Recurring failures

- **Ghost-hit frequency credits consistently underperform**: Three variants across cycles (0001 v002 freq=2, 0002 v002 freq=1, 0003 v002 stricter threshold) all failed or hurt hit-rate. The pattern is clear: prior survival (whether tracked via ghost-hit re-admission or queue history) does not predict future reuse in this workload. Baseline treatment of ghost hits as cold (freq=0) aligns better with access dynamics.

- **Frequency-based selectivity at promotion gates fails repeatedly**: Both dynamic thresholds (Cycle 0001 v003 80% load-adaptive, Cycle 0003 v002 freq>2 graduation gate) showed no measurable gain or degraded performance. Frequency signals are too coarse-grained to drive *exit* decisions from the small queue; size-based *entry* filtering works, but frequency-threshold-based *promotion* filtering does not.

- **Orthogonal compounding consistently underperforms pure structural play**: Cycles 0004–0006 show that combining structural selectivity with frequency-depth scaling (e.g., v002 in 0004, v003 in 0005, v002 in 0006) delivers smaller absolute gains than pushing pure structural selectivity further. At current frontier (1/10+), frequency-depth remains complementary but marginal.

## Open questions

- **Where is the saturation boundary?** The trend holds unbroken to 1/12 with no cliff. Does reduction continue productively to 1/13–1/14, or does the boundary emerge suddenly? Saturation may be far tighter than expected given the workload's reuse clustering.

- **Do extreme structural selectivity and higher frequency-depth compound at the boundary?** Pairing 1/12+ with MAX_FREQ=5 or higher has not yet been tested. If saturation exists beyond 1/12, does tighter selectivity unlock greater frequency-depth compounding, or does frequency-depth remain marginal even at the boundary?

- **Can workload characteristics predict optimal small-queue ratios *a priori*?** Reuse-depth histograms, inter-arrival burstiness, or access-pattern skew metrics might allow future cycles to target saturation directly. This workload's tolerance for 1/12 suggests either exceptional clustering or architectural alignment with frequency-based filtering—understanding which could generalize insights to other workloads.

- **Would adaptive or learned admission policies exceed fixed structural ratios?** The consistent gains from increasingly selective filtering suggest the workload might benefit from dynamic admission policies that adjust selectivity per-phase based on observed reuse patterns, rather than fixed structural constants.

## Per-policy notes

- **S3-FIFO: Structural selectivity is the dominant tuning axis**: All cycles test variants of S3-FIFO with adjustments to small-queue capacity, MAX_FREQ, and promotion thresholds. The policy's frequency-based promotion mechanism effectively filters signal from noise even under extreme under-provisioning (1/12 capacity), indicating strong synergy with the workload's reuse patterns. Size-based admission gating (small-queue capacity) is far higher-leverage than frequency-threshold tuning at graduation points. Orthogonal axes (structural selectivity and frequency-depth) are real but unequally weighted; current frontier gains are entirely driven by structural selectivity.

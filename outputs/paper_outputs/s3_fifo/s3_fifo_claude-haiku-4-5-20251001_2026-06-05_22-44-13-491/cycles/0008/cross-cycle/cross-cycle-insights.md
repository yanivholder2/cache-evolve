# Cross-Cycle Insights

## Recurring wins

- **Structural selectivity (small-queue admission-gate tightening) is the dominant optimization axis across the search landscape.** Every cycle from 0002 through 0006 demonstrated consistent hit-rate improvements via incremental small-queue reduction (1/4→1/5→1/6→1/8→1/9→1/10→1/11→1/12), with aggressive variants consistently outperforming incremental steps and frequency-depth compounding. This progression held unbroken across seven consecutive cycles, establishing structural selectivity as the single highest-leverage tuning lever.

- **Structure–frequency compounding becomes the productive axis at the saturation boundary.** Cycles 0007–0008 show a phase transition: once admission-gate selectivity reaches extreme tightness (1/12–1/14), pairing it with higher MAX_FREQ values (5–6) delivers measurable gains and outperforms pure structural tightening. This reversal—from structure-dominant to structure-frequency-dominant—reveals that the optimization landscape is discontinuous and requires joint optimization near the boundary.

- **Modest, conservative frequency-depth scaling is viable but subordinate.** Cycle 0001's success with a single-level frequency increase (MAX_FREQ 3→4) demonstrated that conservative frequency-depth adjustments aligned with workload reuse patterns can improve performance. However, extending depth further remains productive only when paired with tight structural selectivity at the saturation boundary; in isolation, frequency-depth shows diminishing returns.

## Recurring failures

- **Ghost-hit frequency crediting consistently fails to improve hit-rate.** Cycles 0001–0002 both tested variants granting frequency credits on re-admission (Cycle 0001 v002: freq=2 on ghost-hit re-admission; Cycle 0002 v002: modest earned protection), and both underperformed the baseline. Treating ghost hits as cold (freq=0) and requiring them to earn frequency through actual re-access aligns better with the workload's access patterns; prior survival does not predict future reuse.

- **Frequency-based promotion gating (raising frequency thresholds for small-to-main transitions) hurts performance.** Cycle 0003 v002 tested stricter frequency thresholds (freq>1→freq>2) for promotion and degraded hit-rate compared to size-based selectivity. The distinction is mechanistic: tightening entry-gate capacity filters effectively; tightening graduation thresholds does not. Frequency signals at the promotion stage are apparently too coarse-grained or misaligned with the workload's reuse clustering.

- **Pure incremental structural reduction yields diminishing returns at saturation.** Cycles 0007–0008 show that once small-queue selectivity approaches 1/12–1/14, further incremental tightening (e.g., 1/13 alone) yields only marginal gains (+0.0003, +0.0002). This pattern narrows the saturation boundary and confirms that within the boundary zone, frequency-depth protection becomes the higher-leverage secondary axis.

## Open questions

- **Where is the precise saturation cliff in the structure–frequency space?** Cycle 0008's 1/14+MAX_FREQ=5 still improved hit-rate, but the true utilization collapse boundary remains unlocated. Does saturation occur beyond 1/14 selectivity, or can even tighter structural selectivity (1/15, 1/16) compound with deeper frequency protection (MAX_FREQ=6, MAX_FREQ=7)?

- **Is there a peak in the structure–frequency interaction surface, or does compounding scale monotonically?** Cycle 0008 suggests gains persist at both (1/14, MAX_FREQ=5) extremes. Finer-grained search near and beyond the current best may reveal whether a sharp optimum exists or whether the trade-off frontier extends further.

- **Can workload characteristics (reuse-depth histogram, access-pattern skew, burstiness) predict the optimal structure–frequency pairing *a priori*?** Across cycles, insights hypothesize that introspection into reuse clustering or distribution could predict saturation boundaries and optimal parameters without iterative search. This remains untested and could accelerate future cycles on different workloads.

- **Would adaptive or learned admission policies outperform fixed structural constants?** The workload's transition from structure-dominant to frequency-dominant optimization suggests benefit from dynamic policy selection or adaptive boundaries per-phase. This hypothetical direction has not been explored empirically.

## Per-policy notes

- **S3-FIFO (all cycles).** The baseline and all variants tune S3-FIFO's two primary levers: small-queue admission-gate capacity (controlling entry selectivity) and MAX_FREQ (controlling main-queue retention depth). The policy's frequency mechanism is tightly calibrated; improvements require either modest conservative tuning (Cycle 0001's MAX_FREQ 3→4) or strategic pairing at saturation boundaries (Cycles 0007–0008's structure–frequency compounding). The policy rejects unearned frequency protection (ghost-hit credits) and size-based admission selectivity is more effective than frequency-based promotion gating. The workload exhibits strong enough reuse clustering to sustain extreme selectivity (1/14 ratio) when paired with appropriate frequency protection, indicating the policy's frequency-based filtering remains effective even under severe under-provisioning.

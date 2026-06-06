# Cycle 0010 Insight

## Cohort outcome

Three hypotheses targeted the extreme-selectivity frontier: pushing structural selectivity to 1/16, deepening frequency protection to MAX_FREQ=7 at proven 1/14 selectivity, and pairing the prior cycle's two independent winners (1/15 structure + MAX_FREQ=6 frequency). The two winners—the tighter structural variant and the compound pairing—demonstrated that gains remain available within the saturation zone, but with sharply asymmetric returns. The loser revealed a critical boundary: frequency-depth protection exhausts before structural selectivity does, with MAX_FREQ=7 marking the saturation cliff.

## Patterns

- **Structural selectivity remains productive at 1/16 despite extreme diminishment**: v001's extension beyond 1/15 to 1/16 achieved measurable gains, continuing the pattern of zero degradation across all three cycles at the saturation boundary. However, the improvement magnitude shrinks further—now the smallest seen in the structural-selectivity axis across all ten cycles. This confirms that the workload tolerates extreme admission filtering even at 1/16, but the leverage diminishes exponentially; further tightening to 1/17 or beyond is increasingly unlikely to justify the effort.

- **Frequency-depth saturation boundary localized between MAX_FREQ=6 and MAX_FREQ=7**: v002's loss, combined with the progression of shrinking gains (+0.0014 at MAX_FREQ=5→6 in Cycle 0008, +0.0003 at MAX_FREQ=6 in Cycle 0009, +0.0001 attempted at MAX_FREQ=7 in Cycle 0010), pinpoints the frequency axis hitting a hard cliff. The binding constraint is now the extreme structural gate itself: items curated so aggressively through 1/14 selectivity are so few and high-value that extending their tenure ceases to yield meaningful benefit. Frequency depth entered saturation earlier than structural selectivity.

- **Pairing wins when the interaction respects the workload's reuse clustering**: v003's success with 1/15+MAX_FREQ=6—despite v003's own insight noting internal tension between aggressive upstream filtering and generous downstream protection—reveals a critical principle: in a workload with pronounced reuse clustering, the gain from filtering out noise upstream (1/15) outweighs the loss of reducing the pool benefiting from extended tenure (MAX_FREQ=6). The pairing works because the reuse distribution is so skewed that even fewer items pass the tighter gate, but those that do are so valuable that MAX_FREQ=6 tenure is sufficient to recapture them repeatedly. This contrasts with moderate-structure experiments (Cycles 0007, 0009) where aggressive frequency could not offset poor structural choices, confirming the surface is sharply peaked in the extreme-selectivity region.

## Open questions

- **Is 1/16 the true structural saturation ceiling, or does 1/17–1/20 remain marginally productive?** The trend suggests yes, but microscopic gains at 1/16 leave ambiguity about where degradation first appears.

- **Can pairing even tighter structural selectivity (1/16 or 1/17) with MAX_FREQ=6 or MAX_FREQ=5 unlock additional gains?** v003's success with 1/15+MAX_FREQ=6 hints the interaction space may extend further; does 1/16+MAX_FREQ=6 exist as a better pairing, or does 1/16 require MAX_FREQ=5 to remain competitive?

- **What workload characteristics predict this extreme-selectivity, moderate-frequency regime?** Across four cycles, this workload has consistently rejected moderate-structure approaches and demanded extreme admission filtering paired with moderate (not minimal, not maximal) frequency protection. Do access-pattern metrics (reuse depth histogram shape, inter-arrival clustering) uniquely identify this class of workloads?

## Continuation

The structure–frequency optimization surface has plateaued at the extreme-selectivity frontier (1/15–1/16 structure, MAX_FREQ=5–6 frequency); further refinement in either dimension alone yields diminishing returns below the noise floor. Do not push structural selectivity beyond 1/16 or frequency depth beyond MAX_FREQ=6 without new hypotheses. Instead, test finer-grained pairings in the interaction space one or two steps tighter than current winners (e.g., 1/16+MAX_FREQ=6 if not yet tried, or 1/16+MAX_FREQ=5 to confirm structure dominates) to map the true optimum if it exists. If these tests yield saturation without improvement, pivot to workload characterization: extract the reuse-depth histogram, access-pattern clustering metrics, and phase transitions from raw logs to test whether the extreme-selectivity requirement is *predictable a priori* from access signatures. Once predictability is established, consider learned or adaptive policies that dynamically adjust selectivity and frequency based on observed miss-rate evolution in real-time, rather than fixed constants—this workload's convergence on a narrow optimal region suggests it may be amenable to phase-adaptive tuning.

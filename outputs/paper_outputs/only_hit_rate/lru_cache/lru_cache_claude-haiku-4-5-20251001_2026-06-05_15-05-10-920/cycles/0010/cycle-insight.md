# Cycle 0010 Insight

## Cohort outcome

This cycle tested three hypotheses against Cycle 0009's declared saturation point: whether pure LRU eviction (frequency elimination) is actually necessary (v001, hurt), whether full 8-bit frequency precision outperforms 3-bit quantization (v002, helped—winner), and whether threshold-based frequency amnesia beats smooth multiplicative decay (v003, helped). v002's win reverses Cycle 0009's conclusion that precision is irrelevant; both v002 and v003 demonstrate that the baseline's decay mechanism interacts with frequency signal design in ways that were previously misunderstood, reopening the optimization window.

## Patterns

- **Quantization in Cycle 0009 was a lossy step backward, masked by "neutral" framing.** Cycle 0009 v002's introduction of 3-bit bucketing was declared neutral because it matched the baseline; Cycle 0010 v002 now reveals that full 8-bit precision beats the quantized version. The Cycle 0009 insight's claim that "decay compresses frequency into such a narrow range that precision is irrelevant" misattributed quantization loss to an inherent property of decay. The 0.75× multiplicative decay works equally well (and yields better victim selection) with full 8-bit frequency values, indicating the range compression was an artifact of bucketing, not a fundamental result of the decay mechanism. Precision recovered.

- **Two independent approaches to frequency signal redesign both beat the baseline.** v002 (full precision with smooth decay) and v003 (coarse buckets with threshold amnesia) represent orthogonal optimization axes: granularity (continuous frequency values vs. quantized tiers) and boundary shaping (smooth proportional decay vs. sharp signal/noise cutoff). Both improve hit-rate, suggesting the baseline's combination of coarse-grained tracking and smooth decay is suboptimal on both dimensions. This is not a local-optimum plateau but a point on a multi-axis surface where room for improvement remains.

- **v001's decisive failure closes the frequency-necessity door with finality.** Pure LRU without frequency tracking regresses hard (0.6652), conclusively confirming that frequency discrimination is load-bearing and not redundant to the decay mechanism. This validates that the baseline's frequency counters are not overhead but essential signal. The test was necessary to rule out the possibility that decay-driven recency ordering alone could explain the cache's performance.

- **Cycle 0009's saturation conclusion was premature.** After declaring that decay refinement was exhausted and eviction optimization had converged, Cycle 0010 immediately shows two competing approaches that both exceed the supposed saturation point. The saturation was not real; it was a consequence of testing only within the quantized-bucket constraint. Removing that constraint reveals that both precision and decay-pattern choices remain tunable.

## Open questions

- Does v003's threshold value (freq ≥ 2 retained, freq < 2 amnesia) represent the optimal boundary, or is it a heuristic that could be tuned further? What is the sensitivity of threshold-based amnesia to the cutoff value?

- Can v002's precision gain and v003's thresholding gain be combined—i.e., does 8-bit precision with threshold amnesia outperform both independently? This would test whether they address orthogonal inefficiencies or whether they compete for the same optimization budget.

- Why does the baseline's 3-bit quantization remain competitive (Cycle 0009 neutral) when explicitly compared to itself, but regress when directly compared to 8-bit precision? Is the "neutral" finding in Cycle 0009 an artifact of the test design, or do quantization benefits appear only under specific decay patterns?

- Does the workload contain phase structure that could be exploited by the sharp working-set boundary of threshold amnesia, or is the gain purely from signal/noise separation independent of temporal phases?

## Continuation

The optimization window has reopened; saturation was misdiagnosed. v002 becomes the new incumbent: upgrade from 3-bit quantized bucketing to full 8-bit frequency tracking while maintaining 0.75× multiplicative decay at capacity × 16 intervals and two-pass LRU victim selection. Quantization is confirmed lossy; retire it. 

For Cycle 0011, prioritize testing combinations: (1) **8-bit precision + threshold amnesia** to determine if v002 and v003's gains compound, and (2) **threshold sensitivity analysis** if amnesia proves valuable, to optimize the cutoff boundary. If combined 8-bit + threshold outperforms both independently, that becomes the new baseline and opens further refinement (decay coefficient, interval, threshold tuning). If they compete and one dominates, lock in the winner and explore why. v001's failure is conclusive—do not revisit frequency elimination. The cache remains amenable to optimization; the next cycle will clarify whether precision or thresholding (or both) is the correct path forward.

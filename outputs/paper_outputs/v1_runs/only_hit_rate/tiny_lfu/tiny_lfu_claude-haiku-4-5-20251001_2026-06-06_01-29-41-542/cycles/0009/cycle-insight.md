# Cycle 0009 Insight

## Cohort outcome

Three variants tested refinements to the frequency-cohort partitioning threshold introduced in Cycle 0008 as a breakthrough, plus an alternative partitioning signal. All three regressed: tightening the threshold to 15th percentile over-restricts the victim pool, broadening it to 40th percentile over-includes warm items, and swapping to age-based partitioning reveals frequency is the stronger primary discriminator. The cohort confirms the 25th-percentile frequency threshold is well-calibrated and locally optimal for this workload.

## Patterns

- **The 25th-percentile threshold is a precision-recall sweet spot**: v001 (15th percentile) lost availability of viable victims in the first pass, forcing fallback scans that settle for suboptimal candidates. v003 (40th percentile) lost selectivity by admitting warm-but-old items that shouldn't be evicted. The threshold balances two competing needs—ensuring enough cold candidates exist to find a victim without repeatedly falling back, and remaining selective enough to exclude moderately-warm items. This balance was not an artifact of the cycle-0008 baseline; it is structurally optimal for this trace.

- **Frequency and age are orthogonal, but frequency dominates as the primary partitioning criterion**: v002's regression (age-first vs. frequency-first partitioning) confirms both signals are independently valuable—cycle 0008 showed age-cohort partitioning alone improved hit-rate. However, when forced to choose between them as the primary filter, frequency-based cold identification outperforms age-based staleness. Frequency more precisely identifies items unused at any point, while age conflates stale-but-recent-touched items with genuinely cold ones.

- **Fine-tuning the existing architecture yields diminishing returns**: Cycles 0006–0008 escaped locked optimums by shifting architectural paradigms (relaxation failed → signal addition failed → hierarchical selection succeeded). Cycle 0009 demonstrates that once a sound architecture (hierarchical frequency partitioning) is in place, parameter tuning within that frame converges quickly. The variance around the optimal threshold is narrow; small deviations in either direction degrade performance.

- **The current architecture is robust to reasonable variations**: Unlike prior cycles where formula tweaks or signal additions failed uniformly, this cycle's single winner (v001, despite regression) is a meaningful distinction: all three regressed, but frequency-cohort partitioning at 25th percentile appears to be a true local maximum, not a broad plateau. The architecture itself is sound; only the calibration parameters have tight tolerances.

## Open questions

- Can hierarchical partitioning—frequency first, then age-ranked within the cold cohort—compound the signals without sacrificing the availability that kept v001 viable in cycle 0008, or does layering flatten gains?
- Are other signals (burstiness, inter-arrival variance, request distance) viable secondary partitioning criteria that could improve discrimination within a frequency-cohort?
- Does the 25th-percentile optimum generalize to other workloads, cache sizes, and admission-gate thresholds, or is it trace-specific to tiny-lfu?
- Have we reached a local ceiling on hit-rate under this architectural family (hierarchical victim selection + admission gate), or does a fundamentally different cache policy exist?

## Continuation

The 25th-percentile frequency-cohort threshold is locally optimal; do not pursue further threshold tuning without evidence of workload drift or architectural changes. Do not swap the partitioning signal from frequency to age without layering—frequency is the stronger primary filter. Next cycle, if pursuing gains within this architectural family, test hierarchical partitioning (frequency then age within the cold cohort) to compound orthogonal signals, or explore whether secondary selection criteria (burstiness, inter-arrival time) can improve discrimination. Alternatively, test the current winner on other workloads and configurations to establish whether the optimum is robust or trace-specific to tiny-lfu. If diminishing returns persist, consider shifts to multi-level caches, adaptive policy mixing, or empirical characterization of the theoretical hit-rate ceiling on this workload.

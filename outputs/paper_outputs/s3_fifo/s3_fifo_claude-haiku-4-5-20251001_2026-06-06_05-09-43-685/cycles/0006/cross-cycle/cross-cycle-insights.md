# Cross-Cycle Insights

## Recurring wins

- **Occupancy-responsive thresholds decouple latency and hit-rate**: Cycle 0003/v003 achieved the only dual-metric improvement by making promotion thresholds responsive to main-queue occupancy (freq > 1 when spare, freq > 3 when full), delivering 28–43% latency gains without hit-rate erosion. This represents the strongest baseline across all cycles and remains unbeaten by subsequent refinement attempts.

- **Pre-computed ghost-queue state enables latency breakthrough**: Cycle 0006/v002 demonstrated that storing (K, frequency) tuples in ghost enables dramatic latency wins (49% faster puts) when coupled with a carefully tuned re-admission boost formula. This architecture outperforms later-generation threshold mechanics, suggesting ghost efficiency is a distinct frontier from threshold tuning.

- **Simple, threshold-level changes avoid critical-path overhead**: Cycle 0003's v003 succeeded by adjusting threshold *selection* without adding per-operation computation. All attempts to layer occupancy-awareness into structural mechanics (sizing, decay, age-checking) added measurable latency penalties that offset their theoretical benefits.

## Recurring failures

- **Per-operation overhead negates structural occupancy-awareness**: Cycles 0004–0006 consistently show that implementing occupancy-responsive logic in critical paths (occupancy ratio computation, dynamic resizing, age-ratio calculations) degrades both get and put latencies. The overhead cost of these checks exceeds the hit-rate benefits they unlock, making them inferior to threshold-level adaptation.

- **Ghost-frequency preservation without strict decay discipline corrupts admission signals**: Cycles 0005–0006 demonstrate that retaining frequency information in ghost without matching decay semantics leads to inflated re-admission weights that favor evicted items over genuinely hot ones. Loosening retention thresholds (v001) or fully preserving frequency (v003) both failed; the coupling between retention policy, decay schedule, and re-admission weight is too tight to tune piecemeal.

- **Frequency distribution is a less stable signal than occupancy**: Cycle 0004/v002's attempt to use frequency percentiles as threshold signals proved less robust than occupancy-based signaling. Frequency distribution requires workload-specific calibration and is sensitive to access-pattern shifts; occupancy remains the more direct, workload-agnostic observable.

- **Simple queue-sizing does not recover hit-rate if the admission filter is the binding constraint**: Cycle 0002 showed that expanding small-queue capacity (25%→30%) without loosening the promotion threshold actually reduced hit-rate by increasing eviction churn. The admission filter's restrictiveness is the bottleneck; structural capacity adjustments alone cannot overcome it.

- **Age-based promotion rescue does not match frequency-starvation patterns in Zipfian workloads**: Cycle 0006/v003's hypothesis that items aging out of the small queue deserve looser promotion gates failed in practice. Either the starvation scenario is rare in typical access patterns, or the heuristic (loosening for back-30% of small queue) misses the vulnerable population.

## Open questions

- **Can v002's (cycle 0006) re-admission boost become adaptive?** The fixed `ghost_freq + 3` formula delivers latency wins but overshoots hit-rate recovery. Can occupancy-responsive or frequency-percentile-aware boost scaling preserve latency while tuning hit-rate to match or exceed cycle 0003/v003?

- **What decay depth optimizes hit-rate and latency in ghost-queue retention?** Cycles 0004 and 0006 tested frequency decay but with limited depth exploration (-2 threshold to discard). Would -1 (aggressive) or -3 (lenient) yield better trade-off points?

- **Is cycle 0003/v003's balanced performance a true local optimum?** All subsequent attempts (cycles 0004–0006) failed to improve upon it. Does the search space contain unexplored parameter combinations (occupancy thresholds, decay rate schedules, re-admission formulas) that could break the constraint frontier, or is further optimization futile without new mechanisms?

- **Can recency or hybrid frequency-recency scoring supplement current occupancy signals?** Cycles 0003–0006 relied on occupancy and frequency; recency weight or multi-signal scoring might unlock hit-rate recovery without per-operation overhead.

- **Do occupancy-aware mechanics have low-overhead structural alternatives?** Sampling-based tracking, lazy evaluation, or amortized computation might reduce per-operation cost of occupancy awareness without sacrificing the hit-rate benefits, but none have been tested.

## Per-policy notes

- **Admission filtering via frequency thresholds**: Tight thresholds (freq > 2–3) consistently improve latency but erode hit-rate. Occupancy-responsive thresholds (cycle 0003) break this trade-off by loosening when space permits. Fine-grained threshold tuning (0.4, 0.7 occupancy breakpoints in cycle 0006/v001) yields negligible marginal improvement, suggesting cycle 0003's parameters (0.5, 0.8) are well-calibrated for this workload.

- **Ghost-queue management**: Structural separation (cycle 0003/v001) failed; ghost queues are more effective when integrated into main admission logic. Frequency decay in ghost is mechanically sound but brittle—decay rate and re-admission boost must be tuned together. Pre-computed (K, frequency) tuples (cycle 0006/v002) outperform decay mechanisms on latency but require adaptive re-admission formulas to preserve hit-rate.

- **Queue sizing and capacity**: Expanding capacity without loosening admission thresholds is ineffective (cycle 0002). Dynamic sizing based on occupancy (cycle 0004/v003, cycle 0005/v002) recovers hit-rate at per-operation latency cost. Trade-off between responsive sizing overhead and hit-rate gain is unavoidable; threshold-level adaptation remains more efficient.

- **Eviction timing and aging**: Lazy eviction (cycle 0002/v003) and age-aware promotion (cycle 0006/v003) both address secondary bottlenecks after admission filtering is accounted for. Neither unlocked expected benefits, suggesting admission-filtering tightness, not eviction frequency or age distribution, is the primary constraint.

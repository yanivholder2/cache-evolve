# Cross-Cycle Insights

## Recurring wins

- **Direct measurement beats architectural workarounds.** Cycles 0001–0002 improved hit-rate by tuning sketch parameters to reduce collision error; attempts in Cycles 0006–0007 to work around noise via dual-tier fast paths, collision heuristics, or discretization all regressed. Fixing the signal source (wider sketch, better hardware) produces better gains than clever logic to compensate for noisy estimates.

- **Simplicity and directness outperform parameterized refinement.** Linear hybrid victim scoring (age / (frequency + 1)) beat sqrt-dampening, relative-frequency normalization, frequency tiering, and constant-offset tweaking. The pattern holds across Cycles 0003–0007: straightforward metrics are more robust than elaborated variants. Architectural changes (e.g., partitioning in Cycle 0008) can unlock gains, but within a fixed architecture, simplicity is a win condition.

- **Frequency and age are orthogonal signals; both are essential.** Cycle 0005's removal of age signal caused 0.31% hit-rate loss despite refined frequency estimates, proving temporal locality and frequency capture different aspects of working-set membership. Cycle 0008 confirmed both remain valuable even in hierarchical selection methods. They cannot be substituted or collapsed into a single metric.

- **Deterministic, hierarchical partitioning breaks locked optima.** Cycles 0006–0007 showed global continuous scoring was resistant to all refinements. Cycle 0008's shift to frequency-cohort-first partitioning (cold items first within the victim pool) broke through, improving hit-rate where formula tweaks and signal additions had failed. The plateau was architectural, not fundamental.

## Recurring failures

- **Relaxing constraints via threshold loosening never improves hit-rate.** Gate relaxation from strict `>` to `>=` (v003 Cycle 0001, v001 Cycle 0006) caused regression both times, under different eviction policies. The strict threshold is not an artifact of simpler eviction; it remains optimal as other policies improve. Tied frequency estimates should evict, not admit marginal candidates.

- **Adding signals without proper grounding introduces conflict and noise.** Size penalties (v002 Cycle 0006) overrode frequency when small hot items were in cache. Time-based admission (v003 Cycle 0006) lacked temporal grounding (unbounded elapsed time). Burstiness tracking (H01 Cycle 0007) and collision-detection heuristics (H03 Cycle 0007) added complexity without measurable gain. New signals require design orthogonal to existing ones or fail.

- **Discretizing continuous signals loses discrimination.** Frequency tiering (H02 Cycle 0007) grouped items into percentile buckets to handle estimation noise, but lost the fine-grained discrimination that the simple continuous formula preserved. Similarly, size-based bucketing in victim selection conflicted with the smooth frequency-age trade-off. Continuous scoring naturally handles uncertainty better than thresholding.

- **Parameter tuning on a locked architecture yields zero gains.** Cycles 0004–0005 tested offset and exponent adjustments to the linear hybrid formula; neither produced measurable improvement. Once an architectural choice stabilizes (global continuous scoring), local parameter refinement becomes futile. This pattern precedes Cycle 0008's breakthrough, which required changing the architecture itself.

## Open questions

- **Does the hierarchical partitioning approach from Cycle 0008 generalize?** The frequency-cohort method (25th percentile, cold-first) won decisively, breaking a six-cycle plateau. Can finer percentile thresholds improve further? Can frequency and age partitioning be layered (e.g., oldest items within the cold-frequency cohort)? How does this transfer to other workloads, cache sizes, or admission thresholds?

- **What is the true performance ceiling on this workload?** After eight cycles of tuning, hit-rate improvements have become measurable but narrow. Cycle 0008 unlocked new gain, but it remains unclear whether the partitioned approach approaches asymptote or has runway. Measuring the theoretical upper-bound hit-rate and gap-to-ceiling would clarify whether further policy refinement is productive.

- **Are other cache-infrastructure changes needed?** Sketch tuning (width, depth) saturated by Cycle 0002. Victim-selection logic (formulas, signals) plateaued by Cycle 0007. Partitioned selection broke that plateau in Cycle 0008, but can single-policy tuning fully exploit the remaining headroom? Multi-level caches, adaptive policy mixing, or alternative data structures might unlock further gains.

- **What transfers and what remains workload-specific?** The tiny-lfu trace exhibits steep frequency distribution and strong temporal locality. The strict gate, frequency-age hybrid, and now hierarchical partitioning all succeeded here. Do these findings hold for other access patterns (uniform, hot-set, bursty, scan-heavy), or is each workload locally optimal under different policies?

## Per-policy notes

**Sketch parameter tuning (Cycles 0001–0002, tested further in Cycle 0007)**
- Width expansion: 4→8x (+0.0047 gain), 8→16x (+0.0006 gain) shows classic diminishing returns; pure width scaling has saturated.
- Depth: orthogonal to width; 6-row variant improved modestly, confirming depth is a productive but secondary tuning axis.
- Saturation recycling (Cycle 0003): no measurable impact, suggesting this workload either avoids saturation or the trade-off (lost precision from right-shift) negates gains.
- Collision heuristics (Cycle 0007): failed despite theoretical motivation; simpler minimum-based estimates remain most effective.
- **Conclusion**: Sketch parameters are tuned for this trace. Further frequency-estimation gains require alternative structures, not Count-Min refinement.

**Victim selection: global continuous scoring (Cycles 0003–0007)**
- Linear hybrid (age / (frequency + 1)): simple, stable, beat all refinements within this architecture.
- Parameter tuning (Cycles 0004–0005): offset and exponent adjustments produced zero change; the formula is locally optimal on this workload.
- New signals (Cycle 0006–0007): size, burstiness, time-based terms all failed or regressed; frequency and age are sufficient for continuous scoring.
- Discretization attempts (frequency tiering): lost discrimination; continuous metrics handle noise better.
- **Conclusion**: Global continuous scoring is locally locked. This architecture cannot be improved via formula or signal tweaking; architectural change is required.

**Victim selection: hierarchical partitioning (Cycle 0008)**
- Frequency-cohort (25th percentile, cold-first): broke the prior plateau with measurable improvement; simplest partitioning method, deterministic.
- Age-cohort (oldest items first): also improved, but frequency-cohort won; frequency more directly identifies unused items than age.
- Randomization: regressed 1%; determinism and ordering are essential; deterministic scan exploits cache locality and reduces computation for frequently-evicted candidates.
- **Conclusion**: Partitioned selection is the new baseline. Opens new optimization frontier for threshold refinement, multi-level partitioning, and signal integration within cohorts.

**Admission gate**
- Strict `>` threshold: tested repeatedly, always optimal (v003 Cycle 0001, v001 Cycle 0006 under different eviction policies); not a bottleneck under any tested approach.
- Threshold relaxation (>=): regresses hit-rate by allowing tied frequency estimates to admit marginal candidates; strictness is necessary discriminator, not a limitation of eviction quality.
- Time-based and signal augmentations (Cycle 0006): failed; gate remains frequency-only and simple.
- **Conclusion**: Admission gate is stable and optimal; do not revisit threshold or signal design.

# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0006

## Winners

- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0006/hypotheses/H03.md

## Prior cycles digest

### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants probed distinct structural axes — ghost compression, promotion threshold relaxation, and second-chance credit depth — and none improved hit rate; v002 (freq > 0 threshold) was the least harmful and is the nominal winner, but the cycle effectively confirms boundaries rather than advances the configuration.

## Patterns

- **All three axes hit limiting cases simultaneously.** Halving the ghost cap again crossed below the workload's re-use horizon (v001); widening the promotion gate was safe but found no eligible traffic to benefit from it (v002); tightening MAX_FREQ cut off third-pass retention that live hot items actually exercised (v003). The cohort collectively mapped the outer edges of three independent knobs in a single cycle.
- **main_size/2 is a hard floor for ghost capacity on this workload.** The re-use distances in the trace extend into the 461–922-eviction range; any ghost window shorter than main_size/2 discards genuine signal, not noise. This boundary is now experimentally confirmed from both sides (expanding hurt in cycle 0001; shrinking hurt here).
- **MAX_FREQ = 3 is the functional lower bound for second-chance depth.** The hot set is temporally sticky enough that some fraction of items legitimately earns and uses a third eviction-pass reprieve. This also establishes that second-chance depth has unexplored upward space (≥ 4).
- **The freq > 0 admission gate is safe but inert alone.** The narrow small queue prevents a quality flood, but the workload simply does not carry enough once-accessed-then-reused traffic to translate wider admission into observable gains. This change is not harmful and could be revisited in combination with other structural changes.

## Open questions

- Does raising MAX_FREQ above 3 (e.g., 4 or 5) retain additional hot-set items long enough to improve hit rate, or does it instead delay eviction of moderately warm items and hurt?
- With the freq > 0 admission confirmed neutral, could it interact positively with a higher MAX_FREQ — admitting more items into main while giving them a longer retention window?
- Are there axes entirely unexplored so far (e.g., eviction ordering within main, hit-count reset on ghost re-admission) that could yield gains now that queue sizing and threshold knobs are largely exhausted?

## Continuation

The cycle 0002 winner remains the best-known configuration; next cycle should treat it as the baseline and stop revisiting ghost sizing or small-queue ratios, which are well-characterised. The most promising unexplored axis is MAX_FREQ upward (≥ 4): the evidence that 3 is a lower bound implies the upper direction is worth probing. A secondary experiment worth attempting is combining the now-safe freq > 0 promotion threshold with a higher MAX_FREQ, since the two changes interact structurally — wider admission into main is more viable when retention depth gives newly admitted items time to prove their worth.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants probed retention depth and admission quality: raising the second-chance frequency ceiling (v001), widening small-queue admission (v002), and granting ghost-re-admitted items a head-start credit (v003). Only the ceiling raise won, and the gain was real but modest.

## Patterns

- **MAX_FREQ = 4 yields a genuine but incremental gain.** A non-trivial minority of items reach the old ceiling and benefit from the extra retention pass, confirming the upward direction is viable — but the small magnitude of the improvement suggests this minority is limited, and proportional returns from further ceiling increases are unlikely.
- **The freq > 0 admission gate is definitively closed.** Tried twice (neutral solo in cycle 0003, harmful here in combination with ghost precision), it never helps. Diluting main with single-access items costs more in churn than the marginal ghost-hit promotions can recover; the two mechanisms are orthogonal rather than synergistic on this workload.
- **Ghost re-admission credits are counterproductive.** Seeding re-admitted items at freq=1 displaces incumbents that earned their slots through genuine accesses. The ghost signal confirms re-use potential, but not enough to justify claiming a residency advantage over established main-queue residents.

## Open questions

- Is there a useful stopping point between MAX_FREQ = 4 and a higher ceiling (e.g., 5 or 6), or does the frequency distribution of hot items make any further step negligible?
- With admission thresholds and ghost sizing now well-characterised, are there structural axes entirely unexplored — such as eviction ordering within main, hit-count handling on ghost re-admission at freq=0, or adaptive queue sizing under phase changes?

## Continuation

Next cycle should adopt MAX_FREQ = 4 as the new baseline and make one cautious test of MAX_FREQ = 5, but with low expectations given the already-small delta from 3 → 4. The admission threshold and ghost credit directions are exhausted and should not be revisited. If the MAX_FREQ = 5 probe is also incremental or flat, the highest-yield path forward is likely a genuinely new structural axis — the queue-sizing and admission-threshold knobs have been mapped to their effective limits.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

This cycle probed three independent axes — frequency-ceiling extension, eviction-ordering strategy within main, and promoted-item frequency normalization — yielding one tiny positive (v001), one significant regression (v002), and one neutral result (v003). The winner advances the configuration by the smallest margin seen across all cycles.

## Patterns

- **The frequency-ceiling axis is now exhausted.** The 4→5 increment produced an even smaller gain than 3→4, and the v001 insight characterises the hot-set tail as sparse above the old ceiling. Each step yields a shrinking return; further increments are not a productive direction.
- **Move-to-front reprieve in main eviction is load-bearing recency sorting, not merely a second-chance gate.** The CLOCK-style in-place scan tried in v002 revealed that the baseline's positional churn continuously keeps recently re-accessed items away from the eviction frontier. Removing that implicit recency re-ordering — even while preserving the frequency check — caused a large regression. Eviction ordering is a first-class dimension of this cache's performance.
- **Promoted-item frequency normalization is inert on this workload.** Capping incoming main-queue items at freq=1 had no measurable effect, indicating that over-credited promotions are not displacing established hot residents in practice. This axis can be set aside.

## Open questions

- Since move-to-front is the primary recency-sorting mechanism, are there modifications to that behaviour (e.g., conditional repositioning above a frequency threshold, or partial repositioning) that could improve hit rate further without discarding the recency benefit?
- With queue sizing, admission thresholds, ghost sizing, frequency ceiling, and promotion normalization all characterised, what genuinely unexplored structural axes remain? Candidates include adaptive queue sizing under workload phase shifts, eviction tie-breaking within frequency bands, or behaviour specific to scan-heavy versus hot-set-heavy access phases.

## Continuation

Both the frequency-ceiling and promotion-normalization axes are now closed. The most actionable finding from this cycle is that eviction ordering within main is the most sensitive structural lever yet identified — the implicit recency sort embedded in move-to-front is the real mechanism behind the baseline's performance. The next cycle should treat this as the primary axis: explore whether selective or conditional repositioning during eviction can improve on the baseline's unconditional move-to-front, rather than continuing to chase diminishing returns on the well-mapped parameter knobs.


## Write

Exactly one file: `cycle-insight.md` in the output dir.

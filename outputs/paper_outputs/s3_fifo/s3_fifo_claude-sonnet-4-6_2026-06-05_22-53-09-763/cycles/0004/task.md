# Task: cycle-insight-writer

Cycle: 0004
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0004

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0004/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0004/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0004/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0004/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0004/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0004/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants targeted the balance between the small, main, and ghost queues; only the pure capacity reallocation (shrinking the small queue from 25% to 10%) won, while both attempts to loosen admission quality — lowering the promotion threshold and widening the ghost window — hurt hit rate.

## Patterns

- **Resident capacity beats admission breadth.** The winning change gave more slots to main without touching any admission logic. Both losing changes expanded what could enter main (more promotions, more ghost hits fast-pathed in) and degraded performance, suggesting main queue quality is more sensitive to entry criteria than to raw size pressure.
- **The main queue's second-chance mechanism is the critical bottleneck.** When lower-quality entries flooded in (v002, v003), second-chance eviction could not shed them fast enough to protect genuinely hot items. The frequency threshold (`freq > 1`) and a right-sized ghost window act as upstream guards that second-chance alone cannot replace.
- **Ghost queue value is precision, not coverage.** Extending the ghost window introduced incidental re-accesses that bypassed the small-queue filter. A longer history does not improve signal quality for this trace; it introduces noise from distant, non-recurring accesses.

## Open questions

- Is 10% the right floor for small queue size, or does further shrinking (e.g., 5%) keep improving — and does the answer change under scan-heavy or more bursty phases?
- Could a *smaller* ghost queue (below the baseline `main_size` cap) improve precision further, given that a larger one demonstrably hurts?
- Does the winning small=10% configuration shift the optimal promotion threshold, or does `freq > 1` remain correct regardless of small-queue sizing?

## Continuation

Next cycle should build on v001's winning configuration (small=10%) and probe in two directions: first, whether fine-grained small-queue sizing around 10% (e.g., 8%, 12%) yields further gains; second, whether the ghost queue has an optimal size below the current baseline cap — the evidence from v003 suggests shrinking it is worth testing. Attempts to relax the small-to-main promotion threshold should be parked unless paired with a compensating mechanism to protect main-queue quality.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants probed two axes independently and in combination: further small-queue shrinkage and ghost-queue precision via a halved cap. The winner (v003) stacked cycle 0001's enlarged main queue with a tighter ghost window, confirming that both dimensions contribute independently to hit rate.

## Patterns

- **Ghost precision is a first-class lever.** Both variants that halved the ghost cap (v002 on the baseline, v003 on the cycle-0001 configuration) improved hit rate — the ghost truncation worked regardless of which small-queue ratio accompanied it. The benefit is not an artifact of the enlarged main queue; it is a property of the workload's re-use distance fitting inside a smaller window.
- **The two levers are complementary, not redundant.** v003's combined change outperformed v001 (small-only) and, by extension, the cycle-0001 winner. A larger main queue and a sharper ghost signal amplify each other: the enlarged main slows eviction churn, making the freshest evictions colder on average and therefore making ghost precision matter even more.
- **Small-queue shrinkage is hitting diminishing returns.** The 10% → 8.3% step in v001 produced a gain visibly smaller than the large 25% → 10% jump from cycle 0001. The marginal value of each slot transferred from small to main is declining, signalling a plateau in this direction.

## Open questions

- Does the ghost benefit continue below main_size/2 (e.g., main_size/4), or does the window become too narrow and miss genuine re-uses?
- With v003 as the new baseline, does further small-queue reduction (e.g., capacity/12 or capacity/14) still help, or have we exhausted small-queue tuning?
- The promotion threshold (`freq > 1`) has remained constant throughout; with a sharper ghost signal now filtering main-queue entry, is there headroom to explore `freq > 0` without flooding main?

## Continuation

Next cycle should treat v003 as the new baseline and probe ghost sizing more aggressively — specifically a ghost cap of main_size/4 — to find where ghost truncation tips from precision gain to signal loss. A secondary axis worth opening is whether the combination of the sharper ghost and the now-stable small-queue size creates room to revisit the promotion threshold, which has been untouchable until now. Further small-queue-only shrinkage is unlikely to be the highest-yield direction given the diminishing returns seen in v001.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.

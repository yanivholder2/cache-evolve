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

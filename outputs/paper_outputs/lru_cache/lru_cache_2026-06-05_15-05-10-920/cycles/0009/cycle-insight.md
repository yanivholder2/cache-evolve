# Cycle 0009 Insight

## Cohort outcome

This cycle tested three variations on the decay mechanism locked in by Cycle 0008: switching from periodic uniform decay to per-access activity-responsive decay (v001, regressed), quantizing 8-bit frequencies into 3-bit bucketed tiers (v002, neutral), and replacing fixed decay interval with adaptive hit-rate monitoring (v003, regressed). v002 emerges as the trivial winner by maintaining the baseline unchanged; all three variants either degraded performance or had zero measurable impact. This cycle confirms that decay mechanism refinement is exhausted: the 0.75× coefficient at capacity × 16 interval is definitively locked in, and no further parameter tuning, timing strategy, or precision trade-off will improve upon it.

## Patterns

- **Per-access decay fails by inverting the recency-frequency balance.** v001's discontinuous two-tier system (items accessed within 64 ops retain full frequency; older items decay sharply) recreates the exact failure mode of Cycle 0008's hybrid scoring: overweighting recency at the expense of cumulative frequency history. The uniform periodic decay mechanism has correctly calibrated the rate at which all items age together; making decay responsive to individual access timing breaks this global synchronization and privileges recent accesses over genuinely hot items. This is not a parameter-tuning failure but a fundamental incompatibility between activity-responsive and frequency-based eviction.

- **Frequency precision is irrelevant once decay compresses the range.** v002's neutral result confirms that the 0.75× decay compresses 8-bit counters into such a narrow operational range that both precise and quantized tracking converge to identical eviction behavior. This aligns with Cycle 0004's neutral finding on counter saturation; the workload's access patterns do not expose scenarios where fine-grained precision matters. Bucketing to 3 bits provides no benefit because coarse and fine granularity yield the same victim-selection ordering.

- **Adaptive mechanisms uniformly fail; fixed parameters have won.** v003's regression when adding hit-rate monitoring extends a multi-cycle pattern: Cycles 0005, 0007, 0008, and now 0009 have all attempted adaptive approaches (threshold-based decay, frequency quantization variance, hybrid scoring, and now hit-rate monitoring), and all have either failed or been neutral. The per-operation overhead of counters and periodic calculations does not justify the gains (none observed) from earlier decay triggering. The workload either lacks the phase transitions that adaptive mechanisms could exploit, or such transitions exist but are infrequent enough that fixed-interval decay already captures them.

- **Decay mechanism refinement is conclusively exhausted.** After Cycles 0006–0007 tuned the decay coefficient and interval, Cycle 0008 ruled out hybrid scoring with decay, and Cycle 0009 has now tested three orthogonal decay variations (timing model, precision, triggering strategy)—all have failed or regressed relative to the baseline. The cache has reached saturation at the decay level; no further variations on how, when, or with what precision decay occurs will yield improvement.

## Open questions

- Is saturation at the decay level equivalent to saturation across the entire eviction subsystem? Cycles 0006–0009 have exhausted decay tuning and victim-selection refinement; are there other orthogonal dimensions to explore (e.g., counter representation micro-tuning, cache sizing, or replacement of frequency-based eviction entirely)?

- Does the workload genuinely lack phase transitions, or are there adaptive strategies more sophisticated than hit-rate monitoring (e.g., entropy-based distribution shifts, frequency-distribution divergence) that could detect real transitions and justify the overhead?

- Should the next cycle accept structural saturation and focus on confirmation and micro-tuning, or pivot to entirely different optimization paradigms?

## Continuation

Lock in the Cycle 0008 baseline permanently with high confidence: 0.75× multiplicative decay at capacity × 16 intervals with two-pass LRU victim selection. Decay refinement is conclusively finished—per-access activity-responsive decay fails, frequency quantization provides no benefit, and adaptive interval triggering regresses. Stop all exploration of decay coefficient, interval, timing model, precision, and adaptation strategy. The mechanism is optimal for this workload.

For Cycle 0010, accept that eviction decision-making optimization has reached saturation. The cache has leveraged frequency discrimination, LRU ordering, and decay-driven phase adaptation to their limits. Future exploration must either: (1) pivot toward implementation-level micro-tuning (counter representation, decay triggering efficiency, data-structure layout) if such gains are available, (2) test entirely different eviction paradigms (e.g., non-frequency-based policies) to confirm frequency-based caching is the right strategy, or (3) formally declare saturation and close the optimization loop. The evidence strongly supports saturation; further cycles should confirm rather than assume.

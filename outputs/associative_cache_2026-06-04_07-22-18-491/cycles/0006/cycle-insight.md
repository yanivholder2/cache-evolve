# Cycle 0006 Insight

## Cohort outcome

All three variants tested different ways to layer on top of the Cycle 5 probe-zone baseline: full composition with the stash and LFU eviction, probe size reduction, and LFU eviction alone. Only the full composition won; the other two were neutral or a regression, confirming that the probe admission gate is the primary driver and that the stash and LFU eviction only add value together and on top of it.

## Patterns

- Probe admission and stash-based recovery are genuinely complementary: the probe zone blocks cold items from entering primary, while the stash intercepts warm items displaced from primary by cold bursts before they could accumulate more frequency. The combination is additive because they address non-overlapping failure modes.
- LFU-weighted eviction in isolation, once the probe filter is in place, is neutral. After probe-gated admission, every primary resident holds at least one confirmed access and the frequency distribution within primary is too flat to discriminate meaningfully. The eviction choice matters only when the stash can absorb the displaced item — LFU selects a better victim, and the stash limits the cost of any error, so together they improve outcomes whereas neither alone does.
- The probe zone has a minimum viable window: quarter-capacity ejects re-access candidates before their confirming hit arrives. Half-capacity is at or near the floor for this workload's inter-access gap, not a generous surplus.
- The winning variant brings together every mechanism validated across Cycles 3 through 6 into a single baseline: probe admission at half-capacity, FIFO stash at quarter-capacity, LFU-weighted eviction, and the plus-two promotion credit. The mechanism set now feels structurally complete on the admission and eviction sides.

## Open questions

- With the full composition now the baseline, is there any further structural axis to explore, or has the design converged to a local optimum for Zipfian workloads?
- Does probing above half-capacity yield any gain, or does the filter window become so wide that the overhead outweighs the admission benefit?
- The `metadata-size` evaluator gap has persisted through six cycles. The accumulated per-bucket memory overhead of probe, stash, and LFU counter fields is still entirely invisible to the fitness function; variants designed to trade memory for hit rate remain unverifiable.

## Continuation

Next cycle should branch from v001 and treat it as the new baseline. The most productive remaining axis is probe-zone sizing above half-capacity — testing three-quarters or full-primary-size probe cap would characterise whether the gain is monotone upward or whether half-capacity is genuinely optimal. If probe sizing proves exhausted, the next structural opportunity lies outside the probe-stash-LFU triangle entirely: admission policies based on key-space partitioning, time-windowed frequency, or workload-shift detection. The metadata-size infrastructure gap should be resolved before any further memory-efficiency hypotheses are written, as it has blocked verification for the entire run.

# Cycle 0010 Insight

## Cohort outcome

After four consecutive falsification cycles, this cohort produced two clean wins and one instructive tradeoff: v001 collapsed the ghost-membership probe into a unified state map (halving put latency), v003 raised `MAX_FREQ` to 4 (recovering hit-rate at zero latency cost), and v002 demonstrated that admission-sketch filtering cuts `evict_small()` invocations but introduces a hit-rate regression from warm-cold keys.

## Patterns

- **The two-probe put pattern was the dominant put-path cost, not eviction work itself**: Replacing the `data.contains_key` + `ghost_set.contains` pair with a single unified `state_map.get()` halved put p50 and p99. The ghost-set membership test on every `put()` — called out as an unexamined lever in cycle 0009 — was indeed the remaining movable cost, and eliminating the separate ghost structure resolved it cleanly. The struct-size reduction is a secondary bonus.
- **`MAX_FREQ=4` is a free hit-rate improvement over `MAX_FREQ=3`**: The extra frequency credit reduces premature eviction of medium-frequency entries without any detectable scan overhead. The `MAX_FREQ=5` regression from cycle 0007 was a threshold effect, not a smooth curve; 4 sits in a plateau where the benefit is real and the cost is below measurement resolution.
- **Admission selectivity is effective but workload-a is not a pure one-hit-wonder distribution**: The sketch filter confirmed that `evict_small()` call-rate is a genuine lever on put latency, but strict first-seen rejection over-filters — workload-a contains enough warm-cold traffic to produce measurable hit-rate loss. The direction is valid; the threshold is too aggressive.

## Open questions

- With put latency now halved by the unified map, are put p50 and get p50 now in the same order of magnitude? If get p50 remains the primary fitness signal and is still not moving, is it now at or near the implementation floor for this workload and algorithm?
- Can the admission-sketch threshold be softened (e.g., require two visits before skipping eviction, or use a shorter reset interval) to recover the hit-rate regression while retaining most of the `evict_small()` call-rate reduction?

## Continuation

Cycle 0010 breaks the falsification streak with two structural wins: the unified `state_map` (v001) and `MAX_FREQ=4` (v003) should both be carried forward as the new baseline. The next cycle should focus on refining the admission-sketch approach from v002 — specifically softening the rejection threshold from first-seen to two-visit, or decaying the sketch more aggressively, to close the hit-rate gap while keeping the eviction-rate reduction. A secondary thread worth opening is whether any remaining get-path cost exists now that put-path costs have been substantially reduced, or whether get p50 has genuinely reached the measurement floor. Do not revisit the split-map layout, `MAX_FREQ` values above 4, ghost `freq=1` initialisation, batch-drain eviction, or hasher swaps.

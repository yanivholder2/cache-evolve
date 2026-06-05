# Cycle 0004 Insight

## Cohort outcome

This cycle tested three refinements to the Cycle 0002 winner (periodic frequency decay at fixed `capacity * 16` interval): counter saturation capping (v001, neutral), more frequent decay at `capacity * 8` (v002, regressed), and smoothed multiplicative decay at 0.75× (v003, winner). The winning multiplicative approach reveals that the decay *curve shape* is more critical than decay frequency; the workload benefits from gentler frequency forgetting that preserves discrimination between hot and cold items across phase shifts.

## Patterns

- **Decay interval is robustly tuned.** Accelerating decay from `capacity * 16` to `capacity * 8` actively hurts hit-rate by forcing the cache to forget historical patterns too aggressively. This workload's phase shifts are gradual, not abrupt; the original interval balances frequency accumulation and adaptation correctly, and attempting to speed up adaptation backfires by destroying discriminative power before new patterns stabilize.

- **Decay curve shape outweighs frequency of decay.** The marginal gain from multiplicative decay (0.75× per cycle) over aggressive right-shift division (0.5×) demonstrates that smoother, less harsh forgetting preserves frequency discrimination longer. Items retain relatively higher frequencies across reset cycles, helping the cache reliably identify and protect truly hot items through workload transitions without losing track of historical patterns.

- **Counter saturation is not a bottleneck.** Capping frequency counters at 32 (vs. natural u8 255 saturation) has no measurable impact. The workload either does not generate frequency values near saturation, or the existing periodic decay mechanism refreshes priorities fast enough that an artificial lower cap adds no practical benefit.

## Open questions

- Does the 0.75× multiplicative decay coefficient generalize across workloads and cache sizes, or is this value tuned to this specific trace? How sensitive is the improvement to nearby coefficients (0.7×, 0.8×, 0.85×)?
- Could other decay patterns (exponential curves, piecewise linear decay, frequency-dependent decay rates) further improve hit-rate beyond the marginal gain observed here?
- Why does the right-shift decay (0.5×) lose discriminative power faster than multiplicative decay? Is this purely a function of the magnitude difference, or does the shape of the decay curve interact with the frequency distribution in non-obvious ways?

## Continuation

The multiplicative decay approach (v003) is the new winner and should become the baseline for Cycle 0005. The `capacity * 16` interval remains well-tuned; stop pursuing interval acceleration. **Explore decay curve variations** around the 0.75× multiplicative approach: test adjacent coefficients (0.7×, 0.8×, 0.85×) to see if there is a local optimum, and consider whether different decay patterns (e.g., linear decay, adaptive decay based on frequency distribution) might capture further gains. Abandon counter saturation as a lever; it is not constraining this workload. The current findings suggest the cache is tuning its "memory" via decay shape; the next cycle should probe whether this tuning window extends beyond multiplicative constants to other forgetting mechanisms.

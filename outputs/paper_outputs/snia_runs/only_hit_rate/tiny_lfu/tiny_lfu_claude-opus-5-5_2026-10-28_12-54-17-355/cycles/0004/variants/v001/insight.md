# Variant v001 Insight

## Helped

- hit-rate: hm_0 at the large size recovered past the parent. This is the one cell where a protected split that adapts per workload did what H01 predicted. At this size the sample period is long, so each protected turn scores the split against a stable hit-rate sample, and walking `protected_pct` a few points away from the fixed 80% appears to give hm_0's recency-driven reuse a bit more room. web_0 at the small size also edged up, which fits the climber nudging the split rather than leaving it fixed.

## Hurt

- hit-rate: hm_0 at the small size lost more than the large size gained. web_0 at the large size dipped just below the parent, which is the refutation threshold H01 set itself. The headline summary is therefore essentially flat. Three things in the code explain why the climber could not make the large move to "more probation" that the rationale relied on:
  - The step decays through `as isize` truncation (5→4→3→2→1), so within four protected turns the climber is reduced to 1-point moves.
  - The first turn always moves upward, because `prot_prev_hit_rate` starts at 0 and cannot trigger a reversal. That shrinks probation, the opposite of what hm_0 needed.
  - Alternating turns halves how often the window climber adapts.
- hit-rate: "Each climber compares only against its own previous sample" does not actually isolate the two knobs. The protected climber's reference sample is two periods old, and the window move from the turn in between is folded into its delta, and the same goes the other way. Each knob still gets credited or blamed for the other's moves, now with half the feedback rate.

## Takeaway

Making the protected fraction adaptive did not hurt, but this climber was too timid and too confounded to find a per-workload split. Truncating the step decay locks `protected_pct` into roughly ±10 points of 80 after a few turns. Its uncorrected first move goes in the wrong direction for hm_0. Alternating turns trades window-climber responsiveness for an isolation it does not actually provide. The gains and losses across traces and sizes look like noise from that drift, not a learned split. If the protected split is worth adapting, it needs one of these: a step that does not collapse to 1pp (float accumulator or a proper floor), direction seeded from the parent's hm_0 signal, or a signal that truly separates the two knobs, such as attributing the change to the knob that last moved within the same period, or a shadow/ghost estimate. Simply alternating turns is not enough.

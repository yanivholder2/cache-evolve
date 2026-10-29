# Variant v002 Insight

## Helped

(none: hit-rate did not improve on any trace or size)

## Hurt

- hit-rate: hm_0 dropped slightly at both sizes, so it missed the refutation thresholds of cycle 0004 v002 at both. The promoted block is already written at `m`, just behind the main hand, so it gets a full main rotation even with counter 0. The carried `1..=2` counter adds one or two more rotations. That mostly protects blocks whose only reuse came during probation. Main slots go to them instead of to blocks that earned their counters through hits in main.
- hit-rate: web_0 also dropped at both sizes, more at 131072. The hypothesis expected web_0 to stay flat, because one-touch scan blocks never leave probation. But web_0 blocks with exactly one short-range re-touch now enter main with non-zero counter mass. Each promotion sweep must then walk further and decrement more resident counters before it finds a 0. That wears down the protection of hot main blocks faster per promotion, and it weakens the large-bucket filtering that the parent gained.

## Takeaway

Carrying probation hits into main as a GCLOCK counter is redundant with where the block lands. Placing the promoted block behind the hand already gives it a full rotation of grace, so the extra counter just adds counter mass to main and speeds up the decay of established blocks. The hypothesis's refutation clause applies: main-region retention of newly promoted blocks is not the bottleneck on hm_0. Resetting to 0 on promotion was the better choice, and the open knob is probation depth (`small_len`), not the promotion counter. Per-op-time was not reported in the fitness vector, so the predicted rise in sweep cost from longer decrement walks is unmeasured.

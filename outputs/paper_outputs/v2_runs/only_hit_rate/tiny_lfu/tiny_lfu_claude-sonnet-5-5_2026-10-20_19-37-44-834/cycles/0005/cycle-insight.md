# Cycle 0005 Insight

## Cohort outcome

Three changes were tried: a further-loosened admission rule gated on a window hit (v001), a gentler 25% cut in sketch row width (v002), and reusing precomputed sketch indices across estimate and increment (v003). v001 was named Winner, but its own insight reports a marginal hit-rate loss. As in cycle 2, it looks like the least-bad variant, not a real gain.

## Patterns

- Same recurring split: changes that degrade frequency information (looser admission, narrower rows) lost a little hit-rate. The behavior-preserving change (index reuse) matched the parent exactly.
- Admission loosening has hit its limit. The tie-admit duel already captured the benefit, and going past ties swapped in colder entries. In-window recency adds nothing beyond the sketch estimate here.
- Even a gentle width cut cost hit-rate, so the sketch is already near the width where collisions matter. Combined with cycle 4, width reduction of any size looks unproductive on this workload.
- For the fifth cycle running, the fitness vectors report hit-rate only. The cost goals of v002 and v003 (metadata-size, per-op-time) remain unmeasured, so the Winner choice rests on hit-rate among near-ties.
- Variant authors again could not always read overlay source, so explanations rest on hypothesis text and fitness vectors.

## Open questions

- Does v003's index reuse actually reduce per-op-time, and does the cached index array cost metadata?
- Does v002's size saving justify its small hit-rate loss? That cannot be judged without a size measurement.
- Is workload-a saturated, so that no admission or sketch tweak can move hit-rate on it?

## Continuation

Do not generate more hypotheses until the evaluation is fixed: add metadata-size and per-op-time to the fitness output, plus a second trace (shifting popularity or a different cache size). Drop further admission loosening past the tie-admit rule and drop sketch width cuts. Keep v003's index reuse as a candidate cost win and verify it on speed first.

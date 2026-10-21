# Cycle 0008 Insight

## Cohort outcome

Three changes were tried: single-counting with a retuned (lower) tie-admit threshold (v001), very infrequent sketch halving on the wide sketch (v002), and an intermediate sketch width of 8 on the baseline (v003). v001 was the Winner by matching its parent with a marginal, noise-level gain. v002 lost slightly, and v003 gained slightly but was not measured on size.

## Patterns

- Count semantics and the admit threshold are coupled, as cycle 7 suspected. Retuning them together restored parity, and parity is all it did. Admission-rule tuning has now been flat across several cycles.
- Even a very gentle aging period cost a little hit-rate. A never-decaying sketch is not the limiter on workload-a, so aging joins the dropped levers.
- The intermediate width gained slightly over the baseline, which fits the earlier picture of width helping with sharply diminishing returns. Its hypothesis text also described the change as a reduction when the factor goes up, so the direction and the size claim were muddled and cannot be trusted.
- Every dimension that could separate these variants on cost (metadata-size, per-op-time) is still unmeasured, for the eighth cycle running. The Winner is again a near-tie chosen on hit-rate alone.
- Variant authors again could not read overlay source, so explanations rest on hypothesis text and fitness vectors.

## Open questions

- Does single-count with a lower threshold simplify the code or speed up ops enough to count as a cost win?
- Does width 8 keep most of the width-16 gain at a lower sketch size? That cannot be judged without a size measurement.
- Is workload-a saturated for hit-rate, given eight cycles of noise-level movement?

## Continuation

Fix the evaluation before generating more hypotheses: add metadata-size and per-op-time to the fitness output, plus a second trace (shifting popularity or a different cache size). Then judge the sketch width/size trade-off using v003 and the earlier wide variant. Treat single-count with a lower threshold as a clean base, but drop further admission-rule tuning, sketch aging and halving, and segment-internal tweaks. State explicitly in any width hypothesis which direction the factor scales the sketch.

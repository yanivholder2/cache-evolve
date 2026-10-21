# Cycle 0010 Insight

## Cohort outcome

Three variants were tried: counting each miss once in the sketch with a zero admission margin (from the cycle 0009 winner), rotating the rejected victim to the probation MRU end (same parent), and a from-baseline rebuild as SLRU with a sketch-gated promotion and admission. Only the victim-rotation variant (v002) won, with a marginal hit-rate gain over its parent.

## Patterns

- Hit-rate was again the only measured dimension, and every difference is marginal on one trace at one cache size. The loop is still selecting on noise-level edges.
- Single counting with margin 0 landed almost exactly where double counting with a wider margin did. Count scale and margin offset each other, so only their ratio matters. Re-scaling counts is exhausted.
- Victim rotation was nearly neutral. Changing which entry is challenged barely moves outcomes, so the "stuck probation head" problem is not a real bottleneck. This matches the earlier finding that victim selection is plateaued.
- The from-baseline SLRU rebuild reproduced the known structural benefit of frequency-gated promotion and admission over plain LRU. It added nothing beyond the incumbent, and it did not win.
- Metadata-size and per-op-time have now been missing for ten cycles. The cost of the extra state (last-miss tracking, segment tags, sketch hashing) is unverified.

## Open questions

- Does the rotation gain survive another trace or cache size, or is it noise?
- Do the extra get/put coupling and rotation bookkeeping cost more per-op time than they return?
- Is the missing cost measurement a harness fault?

## Continuation

Stop tuning sketch count scale, admission margin, aging window, and victim selection or rotation; all are plateaued. Treat v002's rotation as an optional minor tweak. Fix metadata-size and per-op-time measurement and add a second trace or cache size before generating more hypotheses. Then change the signal being ranked, for example size or cost awareness, a different eviction policy family, or segment-structure changes tested on scan-heavy and long-reuse traces.

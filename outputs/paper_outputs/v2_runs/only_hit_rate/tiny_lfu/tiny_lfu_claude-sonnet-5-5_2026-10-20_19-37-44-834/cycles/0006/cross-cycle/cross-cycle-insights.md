## Recurring wins

- Changes that preserve frequency information and loosen admission slightly are the only ones that ever helped (cycle 3 tie-admit duel). Gains were tiny.
- Behavior-preserving cost changes (single-hash row derivation, index reuse, slab links) match the parent on hit-rate. They are safe candidates, but their size and speed benefit has never been measured.
- Windowed main-region structure gave a marginal gain early (cycle 1) and has been the stable base since.

## Recurring failures

- Anything that degrades frequency estimates loses hit-rate. This covers narrower rows (any size, 2x or gentle), 4-bit counters without aging, correlated or blocked row indices, and doorkeepers. The sketch is near the width where collisions matter.
- Decay/aging, window-size tuning (static or adaptive), and admission loosening past tie-admit gave nothing or lost on workload-a. The trace looks stable and frequency-driven.
- Segment-internal tuning (protected share, demotion victim) has been flat: ties with the parent.
- "Winner" labels are often near-ties or least-bad losers (cycles 2, 4, 5, 6), so they are weak evidence.
- Evaluation gap: for six cycles fitness has reported hit-rate only, with no metadata-size or per-op-time. Cost-oriented ideas cannot be judged, and only their downside shows. There is also only one trace and cache size.

## Open questions

- Do single-hash, index reuse and slab links reduce per-op-time or metadata? They are free wins if they do.
- Is workload-a saturated for hit-rate? Six cycles of tweaks produced only noise-level movement.
- Does aging, adaptive windowing or a doorkeeper pay off on a trace with shifting popularity or a different cache size?
- Can a compact or blocked sketch with decorrelated rows and aging keep hit-rate while gaining size or locality?
- Is the small tie-admit gain real, or noise?

## Per-policy notes

- TinyLFU/W-TinyLFU family (the only policy so far): the base is a window plus segmented main with a count-min sketch and a tie-admit duel.
- Sketch: u8 counters with conservative update cost no accuracy. 4-bit counters, narrower rows and blocked rows lose. Hash sharing is safe on hit-rate.
- Admission: tie-admit is the sweet spot. Looser rules swap in colder entries.
- Window and segments: window size, protected share and demotion choice are all near-optimal or irrelevant on workload-a.
- Next step before any new hit-rate hypotheses: add size and speed metrics and a second trace, then verify the cost-win candidates.

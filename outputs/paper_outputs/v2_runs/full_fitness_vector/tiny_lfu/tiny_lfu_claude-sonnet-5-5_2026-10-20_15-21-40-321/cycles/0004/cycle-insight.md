# Cycle 0004 Insight

## Cohort outcome

Three changes were tried on the compact-layout parent: a cheap map hasher with an MRU short-circuit on hit (v001), a one-bit doorkeeper in front of the sketch (v002), and a 4x wider sketch (v003). All three were kept as Winners. Only v001 was a real improvement: it cut per-op time and struct size with hit-rate unchanged. v002 and v003 moved hit-rate little or not at all.

## Patterns

- The map hasher, not link layout or the sketch, was the largest remaining latency cost. Swapping it recovered the per-op time lost to cycle 0003's compact layout and kept that layout's size gain.
- Both sketch-accuracy changes failed to yield a meaningful hit-rate signal. The doorkeeper slightly lowered hit-rate and widened the latency tail. Widening the sketch gave only a marginal gain at a large hidden heap cost. Sketch accuracy now looks saturated on workload-a.
- Adding state or hot-path work (a doorkeeper bitset) again raised tail latency and grew the struct. This repeats the pattern from cycles 0001 and 0002.
- v003's per-op improvement cannot be explained by its code change and is most likely noise. This is the second cycle where small latency deltas looked unreliable.
- Sketch heap changes stay invisible in the headline metadata number.
- Two insights were written without a source diff, so some attributions are inferred.

## Open questions

- Does v001's gain hold up under repeat measurement, and which part accounts for it: the hasher or the MRU skip?
- Does v001 combine with the compact nibble sketch and cycle 0002's hashing speedup without losing hit-rate?
- With sketch accuracy, victim choice, tie rules, aging and windows all exhausted, is any hit-rate lever left on workload-a?

## Continuation

Use v001 as the parent and keep stacking per-op cost and footprint reductions on it (the compact sketch and cheap hashing from earlier cycles). Drop the doorkeeper and wider-sketch directions, since sketch accuracy is saturated and the wider sketch only trades heap for a marginal gain. Hit-rate ideas need a different mechanism entirely. Repeat latency measurements before crediting any small delta.

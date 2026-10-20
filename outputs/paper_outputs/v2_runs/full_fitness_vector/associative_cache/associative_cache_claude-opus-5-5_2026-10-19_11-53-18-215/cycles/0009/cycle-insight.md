# Cycle 0009 Insight

## Cohort outcome

Each variant tested one axis:
- v001 stacked the 15/16 cap on cycle 0008 v003's LRU cold entry, with a probation floor of 2.
- v002 added a per-bucket count-min frequency gate for admission.
- v003 swapped the miss-path whole-bucket shifts for an in-place probation rotate, plus a branch-free ghost scan and an MRU early return in `hit()`.

Only v001 moved hit rate up, by a small amount, and it paid for it in get and put tail latency. v003 kept hit rate the same but was slightly slower. v002 lost hit rate and was slower. v002 and v003 are Winners by default, not on merit.

## Patterns

- The cap and LRU cold entry stack, but only just. The floor of 2 never took effect, because 0008 v003 already kept 2+ probation slots at 16–31. So v001 is really "15/16 on buckets of 32+ slots". The cap axis is close to saturated: making probation narrower now mostly buys longer rotates and more ghost-hit re-inserts on the tail.
- An admission signal is only worth something if it can tell keys apart. The undersized, slowly aged sketch saturated, so nearly every key counted as "seen". Admission quietly fell back to MRU entry, which undid v003's one-hit-wonder protection, and the put path got heavier for nothing.
- The miss-path memmove has now been measured, and it is not a cost. Shifting 16–32 sixteen-byte items on the minority of puts that are full ghost hits is cheap. Removing the shift saved nothing visible. The fixed extras that came with it (a ghost scan with no early exit, an MRU branch that rarely fires under 15/16) made both medians worse.
- Across the cohort, every latency regression came from extra per-op work: branches, counter updates, a larger bucket footprint. None came from bytes moved. Get and put now cost instructions per op, not data movement.

## Open questions

- Would a sketch sized correctly help? That needs enough counters for the window size divided by the threshold, index bits taken from more of the tag, and a saturation check done before measuring.
- Would a ghost-hit count, or a 2-bit ghost entry, give a better admission signal than a sketch, at lower cost?
- Is v001's small hit-rate gain worth its tail cost, compared with 0008 v003's 7/8 cap at 32+?

## Continuation

Use cycle 0009 v001 as the hit-rate base, but treat the cap as finished: change neither the cap nor the probation floor any further. Drop miss-path memmove work, branch-free ghost scans and MRU short-circuits, because data movement on the miss path is not the bottleneck. For hit rate, retry admission only if the signal is checked to discriminate before it is measured (a correctly sized and aged sketch, or ghost-hit counts). For latency, cut fixed per-op instructions on the get-hit path rather than bytes moved.

# Cycle 0002 Insight

## Cohort outcome

This cohort cashed in cycle-0001's two safe directions and answered its
open question. All three variants entered the Winners: two pushed `Item`
footprint to its floor (v001 stacks v003's slim `Item` onto v001's MRU
promotion; v002 deletes the last metadata field outright), and one
(v003) widened the promotion gate from "single hottest" to a proportional
top-quarter window, lifting hit-rate above the v001 incumbent without
re-introducing v002-cycle-0001's churn.

## Patterns

- Placement and footprint proved genuinely orthogonal: porting the slim
  `Item` onto the MRU-promotion winner added both wins (smaller entries,
  faster ops) while leaving hit-rate untouched, confirming cycle-0001's
  bet that the two axes don't trade against each other.
- Metadata slimming bottomed out this cycle. Once `timestamp` was gone,
  the lone frequency counter was the only thing padding `Item<u64,u64>`
  past its bare key+value core; deleting it collapsed the alignment
  padding for free because the active positional `ClassicLRU` never read
  the field. There is little metadata left to remove.
- The middle-ground promotion gate worked, but the gain was small — the
  bulk of protectable warm mass on this trace already sits near the MRU
  head, so a quarter-window captures most of what the tight gate missed.
  Widening also shortened target buckets, so latency improved alongside
  hit-rate rather than trading against it.

## Open questions

- Does combining the slim `Item` (H01/H02 footprint floor) with the
  top-quarter promotion gate (v003) compound cleanly, or does the
  shorter-bucket effect of the wider gate change the placement calculus?
- The quarter-window gain was small — is there a better-shaped gate
  (different fraction, or recency-weighted rather than positional) that
  reaches the warm keys the window still misses, or is the trace's
  protectable mass simply near-exhausted?
- With `Item` now at its byte floor, where does the next size win come
  from — bucket/segment structure rather than per-entry layout?

## Continuation

Carry forward the merged best: slim `Item` at its 16-byte floor plus the
top-quarter promotion gate, as the new baseline to beat. Treat further
per-entry metadata trimming as exhausted — pivot the size axis to
container/segment structure instead. On hit-rate, the positional window
is near diminishing returns; next cycle should test gate *shape* (varying
the fraction or trying a non-positional weighting) rather than just gate
*width*, and probe whether placement and the wider gate still stack when
applied together.

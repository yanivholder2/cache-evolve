# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

This founding cohort probed three independent axes against baseline: where
promoted items land (v001), how aggressively warm keys earn promotion (v002),
and how fat each `Item` is on the metadata axis (v003). All three entered the
Winners, but for different reasons — the two unambiguous wins (v001, v003) paid
off on latency and/or size with no regression, while the aggressive-promotion
variant (v002) was a controlled loss on hit-rate that bought only an incidental
latency gain.

## Patterns

- Per-op-time was the easy, repeatable win this cycle: every variant trimmed
  latency, and in each case the cause was removing hidden hot-path cost — an
  O(n) front-shift (v001), a candidate-scan branch (v002), or an
  `Instant::now()` syscall (v003) — rather than algorithmic cleverness.
- Hit-rate behaved as a balance, not a free lunch. Steering *where* a hot key
  lands (v001) lifted retention, but simply promoting *more* keys (v002) flooded
  the probing tier and evicted otherwise-resident keys. The hottest-only gate
  was load-bearing, not dead weight.
- The active eval policy is `ClassicLRU`, which is purely positional. v003
  exposed that the baseline `Item` carried `timestamp` and a word-sized LFU
  counter that the exercised policy never reads — pure overhead on size and
  latency.

## Open questions

- v001 (MRU promotion) and v002 (ungated promotion) move the same lever in
  tension — does a *gated* promotion that lands at MRU (v001's position with
  v002's filter retained) compound the hit-rate gain, or do the effects cancel?
- Is there a middle promotion gate between "single hottest" and "every
  re-access" that widens the protected set without churning the probing tier?
- Since `ClassicLRU` ignores recency/frequency metadata, how much more `Item`
  footprint is dead weight under the current eval config?

## Continuation

Pursue v001's MRU-promotion direction and treat v003's metadata trimming as a
near-free baseline to build on; both are safe foundations. Drop the
unconditional-promotion idea (v002) as-is — the hottest-only gate matters — but
keep its insight that promotion volume must be filtered. Next cycle should test
combining MRU placement with a tunable promotion gate, and explore further
`Item` slimming given the positional-only eval policy.


### Cycle 0002

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


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>

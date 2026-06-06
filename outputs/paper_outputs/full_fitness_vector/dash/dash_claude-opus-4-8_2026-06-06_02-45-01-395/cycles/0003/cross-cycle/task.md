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


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants forked from the cycle-0002 slim+MRU base and probed the
promotion-gate / hot-path axis: a wider positional window (v001), a
frequency-driven gate spending the free `lfu_counter` (v002), and a pure
branch-reorder latency play (v003). All three entered the Winners, but this
was a cohort of near-null results — every change landed within or below the
trace/measurement noise floor, with no clean compounding win among them.

## Patterns

- The promotion-gate axis has saturated on this trace. Widening the gate
  (v001) and switching it from positional to frequency (v002) each nudged
  hit-rate up only marginally, because MRU placement (`InsertPosition::Back`)
  already captures the warm mass either lever would protect — gate shape and
  MRU placement are substitutes, not additive. This directly answers cycle
  0002's open questions 1 and 2: protectable warmth is near-exhausted.
- Touching the hot `get` path to chase hit-rate carries real latency cost
  with little payoff: both the wider gate's promotion cascade (v001) and the
  per-hit counter read-modify-write (v002) raised p50 for a hit-rate gain
  indistinguishable from noise.
- Micro-latency reorders are below this harness's resolution. v003's branch
  reorder strictly removes a scan per hit yet moved latency the wrong way —
  and even `put`, which the change can't touch, regressed — confirming the
  saving is swamped by variance.

## Open questions

- With both placement and gate shape exhausted on hit-rate, is there any
  remaining lever on this trace that isn't noise-limited, or is hit-rate
  fully tapped?
- Since the harness can't resolve single-scan-per-hit savings, what
  granularity of structural change *is* large enough to measure?

## Continuation

Stop tuning the promotion gate and hot-path probe order — that axis is
saturated and its effects sit below the noise floor; further gate-shape or
reorder variants are not worth a slot. Pivot decisively to the
container/segment structure for the next real win (the size axis flagged in
cycle 0002), and size any future latency hypothesis to clear the measurement
noise floor rather than chasing per-hit micro-savings.


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

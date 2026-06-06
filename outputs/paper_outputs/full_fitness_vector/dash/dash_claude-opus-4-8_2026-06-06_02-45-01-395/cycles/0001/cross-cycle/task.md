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

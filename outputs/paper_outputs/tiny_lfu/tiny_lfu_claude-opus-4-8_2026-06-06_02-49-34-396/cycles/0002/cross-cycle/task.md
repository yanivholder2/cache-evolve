# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants were independent, single-lever tweaks to the CountMinSketch
off a shared baseline: frequency aging (v001), single-hash double-hashing (v002),
and narrowing counters from `u32` to `u8` (v003). The two efficiency-focused
sketch reshapings — cheaper hashing and narrower counters — won; the aging variant
lost.

## Patterns

- Winners both attacked sketch *cost* (per-op-time and metadata-size) while
  leaving residency logic and the admission test untouched, and both paid only a
  marginal hit-rate cost — confirming the sketch carries slack that can be traded
  for speed and memory cheaply.
- The loser touched *admission semantics* by decaying the frequency signal; the
  trace appears to have stable, low true frequencies, so aging erased useful mass
  and the byte-width counters never saturated. Signal-preserving changes beat
  signal-altering ones this cycle.
- Tail latency was the recurring soft spot: both an inline full-table sweep
  (v001) and the byte-narrow access pattern (v003) introduced p99 variance even
  when p50 improved.

## Open questions

- Is frequency aging genuinely a dead end here, or just mistuned? The decay
  window was pinned far too tight and the sweep was unamortized.
- How far can counter narrowing go — does 4-bit packing keep the memory win
  without the p99 regression seen at 8-bit?
- Does double-hashing's correlated-rows hit-rate dip compound with other
  sketch changes, or stay marginal?

## Continuation

Pursue the cheap sketch-efficiency levers further: combine the v002 hashing win
with v003 counter narrowing, and push toward 4-bit packing while watching p99.
Before abandoning aging, retry it with a much wider decay interval and an
amortized (non-inline) sweep. Keep favoring changes that preserve the frequency
signal and residency logic over those that alter admission.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Building on Cycle 0001's winning single-hash sketch, this cohort tried three
distinct levers — narrowing counters to `u8` (v001), swapping SipHash for a
multiply-xorshift hasher (v002), and replacing strict-LRU-head eviction with a
sampled-coldest victim walk (v003). All three landed in the Winners, each
buying a small gain on its target axis.

## Patterns

- Every variant traded a tail-latency cost for its primary win: narrowing and
  sampled eviction both regressed p99 (`as u32` mapping, per-eviction sample
  walk) while the cheap hasher cost a marginal hit-rate dip. The p99 soft spot
  flagged in Cycle 0001 persisted across unrelated changes.
- The two efficiency levers (narrowing, fast hashing) again preserved admission
  semantics and paid essentially zero or marginal hit-rate, confirming the
  sketch's slack. The one residency-logic change (sampled eviction) did lift
  hit-rate but only marginally, and at the heaviest tail cost of the cohort.
- Gains are now incremental and axis-local — the easy single-lever wins off the
  baseline appear largely harvested; each remaining lever moves one axis a little
  while nudging another the wrong way.

## Open questions

- Do the three wins compose? Narrowing + fast hashing + sampled eviction were
  each tested against separate parents; their p99 costs may stack rather than
  cancel.
- Can the recurring p99 regressions be amortized away (e.g. precomputed estimates
  on the eviction path) rather than accepted as the price of each win?
- Is sampled eviction's small lift worth its tail cost at this cache depth, or
  does the sample width need tuning to pay off?

## Continuation

Stop probing isolated single levers off the baseline — that vein is thinning.
Next cycle should compose the proven efficiency winners (u8 counters + fast
hasher) into one variant and confirm the gains are additive, then attack the
shared p99 weakness directly. Treat sampled eviction as provisional: retune its
width or drop it unless its hit-rate lift can be bought without the eviction-path
tail cost.


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

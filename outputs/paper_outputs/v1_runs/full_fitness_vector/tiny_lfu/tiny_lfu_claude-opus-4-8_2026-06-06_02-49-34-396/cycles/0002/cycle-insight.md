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

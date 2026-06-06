# Task: cycle-insight-writer

Cycle: 0004
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0004

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0004/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0004/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0004/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0004/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0004/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0004/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

Acting on Cycle 0002's call to stop probing isolated levers, this cohort
composed proven wins and opened a previously untouched axis: u8 counters
folded onto the fast hasher (v001/H01), node links re-encoded as u32
sentinels instead of `Option<usize>` (v002/H02), and v003's sampled
eviction migrated onto the cheap double-hashing path (v003/H03). All three
landed in the Winners.

## Patterns

- Composition held: the efficiency levers proved additive, not
  competing. Stacking the fast hasher with narrowed counters cost no
  extra per-op time and no hit-rate, confirming the sketch slack and the
  hasher win combine cleanly rather than stacking their downsides.
- The big move was attacking the long-pinned metadata-size axis directly.
  It had been frozen only because every prior variant touched the sketch
  heap and left node layout alone; halving the per-entry node via u32
  sentinel links was a clean, regression-free win — the cheapest axis was
  the one nobody had looked at.
- The recurring p99 weakness flagged in Cycles 0001–0002 was finally
  addressed, not just accepted: moving sampled eviction onto the single
  cheap hash tamed the eviction-path tail (the sample walk now shares one
  fold instead of paying SipHash per candidate), turning a tail cost into
  a net positive.

## Open questions

- With node layout, counter width, and hasher all now optimized together,
  what axis carries the next pool of slack — is residency/admission logic
  the only remaining frontier?
- The correlated-rows hit-rate dip from double-hashing persists as the one
  standing cost; is it worth buying back row independence, or is it
  permanently subsumed by the speedup it enables?

## Continuation

The "compose the proven winners" thesis paid off and the easy
efficiency/layout slack is now largely harvested across all axes. Next
cycle should pivot from cost-cutting to hit-rate: probe residency and
admission logic (sample width tuning, victim selection, the admission
test itself), since the per-op-time and metadata-size axes are now tight
and further squeezing them will likely only trade against hit-rate.


## Write

Exactly one file: `cycle-insight.md` in the output dir.

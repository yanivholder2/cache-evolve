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

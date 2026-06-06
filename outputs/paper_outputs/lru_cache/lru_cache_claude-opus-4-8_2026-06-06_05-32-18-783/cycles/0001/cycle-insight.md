# Cycle 0001 Insight

## Cohort outcome

All three variants forked from baseline, each a single-axis lever:
segmented-LRU for hit-rate (v001), narrow `u32` sentinel links for
metadata-size (v002), and a fast inline integer hasher for per-op-time
(v003). The hasher swap won — the only change that improved a hot-path
dimension without paying a hard regression elsewhere.

## Patterns

- Per-op-time was the contested axis: both losing variants moved their
  target dimension but bought it with added per-op latency (extra
  promote/demote branching in v001, per-access index-width casts in v002).
- The Winner improved per-op-time directly and even picked up a free
  metadata side-win (dropping the default hasher's seed) — cheap,
  near-pure changes on the hot path beat changes that trade one axis
  against another.
- Representation/policy changes that widen the node or sprinkle
  conversions did not pay for themselves in locality.

## Open questions

- The SLRU hit-rate gain (v001) is real and targets the LIRS workload's
  weakness — can it be made cheaper so it stops trading against latency?
- v002 flagged that narrow links could be kept without per-access casts;
  is there a cast-free encoding that lands the metadata win?
- v003's median win came with a get tail-latency regression — is the
  hasher change strictly worth it, or does the tail need attention?

## Continuation

Build on v003 as the per-op-time base. Pursue the narrow-link
metadata win again but without per-access width casts, and probe whether
the SLRU hit-rate lever can be reintroduced without its latency cost.
Drop the naive `u32`-cast-everywhere approach as tried-and-regressed.

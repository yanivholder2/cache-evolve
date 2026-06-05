# Cycle 0006 Insight

## Cohort outcome

This cycle ran two converging tests on the promotion gate and opened one
brand-new seam on the stash. The promotion-side probes — a density-independent
per-key touch counter (v001) and loosening the len-2 hottest-only boundary
(v003) — came back flat and slightly negative respectively. The genuinely new
lever, a cuckoo-style stash relocation hop from a clean baseline (v002), was
the only clean positive move. The winner was v001, but its gain was
noise-level; v002 is the more meaningful result.

## Patterns

- The promotion gate is now closed from two independent directions. The touch
  counter replaced *rank* with a density-independent warmth signal and still
  produced nothing — proving the small-bucket ceiling is structural, not a
  rank artifact. The len-2 loosening over-admitted and edged down — proving the
  withheld second resident is genuinely cold, not a warm key held back. Both
  reads point the same way: no promotion-side signal, of any shape, beats the
  occupancy ceiling on this trace.
- The stash's load-bearing property remains *dwell time*, and it is the only
  axis still yielding gains. A single relocation hop into a salt-derived
  alternate bucket lets a few more cold keys catch their second reference
  before dropping — positive, no regression, capacity conserved. This is the
  freshest live lever in the whole search.
- The relocation gain is marginal for a structural reason, not a logic flaw:
  the alternate stash bucket is usually full too, so one hop rescues only a
  sliver of the dwell-time headroom the stash holds.

## Open questions

- The stash relocation is the first new positive seam in several cycles — does
  it deepen (multi-hop cascade, smarter overflow-victim selection, larger
  alternate set) or does it saturate as fast as every prior lever once the
  alternate buckets fill?
- With the promotion gate now exhausted by both rank and touch-count signals,
  is *every* hit-rate lever on this trace ultimately gated by the same hashing
  skew, or is the stash dwell-time axis structurally different?

## Continuation

Stop all promotion-gate work — rank, fraction, and now density-independent
touch-counting are all confirmed dead against the small-bucket ceiling, and
the len-2 boundary is doing real work that must not be loosened. The live
frontier is stash dwell-time: v002's relocation hop is the only fresh positive
result, so next cycle should deepen it (better victim selection, a second hop,
or a larger alternate set) rather than chase more promotion variants. The
recurring skew/occupancy bottleneck — now blocking levers across five cycles —
makes validating against a second, less-skewed trace overdue before investing
further.

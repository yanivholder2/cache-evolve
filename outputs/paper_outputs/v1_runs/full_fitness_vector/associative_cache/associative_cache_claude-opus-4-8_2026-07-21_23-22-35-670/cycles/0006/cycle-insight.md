# Cycle 0006 Insight

## Cohort outcome

The cohort worked to consolidate cycle 5's segmentation win rather than
chase new hit-rate: reclaim the metadata floor the SLRU incumbent had lost
by omission (v001), shed its per-access rotation latency (v002), and probe a
bounded protected cap to keep a live probationary zone (v003). The clean
win was v001 — the segmented bucket now delivers its ceiling-breaking
hit-rate at the 16-byte floor. v003 held the floor but its cap bet
regressed hit-rate; v002's lazy reorder recovered latency but leaked a hair
of hit-rate.

## Patterns

- The cycle-5 "40-byte" metadata cost of segmentation was never a cost of
  segmentation — it was pure overlay-hygiene debt. Trimming the dead
  `Item` fields the SLRU bucket never reads was behaviourally inert on
  hit-rate and reclaimed the full floor, and incidentally sped the
  byte-shifting put path. The recurring overlay hazard, once repaired,
  cost nothing.
- The per-hit protected rotation is confirmed pure overhead in the
  mixed-segment case — but it is not free to drop. It stops mattering only
  while a probationary item shields index 0; once a bucket saturates to
  all-protected, that same rotation was silently maintaining the intra-
  protected eviction order, so removing it trades eviction quality for the
  latency win. Latency reclamation here is entangled with the saturated-
  bucket edge case.
- The protected-cap knob is real but sharply mis-tuned at `max_size/2`.
  Capping that aggressively on short associative buckets demotes genuine
  reuse winners back to probationary and evicts them as scan-fodder,
  lowering hit-rate rather than lifting it. The "protection saturates to
  LRU" concern is genuine, but the corrective cap must sit close to
  `max_size` and demote only when protection is near-total.

## Open questions

- Can the protected rotation be shed for latency without the saturated-
  bucket hit-rate leak — e.g. lazy demotion that reorders only when the
  bucket is fully protected?
- Where does the optimal protected cap actually sit for these short
  buckets — near `max_size`, and does a near-total-only demotion trigger
  turn v003's regression into a gain?
- With the metadata floor now recovered on the segmented incumbent, is
  there any hit-rate headroom left above the SLRU ceiling, or is two-
  segment separation the structural limit for this trace?

## Continuation

Adopt v001 as the incumbent: it is the segmented (SLRU) bucket delivering
the ceiling-breaking hit-rate at the 16-byte floor, with the overlay-
hygiene debt finally cleared — build on it, not on cycle-5/v003. Do not
re-run the aggressive `max_size/2` cap; if pursuing a protected cap at all,
set it near `max_size` and demote only when protection is near-total, and
verify it against v003's regression. Treat v002's latency play as
conditional, not settled — the rotation may only be safely dropped when the
bucket is not fully protected, so any lazy-reorder retry must handle the
saturated case explicitly. Enforce full parent-layout inheritance on every
overlay so the reclaimed 16-byte `Item` is never dropped again.

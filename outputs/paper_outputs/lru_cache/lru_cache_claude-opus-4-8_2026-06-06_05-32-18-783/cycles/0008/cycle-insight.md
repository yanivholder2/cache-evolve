# Cycle 0008 Insight

## Cohort outcome

This cycle spent all three slots closing the live threads the prior
cycles left open: two probe-skipping get caches (v001 single last-key,
v002 four-slot hot-key side table) attacking the named HashMap-probe
hot-path cost, and one smarter ghost re-admission (v003) testing whether
the gated-ghost hit-rate gain reproduces. All three held the 24-byte node
but none advanced — the surviving Variants regressed their base. The
Cycle 0006 7/8 protected-cap champion stands undisplaced.

## Patterns

- The HashMap probe is now settled, from two angles, as not the get
  bottleneck. The baseline lookup is a zero-sized xor-multiply
  `FastHasher` plus an arena index probe — already cheaper than anything
  layered in front to skip it. The single-key guard chain (Option-compare
  plus a key clone) and the four-slot linear scan each added more work to
  the common path than they removed, and both more than offset their
  intended saving. The H03 last-key thread, finally measured here after
  dying on infra, is refuted on merit.
- The cross-cycle gated-ghost question is now answered negatively.
  Reuse-distance re-admission — treating one cross-eviction return as
  proof of hotness and admitting straight to protected — floods the
  protected segment and evicts genuinely hot keys, costing both hit-rate
  and latency. The Cycle 0004 hit-rate record does not reproduce on this
  short-reuse trace; it was trace/size-contingent.
- A recurring unscored cost surfaced: every variant kept `Node` at 24
  bytes by parking its state on `LruCache`, but that grew the cache struct
  (96→120/168 bytes) and the footprint alone perturbed put p50 even when
  put was byte-identical — locality damage the scored metadata-size never
  captures.

## Open questions

- With probe-skipping and ghost admission both now refuted, is the 7/8
  champion at this trace's hit-rate/latency frontier, or does the one
  untouched hot-path cost — link surgery and cache-line traversal — still
  hold room?
- Does growing the `LruCache` struct itself carry a real latency tax
  worth treating as a constraint, given how consistently footprint growth
  alone moved p50?

## Continuation

Keep Cycle 0006's 7/8 protected cap as the standing base; nothing this
cycle moved it. Three threads are now closed — stop revisiting them: any
structure that skips the HashMap probe (the probe is not the bottleneck),
ghost/reuse-distance re-admission (does not reproduce here), and admission
gating generally. The only named hot-path cost never directly attacked is
link surgery and cache-line traversal during promote/demote — if pursuing
latency, target that, and prefer changes that do not grow the `LruCache`
footprint. Absent a fresh structural lever, treat the champion as at or
near this trace's frontier.

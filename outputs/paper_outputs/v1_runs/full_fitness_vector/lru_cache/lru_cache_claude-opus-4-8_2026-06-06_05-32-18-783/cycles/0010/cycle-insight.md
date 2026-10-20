# Cycle 0010 Insight

## Cohort outcome

This cycle reached back into the SLRU lineage — not the flat-CLOCK
champion promoted last cycle — and worked two old axes: protected-cap
geometry from both directions (v001 up to 8/9, v003 down to 3/4) and a
source-level link-surgery store-elimination (v002). All three held the
24-byte node, all three regressed their bases, and none advanced the
frontier.

## Patterns

- The protected-cap geometry axis is now closed from both sides of 7/8.
  Stepping up to 8/9 retained marginally more promoted keys but bought no
  real latency change — the edit touches only a startup constant, so the
  symmetric p50/p99 rise reads as environment noise, not a cliff. Stepping
  down to 3/4 discarded proven-hot keys and pushed the protected segment
  into permanent overflow, firing demote-on-overflow on nearly every
  promotion. The two directions jointly confirm protected-side retention,
  not probation eviction, is the binding constraint, and that 7/8 already
  sits at the usable peak.
- Source-level store-elimination on the link-surgery path is refuted. The
  two trailing NIL self-writes in `unlink` were already dead-code-
  eliminated by codegen (every caller's `push_back` overwrites both link
  words), so removing them at the source bought nothing and perturbed
  instruction scheduling enough to slow the hot path. Link surgery needs
  an *algorithmic* reduction in detach/relink ops, not removal of writes
  the compiler had already dropped.
- The cohort spent all three slots refining the dethroned SLRU structure
  while the standing flat-CLOCK champion went untouched — and reconfirmed
  the prior conclusion that the SLRU geometry/surgery axes hold little
  remaining room.

## Open questions

- With SLRU geometry now bracketed from both sides and source-level store
  trimming refuted, does the SLRU lineage retain any lever at all, or is
  it fully mined?
- The flat-CLOCK champion (cycle 0009/v003) was not probed this cycle —
  its own tuning axes (hand placement, sweep bound, fresh-key admission)
  remain entirely open.

## Continuation

Stop editing the SLRU lineage: protected-cap geometry is closed both
directions around 7/8, and link-surgery latency cannot be won by removing
already-elided stores — only an algorithmic op-count reduction would. The
frontier mover is still cycle 0009/v003 (flat single-list CLOCK), and it
was left idle this cycle. Next cycle should return to it and tune the
clock itself (hand placement, sweep bound, admission of fresh keys)
rather than continuing to bracket the abandoned segmented champion.

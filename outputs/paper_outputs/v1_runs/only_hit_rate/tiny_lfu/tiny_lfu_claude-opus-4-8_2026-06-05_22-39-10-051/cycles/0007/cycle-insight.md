# Cycle 0007 Insight

## Cohort outcome

This cycle finally fired the two probes the prior digests had reserved: a
*corrected* pre-increment entry floor (v001), a direct-to-protected admission
placement (v002), and the long-requested efficiency pivot — a 4x-narrower
sketch leaning on conservative update (v003). v001 took the Winner slot but
only by holding exactly at parent; v002 regressed a hair and v003 regressed
measurably. No lever produced lift.

## Patterns

- The corrected entry floor now engages mechanically, settling cycle 0006's
  inert-gate ambiguity — but it gates nothing on this trace: the probationary
  keys that earn a warm-up hit have already been seen twice, so the floor
  admits exactly the keys the unconditional parent admitted. A live lever with
  no work to do is the cleanest confirmation yet that the hit-rate ceiling is
  structural, not a missed gate.
- Steering placement from the *admission* side (v002) is net negative: a single
  sketch estimate is a weaker signal than an observed in-cache hit, so bypassing
  probation admits occasional false-hot keys. The probationary "earn a hit"
  filter is doing real work and removing it costs more than it saves.
- The efficiency pivot has a hard floor: conservative update does *not* hold
  accuracy at 1/4 width. The extra sketch width was not waste — this workload
  genuinely needs more cells than capacity×1 to separate its distinct keys, so
  the cheapest-cache-for-free route via narrowing is closed.

## Open questions

- With admission, promotion entry/gating, eviction, demotion, *and* a working
  entry floor all now probed and inert, is there any hit-rate lever left
  untouched, or is the structural axis fully and finally mapped?
- v003 refutes 4x narrowing but not the existence of *some* width below the
  current setting that still holds accuracy — is there a shallower memory
  reduction (e.g. 2x) that trades cells for cost without losing hits?

## Continuation

The hit-rate axis is now exhaustively mapped and confirmed at its ceiling: even
a correctly-firing entry floor finds nothing to gate, and admission-side
placement actively hurts. Stop proposing structural hit-rate refinements — every
SLRU steering point is spent. The efficiency pivot is now live but bounded: 4x
narrowing is refuted, so do not retry aggressive width cuts; if cost is pursued,
probe a *shallow* reduction (2x) to find where accuracy actually breaks, or turn
to other cost/efficiency dimensions entirely rather than memory width.

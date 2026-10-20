# Cycle 0006 Insight

## Cohort outcome

The cohort tried a bounded direct-mapped frequency counter behind the admission gate (v001), placing newly admitted entries at the front of the probationary slice (v002), and shrinking the protected region from about 75% to 50% with demotion unchanged (v003). None of the three produced a hit-rate gain on workload-a. v001 and v002 both lowered it, and v003 was flat, yet v003 is recorded as the Winner.

## Patterns

- Every variant regressed or stayed flat on the single measured dimension. The Winner label again does not match the hit-rate evidence, as in Cycle 0004. The selection rule, or a dimension missing from the insights, needs to be checked before v003 is treated as a precedent.
- Shrinking the protected region (v003) changed nothing measurable. Protected capacity is therefore unlikely to be the cause of the protected-tier failures in Cycles 0004 and 0005, which points toward the demotion or second-hit signal.
- Bounding the admission gate's frequency metadata (v001) cost hit-rate. The insight attributes this to three simultaneous changes (slot aliasing, u8 saturation, periodic halving), so the cause is unresolved. The Cycle 0005 question about the shared counter's metadata cost is still open, and this result does not settle it.
- Front-of-probation placement (v002) hurt, while Cycle 0005's eviction-next insert reported a gain. The two may use different meanings of "next" or "front," so the gain from the one-shot-insert direction is not yet secure. Placement semantics need to be pinned down before the two results are compared.
- Per-op time and metadata size were still not measured in any variant, so the cost side remains unknown, as in Cycles 0001 to 0005.

## Open questions

- Why was v003 chosen as Winner when its insight reports no measurable change and its own prediction failed?
- Is the one-shot-insert gain from Cycle 0005 tied to inserting at the victim end of probation or at some other position, and does v002's front placement contradict it?
- Does aliasing or u8 saturation drive the v001 loss? Separating the three changes would show whether a cheaper frequency store is viable at all.
- Is the second-hit or demotion rule the remaining cause of the protected-tier failure?

## Continuation

Next cycle should reconfirm the Winner selection before using v003 as a precedent, and should not pursue protected-region capacity as a hit-rate lever. Pursue the demotion or second-hit signal instead, and state the insert position explicitly, testing victim-end placement against front placement. Drop the direct-mapped, saturating frequency store as a single change; retry metadata savings only with exact keys or with one change at a time, and include metadata size in the fitness vector.

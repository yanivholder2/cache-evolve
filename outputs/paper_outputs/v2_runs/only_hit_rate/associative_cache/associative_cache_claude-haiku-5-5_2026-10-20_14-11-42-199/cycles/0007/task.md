# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0007

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

The cohort tried a second-hit protected tier on top of probationary-first eviction (v001), a sampled probationary victim scan capped at 8 rotating slots (v002), and a single shared frequency counter gating eviction-time admission (v003). Only v003 showed a hit-rate gain on workload-a; v002 was flat to slightly negative and v001 was negative. The recorded Winner is v002, which does not match the hit-rate evidence in its own insight.

## Patterns

- The only positive hit-rate movement came from the shared-frequency admission gate (v003), which compares incoming and resident keys on one scale. This supports the Cycle 0002 suspicion that the earlier admission loss came from a scale mismatch rather than from admission itself.
- The Winner label and the evidence disagree. v002's insight reports a negligible hit-rate dip that misses its own guardrail, while v003 is the variant with a gain. Either the Winner selection used a dimension not shown in the insights, or the label was applied in error. This should be checked before the Winner is treated as a precedent.
- Bounding the victim scan (v002) left hit-rate essentially unchanged, consistent with Cycle 0003's finding that exact-minimum eviction buys little. Its per-op-time benefit remains unmeasured.
- Stacking a second reuse signal (v001) lowered hit-rate. The insight points at the protection cap and demote-on-promotion path rather than the promotion trigger, so the failure is plausibly in the mechanism, not the idea of a second tier.
- No variant measured per-op time or metadata size, and all three insights say the overlays could not be inspected in this run. Flat or negative hit-rate cannot yet be read as a null result about the mechanism, and the cost side is still unknown, as in Cycles 0001 to 0003.

## Open questions

- Which dimension made v002 the Winner, given that v003 carries the only hit-rate gain? Is the Winner rule using a metric missing from the insights?
- Is the v003 gain reproducible across more than workload-a, and does its shared counter add measurable metadata or per-op cost?
- Would a protected tier survive if the cap did not force demotion on each promotion?
- Are the v002 and v001 overlays active under the harness? Without that, their results are ambiguous.

## Continuation

Next cycle should build on the shared-frequency admission gate from v003, and add per-op time and metadata size to the fitness vector before accepting any change. Reconfirm the Winner selection for this cycle before treating v002 as a precedent. Drop the bounded-scan direction as a hit-rate lever, and retry the protected tier only with a changed cap or demotion rule rather than the same promotion path.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

The cohort tried a shared-frequency admission gate on top of probationary-first eviction (v001, the Winner), a removal of demote-on-promotion in the protected half-bucket (v002), and inserting misses at the eviction-next position instead of MRU (v003). The two variants that address one-shot inserts both reported a hit-rate gain on workload-a; the protected-tier variant ended slightly below its parent.

## Patterns

- Both positive results target the same problem: one-shot inserts displacing entries with proven reuse. The admission gate (v001) filters them at miss time, and the eviction-next insert position (v003) lets them leave first. Whether these gains overlap or add is not yet known.
- The Winner's gain is recorded only qualitatively. No magnitude is given, so v001 and v003 cannot be ranked against each other on the evidence in their insights.
- Removing the forced demotion (v002) did not recover the loss from the protected tier in Cycle 0004. The protected cap itself, or the second-hit signal, is the more likely cause, consistent with Cycle 0004's suspicion.
- Hit-rate is still the only measured dimension in all three insights. Metadata size and per-op time remain unknown for the shared counter (v001) and the miss-position flag (v003), even though v003's hypothesis claims no new per-entry metadata.
- v002's insight says its overlay could not be read in this run, so its result is not confirmed against the actual diff. The other two insights do not state this.

## Open questions

- Are the admission gate (v001) and the eviction-next insert position (v003) redundant, or do they stack? Testing each alone against the parent would settle this.
- Does the v003 gain hold beyond workload-a, given the Cycle 0004 concern about single-trace results?
- What does the shared counter's metadata and per-op cost amount to once measured?
- Is the protected-cap failure in v002 a cap-size problem or a second-hit-signal problem?

## Continuation

Next cycle should pursue one-shot-insert filtering, testing the admission gate (v001) and the eviction-next insert position (v003) separately or in combination, and confirm the gain on a second workload. Include metadata size and per-op time in the fitness vector before accepting either change. Drop the protected half-bucket with demote-skip as a hit-rate lever, and do not retry it without a changed cap or second-hit signal.


### Cycle 0006

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.

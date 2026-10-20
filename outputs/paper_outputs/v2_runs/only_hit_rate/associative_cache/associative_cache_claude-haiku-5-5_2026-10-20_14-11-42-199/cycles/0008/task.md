# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0008

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0008/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

The cohort tried inserting admitted keys at the probationary victim end (v001), a second-miss ghost list that admits a key only on its second miss (v002), and a non-strict shared-frequency admission predicate that admits on counter ties (v003, the recorded Winner). Only v002 reported a hit-rate gain on workload-a; v001 and v003 both lowered it.

## Patterns

- The one positive result is a recurrence gate. The ghost list (v002) refuses first-time inserts without a frequency counter, which is the exact-key, no-counter form that Cycle 0006 asked to be retried. It also extends the one-shot-insert direction from Cycles 0005 and 0006.
- The recorded Winner (v003) has a hit-rate regression, and the only gain (v002) is not the Winner. This is the same Winner/evidence mismatch flagged in Cycles 0004 and 0006, and it now recurs. The selection rule needs checking before any of these results is used as a precedent.
- Insert position is a live confound. Placing admitted keys at the victim end (v001) lost hit-rate, while Cycle 0005's eviction-next insert reported a gain. The likely reading is that a newly admitted key needs some residency before it becomes eviction-eligible, but the Cycle 0005 and 0007 placements have not been shown to mean the same thing.
- Tightening the frequency predicate to admit ties (v003) gave no benefit. Strict comparison looks like the better default under unbounded counters.
- Cost is still unmeasured everywhere. v002's ghost list adds per-key metadata and v003 and v001 touch the shared counter, yet none of the three reports metadata size or per-op time. The hypothesis-level cost claims remain unverified, as in Cycles 0001 to 0006.

## Open questions

- Why was v003 chosen as Winner when its insight reports a slight hit-rate regression?
- Does the v002 ghost-list gain hold on a second workload, and what is its metadata size at the same cache size?
- Is the Cycle 0005 eviction-next gain the same mechanism as v001's victim-end loss, or is the placement semantics different?
- Does the v003 tie result reflect the tie rule itself, or a shift in victim selection pressure? The admitted-on-tie count was not checked.

## Continuation

Next cycle should pursue recurrence-gated admission (v002), testing its ghost-list window and checking it on a second workload with metadata size and per-op time in the fitness vector. Drop victim-end placement for newly admitted keys as a hit-rate lever, and do not pursue tie-admission in the shared-frequency predicate. Reconfirm the Winner selection before using v003 as a precedent.


## Write

Exactly one file: `cycle-insight.md` in the output dir.

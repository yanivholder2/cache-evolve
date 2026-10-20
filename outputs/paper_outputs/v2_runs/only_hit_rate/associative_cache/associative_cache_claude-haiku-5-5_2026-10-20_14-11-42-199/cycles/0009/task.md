# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0009

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0009/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

The cohort tried victim-end placement for admitted keys (v001), pruning singleton history from the shared frequency map (v002, the recorded Winner), and quadrupling the second-miss ghost-list window (v003). All three report a hit-rate regression on workload-a, so no variant produced a gain, and the Winner's own insight describes a decrease.

## Patterns

- No variant improved the measured dimension. The Winner label (v002) again conflicts with its evidence, as in Cycles 0004, 0006, and 0007. The selection rule should be checked before v002 is used as a precedent.
- Victim-end placement for admitted keys (v001) lost hit-rate again, now in a second cycle after Cycle 0007's version. The Cycle 0005 eviction-next gain has not reproduced under this placement. Its insight suggests the earlier gain depended on back-of-probation placement, so the two results may not be comparable.
- Widening the ghost-list window (v003) regressed. Together with Cycle 0007's recurrence gate, this suggests the one-window setting is about right and a longer first-miss memory admits stale one-shot keys.
- Pruning singleton frequency history (v002) produced a small regression, not the metadata saving its hypothesis predicted. The change also has no hysteresis, so a full prune could run on many accesses. That is a plausible per-op cost, but it is not measured.
- Metadata size and per-op time are still unmeasured in every variant, as in Cycles 0001 to 0007. The fitness vector still lacks these dimensions.

## Open questions

- Why was v002 chosen as Winner when its insight reports a hit-rate decrease?
- Does the Cycle 0007 recurrence-gate gain (ghost list at one-capacity window) hold when the window is kept at one capacity but the rest of the setup is unchanged, or was that gain specific to its run?
- Does singleton pruning change the admission gate's behavior enough to explain the decrease, or is the loss from some other effect of the prune?
- How often does the prune's full pass run under the current map size, and what does that cost per access?

## Continuation

Next cycle should not pursue victim-end placement, ghost-window widening, or singleton pruning as hit-rate levers. Keep the one-capacity ghost window from Cycle 0007 as the base for recurrence gating, and retest its gain before building on it. Reconfirm the Winner selection before using v002 as a precedent, and add metadata size and per-op time to the fitness vector before accepting any memory-saving change.


## Write

Exactly one file: `cycle-insight.md` in the output dir.

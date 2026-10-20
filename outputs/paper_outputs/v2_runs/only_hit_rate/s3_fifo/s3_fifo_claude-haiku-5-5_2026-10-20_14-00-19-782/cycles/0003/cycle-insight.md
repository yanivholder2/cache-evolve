# Cycle 0003 Insight

## Cohort outcome

The cohort tried a resident-key lookup structure to skip queue scans on misses (v001), a higher small-queue promotion bar (v002), and a doubled ghost queue (v003). Both Winners (v001, v002) are hit-rate neutral against the parent, so neither is a demonstrated gain; v003 is the only variant that lost hit-rate.

## Patterns

- Hit-rate stayed flat for both Winners. v001 is intended as a cost-side change that should leave eviction order alone, and v002's promotion-bar change showed no effect on this trace.
- The promotion bar has now been probed in both directions across cycles: relaxing it to one hit lost hits in Cycle 0001, and raising it to three hits was neutral here. The axis looks saturated on this trace, not a source of gains.
- Widening the ghost window (v003) lost hit-rate. The variant's account is that extra ghost hits bypass the small queue and land directly in main, displacing resident entries. Ghost coverage alone does not appear to be the lever.
- Neither Winner reported metadata-size or per-op-time, so v001's cost claim is still unverified. This gap has now persisted across three cycles.
- v002 could not confirm from its overlay that the `freq > 2` edit landed, so its neutral result may not reflect the intended change. This echoes Cycle 0002's overlay problem.
- Evidence is still one trace (lirs workload-a) at one capacity (2048).

## Open questions

- Does the v001 resident-key structure reduce per-op-time, and at what metadata cost? It cannot be ranked until both are reported.
- Did the v002 edit actually land in the built code? Until that is confirmed, its flat result says nothing about the promotion bar.
- Does the ghost-window loss hold if ghost hits are admitted differently (for example, through the small queue rather than main)?

## Continuation

Keep v001's resident-key idea only as a cost candidate, and require a rerun that reports per-op-time and metadata-size before calling it a win. Drop the doubled ghost queue (v003) as a standalone change; any future ghost-size work must also change how ghost hits are admitted. Drop further promotion-bar tweaks on this trace. Before the next cycle, confirm that each overlay edit is present in the build so results can be attributed to the intended change.

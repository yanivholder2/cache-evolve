# Cycle 0003 Insight

## Cohort outcome

The cohort tried three changes to the admission and aging machinery: capping the protected segment without demotion (v001), switching the sketch's row indexing to the public per-row seeded scheme (v002), and halving the sketch's aging period (v003). The three Winners were each listed for a single trade-off rather than a clear net gain: v002 recovered hit rate at a cost in per-op time, and v001 and v003 mostly reported hit-rate or metadata costs.

## Patterns

- Hit rate and per-op time moved in opposite directions. v002 recovered admission fidelity by matching the public indexing, but gave back most of the speedup that the hoisted-hash design from the prior cycle had provided.
- Bounding the protected segment cost hit rate. The segmentation gain from Cycle 0001 seems to depend on letting the protected set turn over, so caps are the wrong lever.
- Shortening frequency aging hurt hit rate. Stale counts were not what limited the admission gate, which is consistent with the doorkeeper result in Cycle 0002.
- Several attributions are unverified. Source diffs could not be inspected, struct growth and some timing shifts do not match the single-constant changes that were intended, and every result still comes from one trace (lirs workload-a).

## Open questions

- Can the seeded per-row indexing from v002 be made cheaper while keeping the same counter indices, so the fidelity gain survives without the speed loss?
- Is the struct growth in v003 from unrecorded edits, and do the timing shifts in v001 and v003 survive repeat runs?
- Do any of these hit-rate effects hold on a second trace?

## Continuation

Pursue the index-equivalence work from v002: keep the seeded indexing semantics but cut its per-row cost, and verify it against the public path. Drop protected-segment capping (v001) and shorter sketch aging (v003) as directions, since both lost hit rate on this workload.

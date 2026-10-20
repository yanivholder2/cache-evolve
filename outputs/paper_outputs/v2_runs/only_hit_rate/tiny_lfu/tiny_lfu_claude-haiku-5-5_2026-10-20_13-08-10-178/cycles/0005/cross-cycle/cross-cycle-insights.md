## Recurring wins

- Reducing estimator error from colliding keys is the lever that has produced hit-rate gains: widening the sketch (Cycle 0004) and conservative update that raises only row-minimum counters (Cycle 0005) both tightened over-estimates.
- The probation/protected split inside the main region improved hit-rate when it was introduced (Cycle 0003), and the current protected/probation ratio and back-of-probation demotion have held as the better settings against alternatives (Cycle 0004).
- Layout-only sketch changes reliably preserve hit-rate, so they are safe to adopt, but they have not themselves produced gains.

## Recurring failures

- Changes that alter the frequency history the admission comparison relies on keep losing hit-rate: fixed-schedule halving (Cycle 0001) and global aging on a fixed access-count trigger (Cycle 0005). At this capacity the sketch needs its history intact.
- Shrinking estimator depth undid the collision suppression that the min-across-rows estimate depends on (Cycle 0005).
- Delaying admission behind a recency window in front of the LRU cost hit-rate and did not show the expected one-touch-burst benefit (Cycle 0002).
- Key-to-counter mapping rewrites produced a small hit-rate dip (Cycle 0001) and have not been shown to be within noise.
- Several Winners were advanced on claimed footprint or per-operation gains that the fitness vector does not measure, so they advanced by not regressing rather than by demonstrated improvement (Cycles 0001, 0002, 0003, 0004, 0005).
- Narrowing the main region's protected segment and pushing demoted protected entries to the front of probation both lost hit-rate (Cycle 0004).

## Open questions

- Do the width and conservative-update gains hold on other workloads and cache sizes, and do they overlap or stack?
- What do their metadata and per-operation costs look like once the fitness vector measures memory or timing?
- Is there an aging trigger, not tied to access count, that sheds stale frequency without weakening admission?
- Does the probation/protected gain hold beyond this trace, or is the ~4/5 protected ratio specific to it?
- Does the sketch's saturation point ever bind on very hot keys, where the admission comparison could go insensitive?

## Per-policy notes

- Frequency sketch (TinyLFU estimator): width increases and conservative update reduce collision over-counting and are the productive direction; depth reduction, fixed-schedule halving, and global access-count aging hurt. Layout-only changes are neutral on hit-rate.
- Admission: the comparison has been sensitive to any change that shifts when or how frequency history is seen; keep the sketch's history and the admission rule intact in the next cycle.
- Window / entry path: a recency window ahead of the LRU, with admission applied at window exit, lost hit-rate and should not be re-proposed at this capacity.
- Main region (SLRU-style): the probation/protected split is the one structural change that gained; the incumbent protected cap and back-of-probation demotion are the better settings; halved protected cap and push-to-front demotion are ruled out.
- Caveat for all policies: every result so far comes from a single trace at a single cache size, so each gain is provisional until tested elsewhere.

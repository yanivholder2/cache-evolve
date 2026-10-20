## Recurring wins

- Reducing estimator error is the most reliable lever. Widening the sketch (C4) and conservative update, which raises only row-minimum counters (C5), both gained hit-rate by tightening over-counting from collisions. Refinements on top of each other have diminishing returns, so they likely act on the same error.
- Main-region segment sizing matters. The probation/protected split (C3) and a larger protected share (C6) were both hit-rate winners in the main region.

## Recurring failures

- Fixed-schedule aging or halving of sketch counters loses hit-rate (C1, C5). Discarding frequency history weakens the admission comparison at this capacity.
- Changing when or whether admission sees a key loses hit-rate. A recency window ahead of the LRU (C2) and gating first-touch keys out of the sketch with a doorkeeper (C6) both lost. The withheld increments are not recovered by estimate-time correction.
- Reducing sketch depth (C5) loses hit-rate. The min-across-rows estimate needs enough independent rows to suppress collisions.
- Layout-only sketch changes (C1, C2, C3) hold hit-rate by construction, not by measured gain. Their claimed footprint and per-op benefits sit on dimensions the fitness vector does not measure, so they advance by not regressing.
- Main-region variants that shrink the protected cap or push demoted entries to the front of the list lost hit-rate (C4). The incumbent ratio and back-of-probation placement are the better settings on this trace.

## Open questions

- Do the width, conservative-update, and protected-share gains hold on other workloads and cache sizes? Every result so far is one trace at one size.
- What do the estimator and layout gains cost in memory and per-operation time? A timing or metadata dimension would let the Winners be confirmed rather than assumed.
- Do the width and conservative-update gains overlap, or do they stack?
- Is there an aging trigger, not tied to access count, that sheds stale frequency without weakening admission?
- Is there a doorkeeper-style correction that matches the withheld increments closely enough to be worth retrying?

## Per-policy notes

- TinyLFU admission and frequency sketch: wins come from estimator error reduction (width, conservative update). Losses come from aging, depth reduction, and admission-timing or gating changes. Keep the sketch's history intact and treat admission as fixed.
- Main-region segmentation (SLRU-style probation/protected within the main region): the protected-share and split changes are the gainful levers. Protected-cap halving and push-to-front demotion are not.
- Window or LRU front-end (window-TinyLFU style): a recency window ahead of the LRU lost hit-rate at this capacity. Do not re-propose it in that form.
- Layout and allocation (applies across policies): hit-rate neutral by construction. Keep layout changes out of stacking until a metadata or timing signal can confirm them.

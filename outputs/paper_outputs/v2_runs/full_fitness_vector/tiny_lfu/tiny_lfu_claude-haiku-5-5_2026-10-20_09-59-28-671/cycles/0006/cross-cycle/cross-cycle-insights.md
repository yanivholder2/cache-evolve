# Cross-cycle insights

## Recurring wins

- Constant-factor work that leaves eviction order and counter indices alone is the most reliable source of gains. Hoisting or reusing hashes, inlining accessors, and splitting shared put-side paths each moved per-op time without moving hit rate.
- Keeping sketch seeds in a module-level constant table, not per instance, removes the struct growth that per-instance caching caused while keeping the speedup.
- Packing protected-segment state into existing node fields cut metadata without changing hit rate.
- Restoring public per-row seeded indexing corrects admission fidelity. The cost is per-row work on both get and put, so it is a correctness win that gives back speed.

## Recurring failures

- Recency rotation of rejected admission victims lost on both hit rate and per-op time. The frequency gate already does the protective work.
- Changes to admission or aging that do not target a real limiting factor did not raise hit rate: a one-hit-wonder doorkeeper barely moved it, shorter frequency aging lowered it, and a small recency window in front of the gate did not move it while adding list work.
- Capping the protected segment without demotion lowered hit rate, so bounding segment size is the wrong lever.
- Index layouts that change which keys collide in the sketch move hit rate, even when they speed up the per-op path.
- Each attempt to speed up the sketch's per-row indexing has traded away either index fidelity or speed.

## Open questions

- Do the const-table seeds and the hoisted-hash helpers produce exactly the same counter indices as the per-access public derivation for every key, or only matching aggregate hit rate?
- Can the public per-row indices be produced with less per-row work, so fidelity does not cost the speed gain?
- Can a cache-line block-grouped counter layout keep its per-op gain while preserving the original collision pattern?
- Does the protected-capacity branch ever fire on the workload? If not, protected-tail demotion has not really been tested.
- Do the timing shifts (put-side improvements and regressions, get-side gains) survive repeat runs, or are they noise from single runs?
- Are the metadata figures complete? Doorkeeper storage and window bookkeeping do not appear in the reported breakdown, and struct growth has gone unexplained more than once.
- Do any of the hit-rate effects, especially the segmentation gain, hold on a second trace? Every result so far comes from one trace.

## Per-policy notes

- **TinyLFU-style admission with a frequency sketch and segmented resident list (all cycles):** the main levers are the sketch (indexing, seeds, layout, aging), the admission gate (doorkeeper, window), and the protected/probation split.
  - Segmentation is the only source of hit-rate movement seen so far, and it is small. Its benefit appears to depend on the protected set turning over, so caps hurt.
  - The sketch's indexing is the main place where hit rate and speed trade off directly.
  - Admission-side additions (doorkeeper, window) and aging changes have not helped on this workload.
- **Recency handling of rejected victims:** dropped after a single cycle of clear losses. No other recency-ordering change has been tested in this policy family.
- **Evidence scope:** every cycle uses the same single trace (workload-a), so no finding here is yet a general policy property.

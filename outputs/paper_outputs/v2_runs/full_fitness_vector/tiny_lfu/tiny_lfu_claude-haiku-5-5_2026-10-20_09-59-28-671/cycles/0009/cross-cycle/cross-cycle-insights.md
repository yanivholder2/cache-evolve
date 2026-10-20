# Cross-cycle insights

## Recurring wins

- Per-op time is the most reliable lever. Constant-factor changes that leave admission and eviction order alone (shared hashing, hoisted seeds, shared unlink/promotion paths) recur as speed gains with hit rate flat (Cycles 0001, 0002, 0004, 0005).
- Seed and bookkeeping storage belongs in shared constant tables, not per-instance fields. Moving seeds to a module-level table removed the struct growth that per-instance caching had caused (Cycles 0004, 0005, 0006).
- Metadata can shrink independently of policy. Packing protected state into existing fields cut size without moving hit rate (Cycle 0002).
- Collision spread is the one lever that has moved hit rate upward recently. A conservative sketch update and a wider-but-shallower sketch both gave small gains, and a narrower-counter, wider-sketch layout did too (Cycles 0008, 0009). Each stayed within the baseline memory budget.

## Recurring failures

- Changes to admission or aging have not raised hit rate on this workload. Recency rotation of rejected victims, a doorkeeper, a W-TinyLFU-style window, shorter sketch aging, and admission tie-breaking all left hit rate flat or lower (Cycles 0001, 0002, 0003, 0004, 0007). Stale counts and one-hit pollution do not appear to be the binding constraint.
- Capping or shrinking the protected segment costs hit rate. Bounding the protected set and halving its share both lost hit rate and per-op time, so capacity is the wrong lever (Cycles 0003, 0007). Protected-tail demotion alone produced no change (Cycle 0006).
- Layouts that change which keys collide trade hit rate for speed. Cache-line grouping gained per-op time only by changing collisions (Cycle 0006), and taking the column from the per-row seed lost the speed without restoring spread (Cycle 0007).
- Restoring public per-row indexing costs speed. It recovers admission fidelity but adds per-row work on get and put (Cycles 0003, 0005).
- Timing claims from single runs have not held. Put and get shifts appeared in changes that should not touch those paths, and several reported gains or regressions could not be tied to a mechanism (Cycles 0004, 0005, 0006, 0008, 0009).

## Open questions

- Generalization is untested. Every cycle used one trace (workload-a) and one cache size (2048). No gain has been checked on a second trace or size.
- Can collision-spread gains be kept without an extra pass over all rows? This applies to the conservative update and the row/column trade.
- Do narrow saturating 8-bit counters cost admission accuracy? The hit-rate gain was small and the saturation effect was not isolated.
- Do the seed-table indices match per-access derivation for every key, or only in aggregate hit rate?
- Do the put-side tail gain and median regression in the single-loop conservative update survive repeat runs?
- Is the doorkeeper's and the window's memory cost reported at all? Neither storage shows in the metadata breakdown.
- What caused the unexplained metadata movements: the struct growth in one variant, and the smaller sketch size in another?

## Per-policy notes

- TinyLFU (frequency-sketch admission gate): the sketch is where hit-rate movement has come from. Collision-reducing updates and layouts help slightly; admission-side bookkeeping (doorkeeper, tie-breaks, windows, aging) has not. Counter width and row/column shape are the open tuning axes.
- Segmented LRU (probation/protected lists): protected-segment capacity, not turnover, drives the trade-off. Shrinking the protected share or capping it lost hit rate. Pointer-work trims on the MRU path are safe speed wins.
- Trace coverage: all policy-level evidence is from one trace (workload-a) at one size, so no per-policy result is yet general.

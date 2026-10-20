## Recurring wins

- Eviction-neutral changes (lookup collapse, sentinel index, relink skip, index pre-sizing, hasher swap) have repeatedly left hit-rate exactly equal to their parent. This is a dependable non-regression, not a measured gain. Use these as safe bases for stacking.
- Bounded frequency-aware eviction (least-hit entry within a small tail window) is the only change that has moved hit-rate upward, and only slightly (cycle 0007). It is a single-cycle signal, not yet a recurring win.
- No measured gain has held across more than one cycle.

## Recurring failures

- Winner labels are often not backed by the measured dimension. Across cycles 0001–0006 several Winners had no reported gain on their target, and in cycles 0005 and 0006 the named Winner had a hit-rate drop. The selection criterion is not tracking the fitness signal.
- Per-op time and metadata size have never been recorded in any cycle (0001–0007). Every hypothesis motivated by time or size is therefore untested.
- Loosening strict recency lowered hit-rate on workload-a at size 2048: the segmented probation/protected list and second-chance referenced-bit eviction (cycle 0005) both dipped, and second-hit promotion (cycle 0006) also lost.
- Tail insertion of new keys lost hit-rate in both of its cycle 0006 forms, despite a gain in cycle 0005. It cannot be counted as reliable.
- Ghost-set admission filters have held hit-rate or produced no usable result (cycles 0002, 0003, 0007). No cycle has shown the filter engaged, so the idea is untested rather than refuted.
- Infrastructure rejections (two variants in cycle 0001, one in cycle 0004) are not strategy evidence. Those approaches remain unexamined.

## Open questions

- Does tail insertion help on its own against a fixed parent? Cycles 0005 (gain), 0006 (loss), and 0007 (null) cannot be reconciled from the insights.
- What should the Winner criterion require? Several Winners lacked a movement on their own target dimension.
- Does the frequency-aware tail-window gain hold on a second trace or cache size, and what does its per-entry counter cost in memory?
- Did the ghost filter and the tail-insertion overlay ever engage? Engagement counters are needed before either can be ruled in or out.
- The H03 label has meant different mechanisms across cycles (per-op check, pre-sized index, hasher swap, segmentation). Fix one definition before lineages cite it.
- Which direction did the cycle 0004 segmentation variant move hit-rate, and by how much? The insight does not say.
- Does the protected cap or demotion rule in segmentation carry headroom, or is segmentation dead as written?

## Per-policy notes

- Strict LRU (move-to-front recency): representation and lookup changes preserve it. Every change that loosened recency order (segmentation, second-chance, second-hit promotion) lowered hit-rate on workload-a at size 2048, which suggests strict recency is doing useful work on this trace.
- LRU with insertion-position changes (tail or LRU-end insertion, sparse MRU insert): mixed evidence across cycles 0005–0007 (gain, losses, null). Unresolved and not safe to build on until tested alone.
- Segmented probation/protected: dropped as written after a hit-rate dip. Its cap and demotion parameters have not been tested in isolation.
- Second-chance referenced-bit: lowered hit-rate in cycle 0005. No current evidence for it.
- Ghost-set admission: never shown to engage. Where reported, it was neutral or had no usable result. Untested.
- Frequency-aware bounded tail-window eviction: the only upward hit-rate movement, and slight. Memory and per-op cost are unmeasured. Lead candidate, pending a second trace or size.

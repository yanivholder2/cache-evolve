## Recurring wins

- **Admission filtering matters more than how recency is tracked.** S3-FIFO-style admission brought the big hit-rate gain in cycle 1. Every winner since then has been a descendant of that S3-FIFO.
- **Shrinking the ghost has paid off.** The window was halved twice, first from near capacity to half and then to a quarter. Both steps raised hit rate, though the second gain was smaller. Ghost size is the most effective tuning knob found so far.
- **The fixed probation split of about 10% has held up.** Moving away from it, whether adaptively or by enlarging it, never helped.
- **The best results came from changing one knob per variant from a strong parent.** Those cycles gave clear answers. Cycles that changed policy and representation together gave confounded ones.

## Recurring failures

- **Only hit rate has been measured, for four cycles running.** Every memory or speed claim is still unverified. That includes the smaller nodes, the fingerprint ring, the smaller ghost's savings, the cost of the sketch, and LIRS's overhead. This is the largest blind spot in the search.
- **Loosening admission has always cost hit rate.** Examples are a longer ghost window, ARC-style adaptive sizing, and fast paths from the ghost into `main`. Any change that lets keys into `main` without evidence of reuse loses.
- **Admission that is too strict also fails.** Two-hit promotion bars and routing ghost hits through `small` either did nothing or regressed. With a small probation queue, they evict keys whose reuse gap is only slightly longer than that queue.
- **Admission gates applied only to the ghost path leak.** Rejected keys get back into `main` through the other path, so stricter promotion bars and frequency gates applied only to ghost hits did little. Gates have to cover every route into `main` and compare against the real eviction victim.
- **The representation-only experiments keep being put off.** The fingerprint ring at a fixed ghost size and the graft of the hasher plus narrow indices have been recommended since cycle 2 and never run.

## Open questions

- **Where does hit rate turn over as the ghost shrinks?** It could be at an eighth of capacity or lower. Does the best size hold at other cache sizes? So far only one has been tested.
- **Can the harness be made to report `metadata-size` and `per-op-time`?** Until it can, the memory and speed hypotheses cannot be judged.
- **Do the representation changes stack on the current S3-FIFO parent without affecting hit rate?** These are the Fx hasher, `u32` links and the fingerprint ring. What speed or memory gain do they give?
- **Would TinyLFU-style admission against the real victim, applied to all insertions, pay back the cost of its sketch?**
- **Which part of cycle 4's routing variant caused its regression?** It changed both the routing of ghost hits into `small` and the two-hit bar, and the two were never tested separately.

## Per-policy notes

- **S3-FIFO:** the lineage that keeps winning. The levers that matter, in order, are ghost size (smaller has been better so far), then keeping the probation split at about 10%. The sequence-number fix for ghost bookkeeping, which stops stale entries erasing newer ones, is a correctness change and should be carried forward. Ghost hits are a useful reuse signal when the window is short but misleading when it is long.
- **CLOCK:** roughly matches LRU's hit rate, so exact LRU ordering is not needed for hit rate. Worth revisiting only if a memory or latency win is measured, and the risk of a long victim sweep on `put` has not been checked.
- **LIRS:** beats LRU but trails S3-FIFO, and is heavier because of a larger node and a linear walk to enforce its bound. A larger HIR queue made it slightly worse. Set it aside until its costs can be measured and the walk is made O(1).
- **ARC-style adaptivity:** drifted the split away from its best value and lost hit rate. Drop it.
- **Strict LRU with representation tweaks (control):** leaves hit rate unchanged, as designed. These tweaks remain candidates for grafting onto other policies but have not been measured.

## Recurring wins

- Strict admission filtering is the only lever that has raised hit rate (cycles 1–3). S3-FIFO's gain over LRU comes from controlling which keys enter `main`, not from how recency is tracked.
- Tightening the ghost pays off. Cycles 2 and 3 tested it from opposite directions: a longer ghost window lost hit rate and a shorter one (half of capacity) gained. Ghost size is a real tuning knob.
- The fixed ~10% probation split has held up. Moving away from it, whether adaptively (cycle 2) or by enlarging probation in LIRS (cycle 3), has not helped.
- Lineage: cycle 1 S3-FIFO → cycle 3 v001 (half-capacity ghost) is the strongest line so far. Cycle 2's variants were dead ends and should not be used as parents.

## Recurring failures

- No cycle has measured `metadata-size` or `per-op-time`; all three reported hit rate only. Every memory or speed claim is untested: CLOCK, the hasher and narrow-index control, the fingerprint ring, the smaller ghost, and LIRS's overhead. This harness gap is the main blocker.
- Anything that admits keys into `main` without proven reuse while resident has cost hit rate: longer ghost windows, ARC-style adaptive sizing, and fast paths from ghost to `main`.
- A ghost hit is a misleading reuse signal on loops longer than the cache. Acting on it, whether to resize or to lengthen the window, has hurt.
- Tuning a single knob alone fails when another path bypasses it. A stricter promotion bar is undone by the ghost-hit route straight into `main`.
- Planned representation-only experiments (fingerprint ring at the original ghost size, the hasher and `u32`-index graft) have been carried forward twice and never run.

## Open questions

- Where is the optimal ghost size below half of capacity, for example a quarter? At what point does it start dropping genuine re-references?
- Would sending ghost hits into `small` instead of `main` beat simply shrinking the ghost? Would that finally make `PROMOTE_FREQ` > 1 meaningful?
- Once measurement exists: does the S3-FIFO ghost cost more memory per key than removing the links saves? Does CLOCK's sweep cause tail-latency spikes on `put`?
- Are the representation changes (Fx hasher, `u32` links, MRU fast path) independent of policy, so they can be stacked onto the best S3-FIFO variant for a free speed gain?
- What actually explains the hit-rate gap between S3-FIFO and LIRS? Non-resident re-admission handling is the leading suspect, since probation size has been ruled out.

## Per-policy notes

- **S3-FIFO:** the most successful policy throughout. Keep the ~10% probation split. Ghost size matters a lot; smaller has been better so far. The ghost path and the promotion bar interact, so tune them together. Its ghost structures sit outside `Node` and their memory cost has never been measured.
- **CLOCK:** matched LRU hit rate, which shows exact LRU ordering is not needed. It has no hit-rate upside. Revisit only if measurement shows a memory or latency win.
- **LIRS:** beats LRU but trails S3-FIFO. It is heavier: larger `Node` and a linear walk to enforce the ghost bound. Enlarging the HIR fraction regressed. Set it aside unless costs can be measured and re-admission is reworked.
- **ARC-style adaptive S3-FIFO:** drifted from the good split and admitted on ghost hits, which lost hit rate. Abandoned.
- **Strict LRU + representation changes (control):** hit rate unchanged by design. Its value is purely speed and memory, which is still unmeasured and a candidate to graft onto the best policy.

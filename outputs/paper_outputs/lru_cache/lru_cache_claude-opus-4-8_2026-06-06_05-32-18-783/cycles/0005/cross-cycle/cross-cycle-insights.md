## Recurring wins

- Cheap hot-path changes that *remove* work, rather than add a branch, pay off repeatedly: the inline integer hasher, segment-aware fast paths, and the metadata layout fold all improved their target axis without a paying axis.
- Hit-rate (segmentation/admission policy) and per-op-time (hashing, surgery-skips) are orthogonal, stackable axes — the strongest variants moved both at once with neither taxing the other.
- Layout tightening can be genuinely free when casts are centralized: narrow `u32` sentinel links with the segment flag packed into a spare high bit reached the smaller node class while holding the hit-rate record and even shaving latency.
- Gated/gentle admission (route single hits to probation, require repeated evidence) lifted hit-rate without a latency tax by avoiding protected-segment churn — at least on the trace/size where it was first measured.

## Recurring failures

- Moving segment geometry (raw protected-cap widening) buys hit-rate but starves the protected-MRU fast path and regresses per-op latency — repeatedly loses to smarter admission.
- Bare, data-dependent guard branches on the *universal* hot path mispredict and regress both median and tail; the surgery-skip only pays when gated on a cheaply predictable position (inside the protected segment), never on single-list LRU where the touched node is rarely the tail.
- Unconditional ghost-to-protected re-admission over-promotes: transient post-eviction re-references evict genuinely hot keys, dropping hit-rate *below* plain SLRU while still paying ghost-bookkeeping cost.
- Per-access width casts (naive `u32`-everywhere) and control-flow "fusion" of already-correct surgery calls add cost or perturb codegen without algorithmic saving.

## Open questions

- Does the gated-ghost hit-rate gain actually reproduce on workload-a, or was it trace/size-contingent? A near-zero recall delta when folded onto the small node is a direct warning.
- With admission gating apparently tapped out on the short-reuse workload, where is remaining headroom — back on the latency/layout axis, or is the champion near this trace's ceiling?
- Can disjoint wins (small node + gated ghost) fold cleanly onto one base? The fold attempt landed neutral-to-negative; additive cross-cycle prediction has not held reliably.

## Per-policy notes

- **Segmented LRU (SLRU):** the durable champion base; its protected-MRU fast path is the key latency lever, and disturbing segment geometry costs latency. Best results stack SLRU with the integer hasher.
- **Single-list plain LRU:** segment-local fast paths do not transfer here; the surgery-skip mispredicts because the touched node is seldom the tail.
- **Ghost / reuse-distance admission:** right intuition for the LIRS-style trace but needs gentle gating; weak and possibly net-negative on the short-reuse workload, so its benefit looks trace-dependent.
- **Hashing:** the inline integer hasher is a clean, cheap, near-pure win and belongs in every base.

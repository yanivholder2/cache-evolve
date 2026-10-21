## Recurring wins

- Moving from strict LRU to a segmented LRU (SLRU) with scan resistance gave the one clear gain (cycle 1). Frequency-gated promotion and admission (a sketch gate) is the other structural benefit. Rebuilds from baseline keep rediscovering both.
- Raising the protected share helped until about 90–95%, then flattened (cycles 2–4).
- Counting rejected misses in the sketch is a safe, helpful rule. Removing it lost hit-rate. A stricter admission margin is safe only because of this rule.
- Simpler designs stay competitive: doorkeeper-free SLRU plus a sketch is a solid baseline.
- Every later win was marginal, so the incumbent looks near a local optimum on this workload.

## Recurring failures

- Weakening the protected segment lost hit-rate: a smaller share, or ghost-hit direct-to-protected admission.
- Probation-side tweaks were inert or dead by construction: demotion placement, protected-eviction second-chance bits, probation-only frequency scans, and ghost lists. Demotions and fallback paths rarely run at a high protected share.
- Sketch and gate parameters are bracketed on both sides and plateaued: aging window (longer and shorter both lost), count scale versus margin (only their ratio matters), doorkeeper, rejected-miss counting, and victim scan or rotation.
- Adaptive segment sizing and deferred admission windows found no variance to exploit.
- Cheaper-per-op shortcuts, such as skipping relinks, cannot be judged, and they cost a little hit-rate.
- Single-constant tweaks without a second trace to validate them give noise-level selection.

## Open questions

- Metadata-size and per-op-time have been missing from the fitness vectors in all ten cycles. Is it a harness fault? Until it is fixed, claimed savings (CLOCK, the 4-bit sketch, a doorkeeper-free design, relink-skip) are unverified, and selection rests on hit-rate alone.
- Do any gains hold on other traces or cache sizes? All evidence comes from one scan-heavy trace at one size. Long-distance reuse and low-scan traces are untested, and they may revive ghost lists, adaptive sizing and admission windows.
- Can a different ranking signal (size or cost awareness, or a different policy family) move the plateau?
- What is the head-to-head, cost-aware result of CLOCK + sketch against the segmented incumbent?
- Does frequency that persists across segments, or covers protected entries, add anything?

## Per-policy notes

- Strict LRU: only a baseline. Micro-optimizations that preserve its semantics cannot win on hit-rate.
- Segmented LRU (about 95% protected): the incumbent line, saturated on this trace. Remaining tweaks (margin, rotation, 4-bit conservative sketch) are optional and marginal.
- Sketch admission gate (TinyLFU-style): small gain. The extra hashing, counters and decay cost is unmeasured.
- CLOCK variants (1-bit, 2-bit frequency, CLOCK + sketch): repeatedly show an edge over strict LRU but never beat the segmented line. They are incremental on hit-rate, and their claimed value (cheap hits, small nodes) is unmeasured.
- ARC-style adaptive sizing and ghost lists: noise-level or negative, with extra bookkeeping.

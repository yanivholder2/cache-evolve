# Cross-Cycle Insights

## Recurring wins

- **Structural node compaction reliably improves or preserves latency.** u32 sentinel links (C1), protected-bit-in-prev encoding (C2), and the link-array split reducing nodes from 24→16 bytes (C5) all passed without latency regression and improved cache-line density. Structural changes that require no per-op branching are the safest wins.
- **Protected-quota enlargement monotonically improves hit-rate on this Zipfian workload.** The 75% → 87.5% → 93.75% progression across C3/C6/C7 has been cost-free once measured on a clean baseline; each step is settled and additive.
- **Policy changes succeed when they are structurally free.** The 2Q two-queue policy folded into a spare bit (C2) and the pure quota enlargement (C6) both won because they added meaningful behavioral change without growing node size or adding per-op branches. The winning pattern is behavioral improvement at zero structural cost.
- **"Do no harm" is the practical bar for winning.** The tight per-op budget means avoiding latency regression is itself a positive result; no cycle has produced a variant that measurably reduced p50 beyond the noise floor.

## Recurring failures

- **CLOCK-style deferred relinking has failed twice and is closed.** C1 lost to hand-sweep cache-line pollution; C4 lost to Vec header bloat and the same sweep cost at demotion time. Root cause: amortizing relinking only pays when promotions vastly outnumber demotions, but at full cache capacity the two rates are coupled.
- **All admission/graduation filters have failed on this workload.** SEEN_BIT splice (C4), cost-free SEEN_BIT bit-OR (C5), SEEN_BIT + quota composition (C6), eviction-path rescue (C6), and CountMin sketch (C7) all hurt or were neutral on hit-rate. The shared root cause: workload-a does not generate enough within-window repeat accesses for any frequency-based gate to create meaningful discrimination.
- **MRU-tail guard does not generalize.** It won in C2 on one workload but failed in C3 on workload-a; the guard only pays when its fire rate is high enough to amortize the check cost, which workload-a's diffuse access pattern cannot provide.
- **Adaptive mechanisms are wasted on a stationary workload.** Adaptive quota sizing (C4) never triggered an adjustment epoch yet paid a counter-increment and branch on every operation. Mechanisms designed for phase shifts are invisible wins on a stable Zipfian distribution.
- **Composing multiple winners rarely produces a net win.** Layering C2's two winners (C3) and composing SEEN_BIT with quota enlargement (C6) both failed; cumulative branch and path-length overhead exceeds the sum of individual gains.
- **`#[inline(always)]` and the `entry()` API are anti-patterns here.** I-cache displacement and borrow-checker eviction-path constraints nullify the intended benefits (C2).
- **Demotion ordering is inert at high protected quotas.** The mechanism requires frequent protected-queue overflow; at 93.75% protected that event barely fires (C7).
- **Key-clone elision is unresolved, not disproven.** The benchmark uses a `Copy` key type where clone is free; the hypothesis has never been testable under current fixture conditions.

## Open questions

- **Where is the quota reversal point?** The 75% → 87.5% → 93.75% trend is flattening; capacity/32 (~3% probationary) is the next probe and may be below the minimum viable pool size for replenishment diversity.
- **Could a frequency sketch reorder eviction within the probationary queue rather than gate graduation?** Evicting estimated low-frequency probationary items first (instead of strict FIFO) may provide discrimination without the graduation-gating overhead that has failed repeatedly.
- **Does the link-array layout (C5) show latency benefit at the current deep-quota baseline?** When first measured, it was cost-neutral; at 93.75% protected with a different memory footprint profile, the density improvement may now be measurable.
- **Can key-clone elision be confirmed or discarded?** Requires a `String`- or `Arc<str>`-keyed benchmark fixture where clone is non-trivial; three cycles have mentioned this and it has never been tested.
- **Is the recurring zero-floor baseline anomaly a measurement infrastructure bug?** It has inflated apparent regressions in multiple cycles; a fix would sharpen signal quality for latency comparisons.

## Per-policy notes

- **LRU (baseline):** u32 sentinel links are the structural baseline established in C1. Exact LRU ordering is valuable for this workload — the Zipfian distribution has enough reuse that precision in eviction ordering pays off vs. coarser approximations (CLOCK single-bit recency hurt hit-rate, C1).
- **2Q (Two-Queue):** The dominant policy since C2. Protected-bit-in-prev encoding keeps the node compact at 24 bytes (later 16 with link-array split) while providing full 2Q semantics. Protected quota is the primary remaining tuning axis; all admission-filter variants (SEEN_BIT, CountMin) are conclusively closed on workload-a. The quota trend is monotonically beneficial but flattening, suggesting a diminishing-returns plateau is near.
- **CLOCK:** Attempted in C1 (hand sweep) and C4 (bitmap-backed). Both failed. The structural reason — deferred relinking doesn't amortize when promotion and demotion rates are coupled at full cache — is fundamental, not implementation-specific. Do not revisit without a mechanism that breaks that coupling.

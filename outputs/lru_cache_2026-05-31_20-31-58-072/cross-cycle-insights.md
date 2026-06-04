## Recurring wins

- Mechanisms that record **recency** (Cold→Hot promotion, ghost-ring of just-evicted keys, cold-admission gate against scans) keep paying off on the recency-skewed trace in use; the existence of these mechanisms — not their internal shape — carries most of the lift.
- Layering complementary signals (promotion class + ghost ring) beat widening any single signal — the two together close distinct gaps that neither single-axis tweak does.
- Monotonic-magnitude tuning of a lever that already sits on the hot path (e.g. enlarging the ghost ring) produces small but real lifts; this is the safest knob to spend a slot on.
- Hunting for **precondition-collapse / instrumentation bugs** in the champion is high-leverage: a single such fix (cycle 6 v001) broke a three-cycle plateau and produced the first hit-rate lift since cycle 3. Parity-at-champion should now be treated as potentially diagnostic, not terminal.

## Recurring failures

- **Per-slot frequency-leaning state** (saturating aging counters) actively hurts on the recency-skewed trace — it shields stale-popular keys and cold-scan inserts.
- **Hot-side / pass-2 edits** (sticky demotion, Hot-ghosts, hot_credit, fixed Hot quotas) score at parity or regress: pass 2 is dormant at this cap, and the unbounded Hot pool is load-bearing, not bloat.
- **Pass-1 victim-pool re-ranking** (intra-Cold LRU, scan-taint pre-pass, Warm two-touch ladder) produced three orthogonal bit-identical parities — re-ordering doomed Cold slots doesn't change net hit-rate because the spared slot dies within a sweep or two anyway.
- **Unaugmented or naively-augmented segmented-LRU skeletons** (S4LRU plain, S4LRU with asymmetric top-heavy split + L3 ghost ring) both regress below the CLOCK-Pro-lite plateau: multi-hop promotion ladders starve staging levels and the deepest ghost ring rarely fires.
- **Ghost-ring positional weighting** is a dead knob on this trace — freshest-quarter ghost-hits behave identically to older ones.
- **SLRU evaluated at a capacity disjoint from its baseline** has happened twice; the same-size SLRU-vs-LRU question is still open purely for sampling-discipline reasons.
- **Verbatim prescriptive guidance** from a prior cycle-insight failed when applied literally (cycle 5's two-fix S4LRU recipe lost in cycle 6) — such prescriptions should be marked as conjectures to test in one slot, not multi-fix recipes.

## Open questions

- After one plateau break, how close is the post-gate champion to the workload's actual ceiling on `lirs:multi1@512`? Are further scan-admission refinements (sketch sizing, fingerprint width, counting sketch, threshold > 1) worth more than a single shared slot?
- Does the cold-admission gate's lift survive on **other capacities** of lirs:multi1 or on **different traces**, or is it tuned to this specific capacity-vs-scan-rate ratio?
- Would the parity-inert pass-1 / Hot-side knobs differentiate at smaller capacities or on traces where the Cold pool churns faster and pass 2 actually fires?
- Are there other latent precondition-collapse or measurement bugs in the champion lineage masking real levers? One such bug already concealed a working mechanism for a cycle.
- Is the loop's productive bottleneck still policy design at all, or has it shifted decisively to **evaluation-surface breadth** and **fitness-vector breadth** (per-op time, metadata size) — both flagged unaddressed in every cycle so far?

## Per-policy notes

- **Baseline LRU**: structurally permissive — three large representational rewrites in cycle 1 all preserved P1/P2/P3 invariants and arena-slot reuse cleanly.
- **SLRU / S4LRU**: every attempt has failed to land, either via sampling-discipline gaps (cycles 1, 2) or via real structural regression on this trace (cycle 5 plain, cycle 6 with prescribed two-fix). Do not queue another segmented-LRU variant without matched-capacity sampling and a concrete reason to expect it to beat the ghost-anchored single-hop skeleton.
- **CLOCK (plain)**: was the first variant to land on the Pareto front (cycle 1 v002); a recency-only second-chance bit was the right starting shape for this trace.
- **GCLOCK / aging-counter CLOCK**: regressed; the frequency-leaning bit is the wrong shape here. Lineage should stay dropped until other workloads are scored.
- **CLOCK-Pro-lite (Cold/Hot + ghost ring)**: the working skeleton for the loop. Structural credit is now decisive — alternative skeletons and Hot-pool budget caps have all lost. Internal shape has plateaued: ghost-ring sizing is in strong diminishing returns; positional weighting is inert; Hot-side / pass-2 knobs are dormant at this cap; pass-1 victim re-ranking is inert.
- **Cold-admission gate (sketch-on-`put`)**: current working champion (cycle 6 v001). Confirmed real on one trace at one capacity; generality unproven. Worth at most one collective slot of narrow refinement before pivoting to breadth.

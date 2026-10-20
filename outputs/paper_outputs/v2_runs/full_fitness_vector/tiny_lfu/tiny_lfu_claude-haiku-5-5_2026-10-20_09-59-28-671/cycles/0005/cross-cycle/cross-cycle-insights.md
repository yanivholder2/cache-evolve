# Cross-cycle insights

## Recurring wins

- Constant-factor, eviction-order-preserving changes are the most reliable source of gains. Hoisting the hash, inlining accessors, and splitting the put-side unlink and promotion path all cut per-op time with hit rate at or near parent.
- Keeping per-row sketch seeds in a module-level `const` table, not per instance, removes the struct growth that per-instance caching caused while keeping the speedup.
- Packing protected state into existing node fields reduces metadata without moving hit rate, so bookkeeping cost can be trimmed independently of policy.
- Protected/probation segmentation is the only change that has moved hit rate upward. The gain is small and seems to require the protected set to keep turning over.

## Recurring failures

- Recency rotation of rejected admission victims lowered hit rate and added per-op cost. The frequency gate already does the protective work.
- Changes to the admission and aging inputs did not raise hit rate: the one-hit-wonder doorkeeper, shorter sketch aging, and a W-TinyLFU-style recency window. Stale counts and one-hit pollution do not appear to be what limits the gate on this workload.
- Capping the protected segment without demotion lost hit rate, so caps are the wrong lever for segmentation.
- Matching the public per-row seeded indexing restored admission fidelity but brought back per-row work on get and put, giving back most of the speedup. The current formulation cannot keep both.
- Per-instance seed caching grew the struct, which the change was meant to avoid.

## Open questions

- Do the precomputed or `const`-table counter indices match the public per-access derivation for every key, or only in aggregate hit rate? Hit-rate parity is consistent with equivalence but does not prove it.
- Can the public per-row indices be produced with less per-row work than the current mixing, so index correctness does not cost the speed gain?
- Do the segmentation hit-rate gain and the put-side timing improvements survive repeat runs?
- Is the storage for the doorkeeper and window fully reflected in the reported metadata, or is it undercounted?
- Does any of this hold on a second trace, or on a trace where one-hit keys dominate?
- How much of the get-path and unlink savings depends on the share of accesses that hit the newest key?

## Per-policy notes

- Admission (frequency sketch / TinyLFU gate): hashing and seed-layout changes that keep indices fixed are the safe path. Index equivalence against the public path is the main correctness question. Doorkeeper, shorter aging, and the window have not helped.
- Eviction order (LRU-style recency): eviction-order-preserving guards are productive. Rotating rejected victims to the MRU tail is not.
- Segmented (probation/protected) LRU: the only hit-rate mover so far. Capping it hurts, and its per-node and per-cache metadata cost is the price. It needs validation on another trace before further building.
- Recency window (W-TinyLFU style): no hit-rate movement, with added list work on both get and put. Latency got worse.
- Across all policies: every cycle's evidence comes from one trace (lirs workload-a), and source diffs were not inspected. None of the gains has been shown to generalize, and the mechanism links are inferred from hypotheses and fitness results.

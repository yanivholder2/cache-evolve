# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-20T10:24:31+00:00

## timeline

### 10:24:39 thinking



### 10:24:44 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-cycle insights\n\n## Recurring wins\n\n- Constant-factor, eviction-order-preserving changes are the most reliable source of gains. Hoisting the hash, inlining accessors, and splitting the put-side unlink and promotion path all cut per-op time with hit rate at or near parent.\n- Keeping per-row sketch seeds in a module-level `const` table, not per instance, removes the struct growth that per-instance caching caused while keeping the speedup.\n- Packing protected state into existing node fields reduces metadata without moving hit rate, so bookkeeping cost can be trimmed independently of policy.\n- Protected/probation segmentation is the only change that has moved hit rate upward. The gain is small and seems to require the protected set to keep turning over.\n\n## Recurring failures\n\n- Recency rotation of rejected admission victims lowered hit rate and added per-op cost. The frequency gate already does the protective work.\n- Changes to the admission and aging inputs did not raise hit rate: the one-hit-wonder doorkeeper, shorter sketch aging, and a W-TinyLFU-style recency window. Stale counts and one-hit pollution do not appear to be what limits the gate on this workload.\n- Capping the protected segment without demotion lost hit rate, so caps are the wrong lever for segmentation.\n- Matching the public per-row seeded indexing restored admission fidelity but brought back per-row work on get and put, giving back most of the speedup. The current formulation cannot keep both.\n- Per-instance seed caching grew the struct, which the change was meant to avoid.\n\n## Open questions\n\n- Do the precomputed or `const`-table counter indices match the public per-access derivation for every key, or only in aggregate hit rate? Hit-rate parity is consistent with equivalence but does not prove it.\n- Can the public per-row indices be produced with less per-row work than the current mixing, so index correctness does not cost the speed gain?\n- Do the segmentation hit-rate gain and the put-side timing improvements survive repeat runs?\n- Is the storage for the doorkeeper and window fully reflected in the reported metadata, or is it undercounted?\n- Does any of this hold on a second trace, or on a trace where one-hit keys dominate?\n- How much of the get-path and unlink savings depends on the share of accesses that hit the newest key?\n\n## Per-policy notes\n\n- Admission (frequency sketch / TinyLFU gate): hashing and seed-layout changes that keep indices fixed are the safe path. Index equivalence against the public path is the main correctness question. Doorkeeper, shorter aging, and the window have not helped.\n- Eviction order (LRU-style recency): eviction-order-preserving guards are productive. Rotating rejected victims to the MRU tail is not.\n- Segmented (probation/protected) LRU: the only hit-rate mover so far. Capping it hurts, and its per-node and per-cache metadata cost is the price. It needs validation on another trace before further building.\n- Recency window (W-TinyLFU style): no hit-rate movement, with added list work on both get and put. Latency got worse.\n- Across all policies: every cycle's evidence comes from one trace (lirs workload-a), and source diffs were not inspected. None of the gains has been shown to generalize, and the mechanism links are inferred from hypotheses and fitness results.\n"
}
```

### 10:24:46 chat

I wrote `cross-cycle-insights.md` to the cross-cycle output directory for cycle 0005. It has the four required sections in order, built only from the inlined ledger digest. I did not read the previous version of the file.

The main signals across the five cycles:

- **Recurring wins:** constant-factor changes that keep eviction order have repeatedly cut per-op time with hit rate unchanged. Keeping seeds in a `const` table avoids the struct growth that per-instance caching caused. Segmentation is the only change that has moved hit rate upward, and only slightly.
- **Recurring failures:** rotating rejected victims to the MRU tail, the doorkeeper, shorter sketch aging, and the recency window did not raise hit rate. Capping the protected segment lost hit rate. Restoring the public per-row indexing fixed admission fidelity but brought back the per-row cost.
- **Open questions:** whether the `const`-table indices match the public path for every key, whether the public indexing can be made cheaper, whether the segmentation and put-side gains survive repeat runs, and whether anything holds on a second trace.
- **Per-policy notes:** grouped by admission gate, eviction order, segmented LRU, and the recency window. Every cycle's evidence comes from one trace (lirs workload-a), so none of the gains has been shown to generalize.


---

- finished: 2026-10-20T10:24:47+00:00
- duration: 15.46s
- status: ok
- input_tokens: 4
- output_tokens: 2996
- cache_read_tokens: 37204
- cache_write_tokens: 10303
- cost_usd: 0.003931

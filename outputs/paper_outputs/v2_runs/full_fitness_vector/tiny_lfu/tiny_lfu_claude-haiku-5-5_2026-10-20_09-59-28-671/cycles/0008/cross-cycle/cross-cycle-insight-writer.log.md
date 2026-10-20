# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-20T10:36:22+00:00

## timeline

### 10:36:29 thinking



### 10:36:35 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-cycle insights\n\n## Recurring wins\n\n- Per-op-time gains that leave eviction order and hit rate unchanged keep recurring. Hoisting the hash, inlining get-path accessors, and splitting the put-side unlink/promotion path all improved timing without moving hit rate.\n- Keeping per-row sketch seeds in a module-level `const` table, not per instance, removed the struct growth that per-instance caching caused and kept the speedup.\n- Packing protected state into existing node fields cut metadata without changing hit rate, so segmentation's bookkeeping cost can be trimmed independently of the policy.\n- Collision spread is the lever that has moved hit rate. Conservative counter updates and reshaping the sketch toward more columns and fewer rows both reduced inflation from colliding keys and gave small gains.\n\n## Recurring failures\n\n- Recency rotation of rejected admission victims lost on both hit rate and per-op time. The frequency gate already does that protective work.\n- Admission and aging tweaks have not raised hit rate on this workload: a one-hit-wonder doorkeeper, a recency window in front of the gate, halved sketch aging, and admission tie-breaking all left hit rate flat or lower. Stale counts and one-hit pollution do not appear to be the bottleneck.\n- Capping or shrinking the protected segment cost hit rate and per-op time. Segmentation's gain seems to depend on the protected set turning over, not on bounding it.\n- Layouts that change which keys collide cost hit rate, and layouts that restore collision fidelity cost speed. Cache-line grouping gave a clear speedup but changed collisions; restoring public per-row indexing recovered hit rate but reintroduced per-row work. No variant has yet had both.\n- Changes that add a per-row lookup or extra pass on every access consistently regressed get and put timing.\n\n## Open questions\n\n- Does the collision-spread gain from fewer rows and more columns hold on a second trace and at other cache sizes?\n- Can conservative counter updates keep their collision benefit without an extra pass over all rows?\n- Can public-equivalent per-row indices be produced with less work than the seeded mixing, so index fidelity and speed stop trading off?\n- Do the `const`-table seed indices match the per-access derivation for every key, or only in aggregate hit rate?\n- Do put and get timing shifts survive repeat runs? Many were single-run results.\n- Does a protected-capacity setting exist that exercises demotion without losing hit rate?\n- Does any of this hold on a trace where one-hit keys dominate, or on a second trace at all?\n- What caused the unexplained metadata and struct size movements in several variants, given that the hypotheses forbade them?\n\n## Per-policy notes\n\n- Admission gate (frequency sketch plus frequency-based victim check): the main source of protection. Changes to the gate's inputs, such as doorkeepers, windows, aging, and tie-breaks, have not improved hit rate. Changes to the sketch's collision behaviour are the exception that has moved it.\n- Eviction and recency (LRU-style list ordering): reordering rejected victims was harmful. Pointer-work trims that preserve eviction order are safe, and those are the most reliable speed gains.\n- Segmentation (probation/protected): segment size is sensitive. Capping or shrinking the protected share hurt, and tail demotion has seen little hit-rate effect. Metadata cost is the main drawback, and it can be reduced by packing flags into existing fields.\n- Frequency sketch layout: seeds, row/column shape, and block grouping drive both hit rate and speed. Layout changes that alter collisions trade hit rate for speed, and the best-performing direction so far is the collision-spread change.\n- Evidence base: all results are from one trace (workload-a) at one cache size, mostly from single runs, and overlay source was not inspected. Treat every attribution as provisional.\n"
}
```

### 10:36:38 chat

I wrote `cross-cycle-insights.md` to the cycle 0008 cross-cycle output directory. It has the four required sections in order, built only from the inlined ledger digest. I did not read any previous version of the file.

The main signals across cycles 0001–0008:

- **Recurring wins:** per-op-time gains that leave eviction order and hit rate unchanged, metadata packing into existing fields, and `const`-table seed storage. Collision-spread changes (conservative counter updates, more columns and fewer rows) are the only recent hit-rate movers.
- **Recurring failures:** recency rotation of rejected victims, admission and aging tweaks (doorkeeper, window, shorter aging, tie-breaks), capping or shrinking the protected segment, and per-row work added on every access.
- **Open questions:** whether the collision-spread gain holds on a second trace and other cache sizes, whether the timing shifts survive repeat runs, and whether index fidelity can be restored without the speed loss.
- **Per-policy notes:** grouped by admission gate, recency, segmentation, and sketch layout. All evidence comes from one trace at one cache size, mostly single runs, so the file marks attributions as provisional.

The file has no specific numbers, since the digest is qualitative and the constraints forbid inventing them.


---

- finished: 2026-10-20T10:36:39+00:00
- duration: 16.54s
- status: ok
- input_tokens: 4
- output_tokens: 2922
- cache_read_tokens: 39846
- cache_write_tokens: 12881
- cost_usd: 0.004436

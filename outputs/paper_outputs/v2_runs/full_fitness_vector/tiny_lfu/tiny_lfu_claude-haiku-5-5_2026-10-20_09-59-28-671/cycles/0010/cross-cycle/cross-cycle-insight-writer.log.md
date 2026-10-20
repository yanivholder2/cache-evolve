# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-20T10:43:56+00:00

## timeline

### 10:44:05 thinking



### 10:44:11 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-cycle insights\n\n## Recurring wins\n\n- Constant-factor, eviction-preserving work is the reliable source of gains. Hit rate stayed at the parent's value in most cases while per-op time moved (hash reuse, seed caching, single-loop updates, get-path short-circuits).\n- Keeping per-row sketch seeds in a module-level `const` table, not per instance, kept the speedup without growing structs. Hit rate stayed flat.\n- Packing protected-segment state into existing node fields cut metadata without changing hit rate.\n- Collision-spread changes in the sketch are the only lever that has moved hit rate upward. Conservative updates and a wider, shallower sketch each showed small gains. Narrow counters showed a gain once and a loss once, so the result is unsettled.\n- Index-preserving changes keep hit rate; changes that alter which keys collide move it.\n\n## Recurring failures\n\n- Changes to admission or aging did not raise hit rate: a doorkeeper, shorter frequency aging, a recency window in front of the gate, and admission tie-breaking. The frequency gate appears not to be the limiting factor on this workload.\n- Shrinking or capping the protected segment cost hit rate. Protected-tail demotion alone left hit rate unchanged.\n- Recency rotation of rejected victims hurt both hit rate and per-op time.\n- Per-row work added on get or put paths (per-row indexing, per-row column lookups, extra list moves) cost per-op time and gave back earlier speedups.\n- Single-run timing shifts were repeatedly unexplained or not reproduced. Treat put-side timing claims as unverified until repeat runs confirm them.\n\n## Open questions\n\n- Do the collision-spread gains (conservative update, row/column trade, narrow counters) hold on a second trace and at other cache sizes? All evidence so far is one trace at one size.\n- Is the narrow-counter result a sizing error? The u8 variant's sketch came out at half its intended size, so it needs a re-run before the direction is judged.\n- Do the const-table seeds produce the same counter indices as the public per-row path for every key, or only in aggregate hit rate? This has been open since the early cycles.\n- Can the public per-row indexing be made cheap enough to keep its fidelity without losing the speed gain?\n- Does the get-path MRU short-circuit and the single-loop conservative update's put-tail gain survive repeat runs?\n- Does the one-hit-wonder-heavy trace ever exist in this set? The doorkeeper was never tested where it could matter.\n- Cycle 0001 labels its trace \"lirs workload-a\"; later cycles say \"workload-a\". Confirm these name the same trace.\n\n## Per-policy notes\n\n- TinyLFU-style admission (count-min sketch with a frequency gate): hit-rate movement has come only from sketch collision spread. Admission-side and aging-side changes have not helped.\n- Segmented LRU (protected and probation segments): the protected segment's capacity matters more than its tail turnover. Reducing capacity hurt; forcing demotion more often did not pay off.\n- Sketch layout (rows, columns, counter width, cache-line grouping): layout changes that preserve collision patterns hold hit rate. Those that change collisions can gain or lose and need trace-level confirmation.\n- Metadata: packing into existing fields is safe. Per-instance seed storage added growth and should stay in constant tables.\n"
}
```

### 10:44:14 chat

I wrote `cross-cycle-insights.md` in the cross-cycle output directory, using the four required sections in order. I did not read the previous file, and I used only the ledger digest from the prompt.

The main points across cycles 0001–0010:

- **Recurring wins:** Constant-factor changes that leave eviction order alone have been the most reliable source of gains. Keeping sketch seeds in a `const` table avoided struct growth. Collision-spread changes to the sketch are the only lever that has moved hit rate upward, though the evidence is small and mixed.
- **Recurring failures:** Admission and aging changes (doorkeeper, shorter aging, window, tie-breaking) did not raise hit rate. Shrinking the protected segment cost hit rate. Per-row work on hot paths gave back speed. Single-run timing claims have repeatedly failed to hold up.
- **Open questions:** Whether the collision-spread gains hold on a second trace or cache size. Whether the 8-bit counter result is a sizing error that needs a re-run. Whether the const-table seeds match the public indexing for every key. Whether the get-path and put-path timing gains survive repeat runs.
- **Per-policy notes:** Grouped under TinyLFU-style admission, segmented LRU, sketch layout, and metadata.

Cycle 0001 labels its trace "lirs workload-a", while later cycles say "workload-a". I flagged this in the open questions because I couldn't tell from the ledger whether they refer to the same trace.


---

- finished: 2026-10-20T10:44:14+00:00
- duration: 17.95s
- status: ok
- input_tokens: 4
- output_tokens: 3287
- cache_read_tokens: 41454
- cache_write_tokens: 14786
- cost_usd: 0.005016

# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0001/cross-cycle
- allow_edits: True
- started: 2026-10-28T18:29:16+00:00

## timeline

### 18:29:26 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0001/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n_Projection of 1 completed cycle (0001). Nothing below has been repeated across cycles yet, so every item is provisional._\n\n## Recurring wins\n\n- (None confirmed across cycles yet.) Cycle 0001's best lead was fixing the stash's ordering so that items already reused outlast brand-new or never-hit items. Changing where demoted items enter the stash (at the MRU end) improved hit-rate on every trace and cache size.\n- A change that only reorders lookups, without touching what gets admitted or evicted, left hit-rate exactly the same. Such changes can be stacked onto a winner without risking hit-rate.\n\n## Recurring failures\n\n- (None confirmed across cycles yet.) A fixed promotion gate (two hits before a stash item promotes) helped small, crowded caches. At large sizes it gained nothing or slightly hurt (web_0). A gate that ignores capacity looks fragile.\n- The evaluation only measured hit-rate. That made the cost-focused variants unscorable, and every per-op-time and metadata-size claim is still unverified. A cost-only variant gives no signal until the evaluation measures cost.\n\n## Open questions\n\n- Do the stash-entry position (MRU demotion) and the promotion gate add up when combined, or overlap? Both protect reused items from one-hit arrivals.\n- Does a promotion gate that scales with occupancy or capacity keep the small-cache gains without the large-cache regression?\n- Does target-first lookup order actually lower get latency? This needs an evaluation that reports per-op-time and metadata-size.\n\n## Per-policy notes\n\n- Stash/two-tier baseline lineage: improvements come from how items move in and out of the stash, and the benefit depends on cache size. Fixes at the entry point (demotion position) help more at large sizes, where more items get demoted. Fixes at the exit point (promotion gate) help more at small sizes, where the normal tier is crowded.\n- Current lineage head: v001 (MRU-end demotion), from cycle 0001.\n"
}
```

### 18:29:31 chat

I wrote `cross-cycle-insights.md` in the cross-cycle output dir, with the four required sections in order. The ledger has only one completed cycle (0001), so nothing has repeated across cycles yet. The file says so, and every item in it is marked as provisional.

- **Recurring wins:** Cycle 0001's best lead was v001, which puts demoted items at the most-recently-used end of the stash. It improved hit-rate on every trace and cache size. Changing only the lookup order (v003) left hit-rate exactly the same, so that change can be added to a winner safely.
- **Recurring failures:** The fixed two-hit promotion gate (v002) helped small, crowded caches but gained nothing or slightly hurt at the large size (web_0). The evaluation measured only hit-rate, so every claim about speed or metadata size is untested.
- **Open questions:** Whether v001 and v002 add up when combined; whether a gate that scales with cache occupancy keeps the small-cache gains without the large-cache loss; and whether the lookup reorder actually lowers latency.
- **Per-policy notes:** In the stash/two-tier lineage, changing how items enter the stash helps more in large caches. Changing how they leave it helps more in small ones. The current best variant is v001.


---

- finished: 2026-10-28T18:29:31+00:00
- duration: 15.14s
- status: ok
- input_tokens: 4
- output_tokens: 1233
- cache_read_tokens: 35058
- cache_write_tokens: 5935
- cost_usd: 0.079168

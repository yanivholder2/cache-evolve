# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-11-05T23:12:37+00:00

## timeline

### 23:13:12 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0007)\n\n## Recurring wins\n\n- **Keep proven items resident.** The two-hit promotion gate, MRU demotion into stash, probing overflow and the probing-room gate each raised hit rate a little on every trace and size, and the gains stacked (0001–0004). The current lineage is gate + MRU demotion + probing overflow + probing-room gate (0004 v002).\n- **MRU demotion for every demoted item works with or without the gate** (0002, 0003). Variants that weakened it lost hit rate (0005 v003).\n- **Fewer cascades help only if stash keeps draining.** Probing overflow cut cascades and kept items moving out of stash, which recovered `put` p99 (0003). The single bounded deferral did the same (0005 v002).\n- **Making the stash probe cheaper is the only lever that has moved `get` p50.** The stash presence mask (0007 v002, the first Winner) lowered p50 with bit-identical hit rate.\n- **Cheap per-item state** (reusing the `lfu_counter` field) and **hashing once per lookup** improve things without growing metadata or changing hit rate. They are easy to stack onto other changes.\n\n## Recurring failures\n\n- **Skipping or deferring a cascade only moves its cost.** Unbounded deferral backed up the stash (0004). Bounded deferral pushed the cost onto a later `get` (0005). Skipping elsewhere pushed it onto `put` or onto probing lookups (0004).\n- **Gating deferral on the state of one stash bucket fails.** Fill level is always near full (0006), and the marked-LRU signal (0007) just shifted cost between `get` and `put`. Neither separated a harmful deferral from a harmless one.\n- **Changes to the common hit path other than the stash probe have done nothing for p50.** This covers lookup order (stash last made things worse, 0003), scan direction (0005), skipping the MRU reorder (0006), cascade avoidance (0004) and repeated hashing (0007).\n- **Extra work on stash churn shows up on the hot path.** Examples are mask recompute on removal (0007), stash pressure from longer residency (0002, 0005) and the fused rotate at a second call site (0006).\n- **The ungated MRU + overflow line** lost on `get` p99 because of the probing lookup penalty (0004). **Liveness signals taken from a single normal-tier stay** lost hit rate (0005).\n- **Small p99 moves on paths a change doesn't touch** keep appearing (0005–0007). Treat them as possible noise or codegen effects until a rerun confirms them.\n\n## Open questions\n\n- Can the stash mask be maintained in O(1) on removal (counting bits, or bits stored next to bucket metadata) and keep the p50 gain without the `put` and `get` p99 cost?\n- How much of the mask's tail cost comes from false positives and how much from refresh? Would a wider fingerprint help?\n- Once the stash probe is cheap, does the Dash-level segment hash hold any of the remaining p50 gap?\n- Can the fused in-place rotate, limited to the third-hit path or kept out-of-line, keep its `get` p99 gain without the `put` regression?\n- Is there a deferral signal other than stash-bucket state that stops the small-cache hit-rate loss?\n\n## Per-policy notes\n\n- **Dash (only policy studied so far):**\n  - hm_0 at the large size is the most sensitive cell for retention. It gains most from changes that keep items longer and loses most when retention drops. Use it as the first check for hit-rate regressions.\n  - Cache size changes whether stash residency helps: longer stash stays help the large cache and hurt the small cache.\n  - Stash hits are a large share of all hits, so checking stash first is the right order. The goal is to make that check cheaper, not to move it.\n  - Base candidates: 0004 v002 for p50 and cost work, 0005 v002 for deferral and tail work, 0007 v002 (stash mask) as the base for the next round of mask-maintenance changes.\n"
}
```

### 23:13:21 thinking



### 23:13:21 chat

I've written `cross-cycle-insights.md` in the cycle 0007 cross-cycle directory. It covers all seven cycles in the four required sections, in order, and uses no numbers.

- **Recurring wins:** Changes that keep proven items resident raised hit rate a little everywhere, and the gains stacked. These are the two-hit gate, demoting into the most-recently-used end of the stash, probing overflow and the probing-room gate. Cutting cascades only helped when items kept flowing out of the stash. The stash presence mask (0007 v002, the only Winner so far) is the only change that has moved median `get` latency.
- **Recurring failures:** Deferring or skipping a cascade only moves its cost to another operation. Gating the deferral on how full a stash bucket is, or on whether its least-recently-used item is marked, did not work. Every common-path change other than the stash probe left median `get` flat. Extra work on stash churn shows up as slower operations. Small p99 shifts on paths a change doesn't touch need a rerun before anyone trusts them.
- **Open questions:** Can the stash mask be updated cheaply on removal? How much of its tail cost comes from false positives? Does the segment hash hold any of the remaining median gap? Can the fused rotate be confined to the third-hit path? Is there a deferral signal that doesn't come from stash-bucket state?
- **Per-policy notes:** Dash is the only policy studied so far. hm_0 at the large size is the cell most sensitive to retention changes. Cache size decides whether longer stash stays help. The notes also list which earlier variant to use as the base for each line of work.


---

- finished: 2026-11-05T23:13:22+00:00
- duration: 44.65s
- status: ok
- input_tokens: 4
- output_tokens: 2048
- cache_read_tokens: 40668
- cache_write_tokens: 12205
- cost_usd: 0.147533

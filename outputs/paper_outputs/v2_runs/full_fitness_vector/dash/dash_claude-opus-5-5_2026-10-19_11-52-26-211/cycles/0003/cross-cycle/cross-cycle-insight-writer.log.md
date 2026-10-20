# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-19T12:17:49+00:00

## timeline

### 12:18:04 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Treating the stash as a scan-resistant probationary queue, ordered by proven reuse, is the only hit-rate lever that has paid off. Demotees go to the stash's MRU end (cycle 1) and fresh puts go to its LRU/Front end (cycle 2). The two gains stack, and this pair is the current best parent.\n- Changes to independent parts of the get/put path (stash insert position, lookup order, stash-hit destination) combine cleanly. The cache state you get after combining them is what you would predict.\n- The only latency gains you can trust come from skipping work: probing the tier that serves most hits first, or avoiding the remove→promote→demote→reinsert cascade. Credit a latency gain only when the code change explains it.\n- Metadata-size has not moved in any variant so far. Placement and ordering changes are free on that dimension.\n\n## Recurring failures\n\n- Changing how items get promoted out of the stash, in either direction, has always cost hit-rate. MRU placement on stash→target (cycle 2) failed. So did a second-hit gate on stash→target and an ownership gate on target→probing (cycle 3). Default LRU-ordered Front promotion looks like a local optimum. The buckets are too small for any extra filter to beat plain recency.\n- Placement tweaks on rare corner paths (such as balanced placement on stash hits) are safe but rarely trigger, so the change barely shows up.\n- A fixed lookup order doesn't carry over from one lineage to another. Target-first won when stash hits were rare (cycle 1). It lost once fresh-at-Front placement routed first reuse through the stash (cycle 3). The best order depends on placement policy.\n- Latency comparisons across runs are unreliable. Big shifts showed up on paths a diff never touched (most of all in cycle 1), and most put-path moves look like noise.\n- Every get-tail reduction so far came from promoting less often, and each one gave back hit-rate. Cutting how often promotion happens is not a free latency lever.\n\n## Open questions\n\n- Can a promotion cascade be made cheaper without changing which items get promoted? Options include in-place swaps instead of remove+reinsert, or a cap on how deep a cascade goes.\n- Can bucket scans be made cheaper, for example with per-slot fingerprints, at an acceptable metadata-size cost?\n- Structural levers are still untested: stash capacity and associativity, and the eviction order inside target buckets. Would bigger stash buckets let admission filters work where 2-slot buckets couldn't?\n- How do hits split between the target, probing and stash tiers under the current parent? It has never been measured directly, yet it decides lookup order and where cascade cost matters.\n- Would balanced or spare-neighbour placement at initial put or target-hit time trigger often enough to matter?\n- Would an adaptive or per-key lookup order beat a fixed one, or would the extra branches cost more than they save?\n\n## Per-policy notes\n\n- Stash (dominant capacity tier): its internal eviction order is the main hit-rate lever. Fresh entries go to Front (LRU end) and reused demotees go to Back (MRU end). As hit-rate rises, more traffic lands on the slow stash-hit→promote path, which slightly erodes get p50.\n- Target buckets: these are tiny, so one MRU-placed promotion evicts residents that were already hit. Front placement on promotion works as an implicit admission filter and should stay. Eviction order inside target buckets hasn't been tested.\n- Probing buckets: these hold the hottest items. Gating promotion into them (native-ownership gate) lost real reuse. The climb within each bucket already keeps probing selective.\n- Lookup order: this is a latency-only knob with no effect on hit-rate. Pick it based on the measured hit split under the current placement policy, not from earlier cycles' results.\n"
}
```

### 12:18:11 chat

I've written `cross-cycle-insights.md` in the cross-cycle output folder. It draws only on the three cycle summaries and keeps the four required sections in order.

- **Recurring wins:** The only hit-rate gain so far comes from using the stash as a waiting area ranked by proven reuse. Fresh puts go to its least-recent end and reused demoted items go to its most-recent end. The two gains add up, and that pair is the current best parent. Changes to independent parts of the path combine cleanly. The only latency gains worth trusting are ones the code change explains, such as checking the busiest tier first or skipping the promotion cascade.
- **Recurring failures:** Every change to promotion out of the stash, in either direction, has lowered hit-rate. That includes hotter placement and both kinds of stricter admission. Cycle 1's best lookup order flipped in cycle 3 once placement changed, so it doesn't carry over. Latency comparisons between runs are unreliable. Every cut to the get tail so far came from promoting less often and cost hit-rate.
- **Open questions:** Making the cascade cheaper without changing which items get promoted, and cheaper bucket scans. Still untested: stash size and associativity, and eviction order inside target buckets. Also open: a direct count of how hits split between the tiers, placement at initial put time, and whether an adaptive lookup order would help.
- **Per-policy notes:** Grouped by tier (stash, target buckets, probing buckets) plus lookup order, which affects latency only and should be chosen from the measured hit split under the current placement.


---

- finished: 2026-10-19T12:18:11+00:00
- duration: 22.37s
- status: ok
- input_tokens: 4
- output_tokens: 1934
- cache_read_tokens: 35836
- cache_write_tokens: 8580
- cost_usd: 0.114503

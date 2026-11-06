# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-11-05T22:54:53+00:00

## timeline

### 22:55:10 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Retention levers reliably lift hit rate.** The two-hit gate on stash→normal promotion (c1), MRU demotion into stash (c2, c3), and \"don't evict a proven resident\" / probing-room placement (c2, c4) each raised hit rate on every cell tested. MRU demotion helps even without the gate (c3).\n- **hm_0 at the large size is the most sensitive cell.** Across all five cycles it gained the most from changes that keep items with long re-reference gaps, and lost the most from changes that weaken retention. Use it as the first check on any retention change.\n- **Probing overflow on the gated MRU line improves the tail.** Cycle 3 showed it reduces demotion cascades while items keep flowing out of stash, and that recovered most of the `put` p99 regression. Of all the latency levers tried, it is the only one that has clearly worked.\n- **A bounded deferral keeps stash draining.** The single-deferral variant in c5 recovered `put` p99, where the unbounded deferral in c4 had made it much worse.\n- **Reusing existing per-item fields (e.g. `lfu_counter`) keeps metadata size flat.** Use this for any new per-item state.\n\n## Recurring failures\n\n- **Skipping or deferring cascades only relocates their cost** (c4, c5). The cost lands on `put` through fuller buckets, or on a later `get` through foreign residents in probing or a forced promotion. Counting cascades is the wrong goal; what matters is how full stash is and which path pays the cost.\n- **Lookup and scan ordering does not move `get` latency.** Checking the normal tier before stash (c3) made `get` worse, because stash hits are a large share of all hits. Reversing the in-bucket scan (c5) changed nothing, because the buckets that miss still need full scans.\n- **Weakening retention costs hit rate, worst on hm_0 at the large size.** Examples: dropping the MRU move on a first stash hit (c2), and sending unreused demotions to stash LRU based on a single-stay liveness signal (c5).\n- **The ungated MRU + overflow branch is not a cheaper alternative** (c4). The lookup penalty for items placed in probing outweighed the `put` savings.\n- **Every variant so far has regressed at least one p99, so there are no Winners after five cycles.** Some of these small tail moves may be noise, since even cost-only changes moved the tail (c5).\n- **Hit rate and latency trade off against each other on this line.** Each cycle's best variant gained hit rate and gave back some latency. The gate's `get` p50 cost has never been recovered.\n\n## Open questions\n\n- Where is the `get` p50 gap on the common hit path? The remaining suspects are how many buckets each hit probes (stash is checked on every lookup) and the shift cost when an item moves to MRU. Cascades are ruled out (c4, c5), and so are scan and lookup order (c3, c5).\n- Can the stash-hit promotion chain (remove → promote → demote) be made cheaper in itself, rather than skipped or deferred?\n- Should deferral depend on stash pressure? The goal is to keep the large-cache gain and the `put` p99 recovery without losing hit rate in the small cache.\n- Can the forced promotion on the third hit be moved off the `get` path?\n- Can noise be separated from real effects on small p99 moves, for example with repeated runs, before more tail-driven decisions are made?\n\n## Per-policy notes\n\n- **Dash (only policy studied so far):** the base lineage is gate (c1) → + MRU demotion (c2) → + probing overflow (c3) → + probing-room gate (c4) → optionally + single deferral (c5).\n- **Dash:** the cache size decides whether longer stash residency helps. It helps the large cache by keeping proven residents in place. It hurts the small cache because deferred items take up stash slots.\n- **Dash:** the `get` slowdown follows stash-hit traffic, not the gate itself (c3). The `put` p99 follows how full stash is (c2–c5).\n- **Dash:** H01 and H03 from cycle 1 were lost to build HTTP 429 errors, and were later folded into the lineage as MRU demotion and probing placement. Infra failures carry no signal about the idea that was being tested.\n"
}
```

### 22:55:17 thinking



### 22:55:17 chat

I wrote `cross-cycle-insights.md` from the five-cycle ledger, using the four required sections in order. After five cycles there are still no Winners: every variant gained hit rate and made at least one p99 slightly worse.

- **What keeps working:** keeping proven items alive longer raises hit rate on every cell. That covers the two-hit gate, demoting to the stash MRU end, and placing promotions in free probing space. hm_0 at the large size is consistently where these changes show up most, in both directions. Probing overflow (cycle 3) is the only latency change that has clearly worked: it cut cascades while items kept flowing out of stash, and that recovered most of the `put` p99 regression. A deferral capped at one promotion (cycle 5) also kept stash draining.
- **What keeps failing:** skipping or deferring cascades only moves their cost to `put` or to a later `get`. Changing lookup order or scan direction does nothing for `get` latency. Anything that weakens retention loses hit rate, worst on hm_0 at the large size. Some of the small tail moves may be noise.
- **Open threads:**
  - The `get` p50 gap is most likely in how many buckets each hit probes or in the cost of moving an item to MRU.
  - The stash-hit promotion chain itself could be made cheaper.
  - Deferral could depend on how full stash is, to avoid the small-cache hit-rate loss.
  - Repeated runs would show which small p99 moves are real.
- **Per-policy notes:** Dash is the only policy studied so far. The notes list its base lineage cycle by cycle, which effects depend on cache size, and that the two cycle-1 variants lost to HTTP 429 build errors say nothing about their ideas.


---

- finished: 2026-11-05T22:55:17+00:00
- duration: 23.93s
- status: ok
- input_tokens: 4
- output_tokens: 2153
- cache_read_tokens: 38707
- cache_write_tokens: 10316
- cost_usd: 0.13393

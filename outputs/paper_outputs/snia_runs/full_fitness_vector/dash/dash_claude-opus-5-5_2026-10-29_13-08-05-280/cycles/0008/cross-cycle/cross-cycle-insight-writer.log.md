# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-29T14:20:44+00:00

## timeline

### 14:20:51 thinking



### 14:21:08 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Admission guards on \"who enters\" a bucket are the most dependable hit-rate lever.** This holds across cycles 1, 3, 4 and 5. Examples: the two-hit stash gate, the extra hit needed to enter a full target, and the two hottest hits needed to enter a full probing bucket. Each one asks for one more piece of reuse evidence before a full bucket is entered, which cuts the cascades that push reused residents out.\n- **Removing work beats deferring it.** Fewer multi-bucket moves (promotions plus demotion cascades, stash churn) improve hit-rate and latency together. When a change dropped the moves outright, the tail got faster (cycles 3, 5). When it only postponed them, the tail did not improve (cycle 4).\n- **The displaced probing resident is disposable.** Evicting it outright, instead of re-homing it by cascade or swap, cost no hit-rate on two lineages (cycles 7, 8) and slightly helped large-cache hm_0. It also frees a slot, which avoids the occupancy cost of a swap.\n- **Using a recency window as a throttle** on how often the costly swap fires lowered the tail (cycle 7). It is the only lazy check that actually reduced cost rather than moving it.\n- **Shrinking Item layout worked once the real blocker was removed.** `timestamp: Instant` was the field holding size up. Replacing it with a `u32` tick, plus a `u8` counter, gives a 24-byte Item. The tick has to be per-segment: a global atomic caused contention regressions (cycles 4, 5).\n- **Large cache sizes, and hm_0 in particular, are where reuse-aware changes win or lose most of their hit-rate.**\n\n## Recurring failures\n\n- **Any bookkeeping on the common target-hit path raises median get.** This includes counter writes, compares, and resets. It has shown up in at least four cycles (1, 2, 5, 8), even when the logic almost never fires.\n- **Stacking promotion levers on the same arm doesn't add up.** Pairs such as swap with gate removal, or window with cascade, address the same cost. Once one is in place, the other adds nothing (cycle 7).\n- **\"Faster comeback\" mechanisms lose hit-rate.** Re-admission credits, MRU demotion landing and wider probe windows all raise churn or cost without gaining hits (cycles 2, 3, 8).\n- **Placement based only on capacity does nothing for hit-rate** (cycles 1, 2). This covers free-slot balanced placement and its flag-limited form.\n- **Swaps that never free a slot keep buckets saturated** and push more operations onto slow full-bucket paths. The tail cost often outweighs the hit-rate gain (cycle 6).\n- **Protecting stash entries with no aging** (preferring to evict never-hit entries) starves fresh arrivals and loses hit-rate everywhere, small caches most (cycle 8). The stash's LRU-position exit is already the right choice.\n- **Size-gated thresholds have little headroom left.** The stash size gate is redundant on the swap/eviction lineage, though it was still holding up small-cache hm_0 under the cascade.\n- **Latency-only \"Winners\" can't be trusted.** Cycles 3 and 5 through 8 each showed uniform shifts that touched paths the diff never changed, including put. An identical-baseline noise re-run has been requested for seven cycles and has never been done.\n\n## Open questions\n\n- **How large is per-op-time noise?** This blocks everything else. Run the parent, an identical copy of it, and recent candidates together on one host in one session, as a hypothesis of its own.\n- **How often do the guarded full-probing and full-target arms actually fire?** And which paths dominate get and put p99 now: stash→target moves or put eviction?\n- **What limits hit-rate in hm_0-small and web_0-large on the current lineage?** Throttle tuning and stash-exit protection have been ruled out. The remaining candidates are the arm's mechanics and admission.\n- **Can a non-hottest-hit staleness signal be captured without touching the common hit path?**\n- **Would an aging bound make stash-exit protection viable,** or is that lever dead?\n- **Does narrowing the window on the eviction lineage cut latency** without costing hit-rate? Would a window sized to the cache help small caches?\n\n## Per-policy notes\n\n- **ClassicLRU (the only evaluated policy):**\n  - It never reads the timestamp, so changing the tick or layout cannot change its hit-rate. Only the cost of maintaining the field matters.\n  - Removing the tick entirely may be an option.\n- **Dash segment tiers (target / probing / stash):**\n  - The current baseline lineage is the per-segment tick with the 24-byte Item, the full-target door guard, the full-probing two-hit guard, a window throttle, and a single outright eviction of the probing resident.\n  - Guards help small caches only at tiers where candidates stay long enough to collect hits. Target qualifies; stash does not, unless the cache is large.\n"
}
```

### 14:21:15 chat

I rebuilt `cross-cycle-insights.md` from scratch using only the ledger for cycles 0001–0008, with the four sections in the required order.

**Biggest open issue:** we still don't know how noisy the per-op-time measurement is. An identical-baseline re-run has been asked for in seven straight cycles and never done. Until it is, any variant that wins only on latency can't be trusted, so the file says this re-run should be a hypothesis of its own next cycle.

- **What keeps working:**
  - Requiring one more reuse hit before an item enters a full bucket. This is the most reliable way to raise hit-rate.
  - Cutting multi-bucket moves outright rather than delaying them.
  - Simply evicting the displaced probing resident, which cost no hit-rate on two lineages.
  - A recency window used to limit how often swaps fire.
  - The 24-byte Item with a per-segment tick.
- **What keeps failing:**
  - Any extra check on the common hit path, which raises median get.
  - Stacking two promotion levers on the same arm; they overlap instead of adding up.
  - Mechanisms that make it faster for evicted items to come back.
  - Placement based only on free space.
  - Swaps that never free a slot.
  - Protecting stash entries with no aging.
- **Open questions:**
  - How often the guarded arms actually fire.
  - Which paths now cause the get and put tails.
  - What still limits hit-rate on hm_0 at small sizes and web_0 at large sizes.
- **Per-policy notes:**
  - ClassicLRU is the only policy evaluated, and it never reads the timestamp. So layout and tick changes can't move its hit-rate; only their upkeep cost matters.
  - The tier-level notes also record the current baseline lineage.

All claims are qualitative; I made up no numbers.


---

- finished: 2026-10-29T14:21:16+00:00
- duration: 31.94s
- status: ok
- input_tokens: 4
- output_tokens: 2542
- cache_read_tokens: 42584
- cache_write_tokens: 14575
- cost_usd: 0.175973

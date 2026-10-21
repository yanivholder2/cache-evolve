# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:56:52+00:00

## timeline

### 04:57:16 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0007)\n\n## Recurring wins\n\n- **Filter, don't protect.** Every winner has made it harder to get into scarce space (0001: stash-hit admission; 0004: BIP fresh-key insertion; 0006: rarer BIP; 0007: recency-gated ghost). No winner has added protection on top of the existing mechanisms.\n- **Protected stash capacity is zero-sum.** From 0005 to 0007 the rule held four cycles running. Any slot taken from pre-credited or evidence-backed ex-residents costs at least as much as it gives back to fresh keys.\n- **SLRU/2Q split.** Fresh keys face a harsh bar, while proven ex-residents return cheaply and are protected. This split came through in 0002 (MRU stash re-entry), 0004 (LRU-end probation) and 0005–0007 (ghost-guided MRU).\n- **Placement beats counters.** Changing insert position or residency moves hit-rate. Changing promotion counters (pre-credit, frequency duels, ghost credit) does not (0004 null, 0005 win).\n- **Recent evidence beats more evidence.** A ghost horizon of about one cache works (0005). A larger window lost (0006), and gating to about half a window won (0007).\n- **Overlapping paths: the selective one wins.** Once the ghost existed, BIP's blind MRU path became a net cost (0006, 0007).\n\n## Recurring failures\n\n- **Loosening admission or adding capacity to probation.** MRU landing in normal buckets (0001), wider probation (0005) and set-dueling DIP (0005) all weakened the protected region and lost or broke even.\n- **Stacked retention mechanisms that protect the same items.** These were tried as the CLOCK scan on top of MRU (0003), scan-skip for ghost hits (0006) and threshold 3 on top of MRU re-entry (0003). They add nothing because position and frequency protection overlap.\n- **Counter-only tweaks.** Ghost pre-credit (0004), the TinyLFU duel against the LRU-end resident (0004) and target→probing frequency gating (0002) did nothing. Front landing already couples frequency to position.\n- **Free-slot placement** (0001) rarely fires because the normal buckets are almost always full.\n- **Cascade-free routing** (0003, 0007) left hit-rate flat, and in 0007 the result was bit-identical. Either it never fires at this size or it never applied. It can't be scored without a per-op-time dimension.\n- **Long ghost windows.** Stale far-reuse hits displace residents with shorter reuse. Misses with reuse distance beyond about one cache can't be recovered through the stash.\n\n## Open questions\n\n- Do gated ghost (0007 v002) and BIP removal (0007 v001) stack? Both gained on their own, and they touch different paths.\n- Where is the best ghost recency cutoff? Can the ghost table shrink to the gated horizon and save metadata?\n- **Measurement gap (raised every cycle, never fixed):** per-op-time and metadata-size are unmeasured for every winner. Ghost, flag and epoch metadata keep growing.\n- **Generalisation gap:** every result comes from one LIRS trace at one cache size. It is unknown whether BIP removal, the ghost horizon and the saturation findings carry over.\n- **Overlay verification:** writers have never been able to inspect the overlay diff. Exact-parent matches (0003 v001, 0007 v003) may be changes that never applied. A displacement/cascade counter would help settle this.\n- Structural capacity levers, such as the stash-to-normal slot ratio and stash size, are still untested.\n\n## Per-policy notes\n\n- **Normal tier (target/probing, Front landing):** admission matters and placement inside a bucket does not. The target→probing edge is already filtered by Front landing. Neighbour cascade damage is mostly absorbed now that the stash recovers demoted items cheaply.\n- **Stash admission threshold:** 2 beats 1, and 3 is near the optimum or redundant once MRU re-entry exists. Don't push it higher.\n- **Stash victim choice:** the second-chance scan gave only a noise-level gain. The lever is mostly exhausted.\n- **Stash insertion (BIP):** LRU-end insertion of fresh keys was a real lever. Its blind MRU path has been tuned toward \"off\" now that the ghost exists.\n- **Ghost table:** it pays only when a ghost hit changes placement (stash MRU), not the counter, and only for recent evictions (roughly half a cache to one cache). Scan protection for ghost hits is redundant.\n- **Overall:** hit-rate gains on this trace have shrunk with each cycle. The stash path is close to tuned out, so the next real signal probably needs new traces or sizes, or new fitness dimensions.\n"
}
```

### 04:57:23 thinking



### 04:57:23 chat

I've written `cross-cycle-insights.md` with the four required sections. It's built only from the ledger for cycles 0001–0007, with no numbers added.

- **What keeps winning:** making entry into scarce cache space stricter. No winner has added protection. Space reserved for evicted items that are likely to come back is zero-sum: giving any of it to fresh keys costs at least as much as it gains. Changing where a key is placed matters more than adjusting hit counters, and recent ghost hits are better evidence than older ones.
- **What keeps failing:**
  - loosening admission or widening the probation area;
  - stacking two retention mechanisms that protect the same items;
  - changes that only adjust counters;
  - placing items into free slots, which are almost never free;
  - large ghost windows;
  - cascade-free routing, which can't be scored without a per-op-time measurement.
- **Still open:**
  - whether 0007's two gainers stack (the recency-gated ghost and removing BIP);
  - the best recency cutoff for the ghost, and whether its table can shrink;
  - per-op-time and metadata-size, which have never been measured for any winner;
  - every result comes from one LIRS trace at one cache size;
  - nobody has checked whether the changes were actually applied;
  - stash size, and how slots are split between the stash and the normal tier, are still untested.
- **Per-policy notes** cover the normal tier, the stash admission threshold, stash victim choice, BIP insertion and the ghost table. They end on the main point: the stash path looks nearly tuned out on this trace.


---

- finished: 2026-10-21T04:57:23+00:00
- duration: 30.98s
- status: ok
- input_tokens: 4
- output_tokens: 2311
- cache_read_tokens: 39518
- cache_write_tokens: 12622
- cost_usd: 0.15592

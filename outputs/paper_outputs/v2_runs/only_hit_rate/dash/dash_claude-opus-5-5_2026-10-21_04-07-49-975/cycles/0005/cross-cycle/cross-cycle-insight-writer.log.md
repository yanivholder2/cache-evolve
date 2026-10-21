# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:43:05+00:00

## timeline

### 04:43:21 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Asymmetric SLRU/2Q-style split: fresh keys face a harsh bar, proven ex-residents get a cheap path back and are protected. Every winner since 0001 is a version of this: stash-hit admission filter (0001), MRU stash re-entry for demoted items (0002), BIP fresh-key insertion (0004), ghost-guided MRU placement (0005).\n- Placement/position beats counter credit. Insert position of fresh stash keys (0004) and ghost hits that change placement (0005) moved hit-rate. Counter-only pre-credit and frequency tweaks did not.\n- The stash works best as a two-part structure: a narrow probation region that one-touch keys churn through, and a protected victim cache for pre-credited ex-residents. Leave the protected region intact and admit fresh keys into it only when there is evidence they return, e.g. a ghost hit within a window about one cache in size.\n- Stricter admission at the stash→target edge helped early on (thresholds 2 then 3). Returns flattened by 0002–0003.\n\n## Recurring failures\n\n- Anything that takes protected capacity away from ex-residents: wider probation (0005), set-dueling DIP restoring MRU insertion (0005), and MRU landing in the normal tier that lowers the bar for probing (0001). Stash capacity is close to zero-sum on this trace.\n- Frequency gating that reads what position already encodes: a target→probing hit bar (0002), and a TinyLFU duel against an LRU-end resident that is almost always counter-0 because of Front landing (0004).\n- Counter-only pre-credit for keys that never hit while resident, e.g. short per-bucket ghosts that credit the counter (0004).\n- Mechanisms that only take effect when a slot is free (0001 spill), since normal buckets are almost always full together.\n- Stacking retention mechanisms that protect the same items: a stash victim scan, threshold 3 on top of MRU re-entry, or cascade rerouting all came out null (0003).\n- Process failure: every cycle so far was scored on hit-rate only, on one LIRS trace at one cache size. Each continuation asked for more coverage and none was done. Overlay application was never verified (0003 v001 matched its parent exactly).\n\n## Open questions\n\n- Per-op-time and metadata-size for the winner chain, which are now urgent because the ghost table adds metadata. A related question is whether cascade-free routing (0003 v003) wins on per-op-time.\n- Do BIP and the ghost gains generalise to other traces and cache sizes, or do they overfit LIRS loops?\n- Ghost tuning: a larger window, or a ghost hit that also protects the key from the next scan pass. Also tuning `BIP_PERIOD`, the only fresh-key survival knob that takes no protected capacity.\n- Structural capacity levers: the stash-to-normal slot ratio and stash size.\n- Are the remaining misses mostly true one-touch keys or keys with reuse distances beyond one cache's worth of evictions? This sets the ceiling for further hit-rate work.\n\n## Per-policy notes\n\n- Normal tier (target/probing, Front landing): position and frequency are coupled, and the \"is hottest\" check reads position. Every eviction there sets off a cross-bucket demotion cascade. Since 0002, cheap stash recovery has made that cascade harmless for hit-rate.\n- Stash: larger than the normal tier, and the stash design is where most of the hit-rate gains came from. Current lineage: two-hit admission filter, then MRU re-entry with pre-credit, then a second-chance scan (a noise-level winner), then BIP fresh insertion, then a ghost table about one cache in size with MRU placement on a ghost hit. Victim-choice tweaks were saturated by 0003.\n- Target→probing edge: little left to gain through admission. Drop it.\n- Adaptive insertion (DIP): it lost, but its miss signal was noisy and its leader groups were small, so this is weak evidence against it rather than proof.\n"
}
```

### 04:43:29 chat

I've written `cross-cycle-insights.md` in the cycle 0005 cross-cycle directory, using the four fixed sections in order. It is built only from the ledger digest you supplied.

- **Recurring wins:** Every winner from cycle 0001 to 0005 does the same thing. New keys face a harsh bar to get in, and items that were in the cache before get a cheap way back and are protected. Changing where an item is placed has worked; changing only its hit counter has not.
- **Recurring failures:**
  - Changes that took protected stash space away from returning items lost or broke even, e.g. a wider probation area and adaptive (DIP) insertion.
  - Frequency checks that only repeat what an item's position already says did nothing.
  - So did changes that only matter when a slot is free.
  - It also records a process failure: every cycle was scored on hit-rate alone, on one LIRS trace at one cache size. No cycle followed the earlier calls to add per-op-time, metadata-size or another trace.
- **Open questions:**
  - Per-op-time and metadata-size are still unmeasured. That is now urgent because the cycle 0005 winner adds a ghost table.
  - It is not known whether the gains hold on other traces and cache sizes.
  - Still to probe: the ghost table's window size and the `BIP_PERIOD` setting, and the split of slots between the stash and the normal tier.
- **Per-policy notes:** The notes are grouped by part of the design: the normal tier, the stash and how it evolved over the cycles, the target→probing step (marked as worth dropping), and DIP. Since cycle 0002 the stash's cheap recovery has made the neighbour-bucket cascade harmless for hit-rate.


---

- finished: 2026-10-21T04:43:30+00:00
- duration: 24.77s
- status: ok
- input_tokens: 4
- output_tokens: 2053
- cache_read_tokens: 37508
- cache_write_tokens: 10320
- cost_usd: 0.13174

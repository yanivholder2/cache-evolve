# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:35:38+00:00

## timeline

### 04:35:56 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0004)\n\n## Recurring wins\n\n- The SLRU/2Q split has won three times by different means (0001 admission threshold, 0002 MRU stash re-entry, 0004 BIP insertion). Fresh keys face a harsh bar, and proven ex-residents get protected. Each cycle's winner made this split sharper.\n- Admission to the scarce normal tier sets hit-rate more than placement inside it does. A stash-hit filter at the stash→target edge is cheap because the stash is big enough to hold unproven keys while they wait.\n- Stash levers that act on *where an item sits* paid off when they told fresh keys apart from ex-residents: MRU re-entry for demoted items, LRU-end (bimodal) insertion for fresh keys. Victim scans and counters only pay off once positions already separate these two groups.\n- The lineage so far is 0001 v003 → 0002 v003 → 0003 v002 → 0004 v003, and each step built on the one before. Gains have gotten smaller, but stacking structural position changes has not regressed.\n\n## Recurring failures\n\n- **Counter/threshold tweaks after the first one:** pushing the stash threshold to 3 helped a little once (0002) and did nothing when stacked on MRU re-entry (0003). Ghost pre-credit (0004) only lowers a bar that keys with long reuse distances never reach. Changing counters does nothing for keys that never get a hit while resident.\n- **Frequency gating where position already encodes frequency:** both the target→probing frequency bar (0002) and the TinyLFU duel at stash→target (0004) were flat. Front landing makes the normal-tier eviction candidate almost always a counter-0 fresh arrival, so a frequency check reads what position already tells it.\n- **Free-slot or alternate placement in the normal tier** (0001 spill to probing, 0001 MRU landing): normal buckets are almost always full together. MRU landing also lowers the bar for moving to probing, because the \"is hottest\" check reads position.\n- **Overlapping retention mechanisms:** a CLOCK-like scan, frequency protection and MRU placement all protect the same ex-residents. Stacking them adds nothing (0003).\n- **Cascade rerouting for hit-rate** (0003): once the stash recovers demoted items cheaply, it just swaps one recoverable victim for another.\n- **Process failure, repeated every cycle:** cycles 0001–0004 all ran hit-rate only on one LIRS trace at one cache size. Each continuation asked for per-op-time and metadata-size measurements and more traces, and none of them happened.\n\n## Open questions\n\n- Per-op-time and metadata-size have never been measured for any winner. The best per-op-time candidate is 0003 v003 (cascade-free routing, no hit-rate cost). v003's claimed get-latency gain from 0001 was never checked either.\n- Does any gain hold outside this LIRS trace and cache size? BIP insertion in particular may be overfitting LIRS loop patterns.\n- BIP tuning: the insertion period, and whether a probation area wider than one slot helps keys with moderate reuse distance.\n- Could ghosts work if the window covers roughly cache-size evictions and a ghost hit changes placement or residency, not the counter?\n- Are the stash capacity and the stash-to-normal slot ratio untapped structural levers?\n- Was 0003 v001's exact match with its parent a real null, or did its overlay never get applied? Build verification has not been confirmed.\n\n## Per-policy notes\n\n- **Stash→target edge:** a threshold of 2 is the main win, and 3 is about the optimum (no gain once stacked with MRU re-entry). Don't push the threshold further, and don't add duels against the LRU-end resident.\n- **Target→probing edge:** Front landing already filters it. Frequency gating does nothing, and churn from this edge seems small.\n- **Stash retention/eviction:** MRU re-entry for demoted ex-residents and LRU-end BIP insertion for fresh keys are the effective controls. The second-chance (counter-0-preferring) scan is neutral by itself and only became useful once fresh keys were placed at the LRU end.\n- **Normal tier:** the demotion cascade into neighbour buckets mattered in 0001. Since MRU stash re-entry (0002) it barely matters for hit-rate. Its remaining value is probably per-op-time.\n- **Ghost/non-resident history:** the per-bucket window was too short, and a counter credit was the wrong action. If revisited, change both.\n"
}
```

### 04:36:02 chat

I've written `cross-cycle-insights.md` from the ledger digest for cycles 0001–0004, using the four required sections in order.

- **Biggest finding:** every cycle so far was scored on hit-rate only, on one LIRS trace at one cache size. Each cycle's continuation asked for per-op-time and metadata-size measurements and a second trace or size, and none of them ran. The file lists this under recurring failures.
- **Recurring wins:** the pattern that keeps winning is a split where fresh keys face a harsh bar and proven ex-residents get protected. It won three times, by different means each time:
  - a stricter admission threshold (0001);
  - putting demoted items back at the MRU end of the stash (0002);
  - inserting fresh keys at the LRU end of the stash (0004).
- **Recurring failures:**
  - further counter or threshold tweaks after the first one;
  - frequency gating where bucket position already tells you the same thing;
  - placement into free slots in the normal tier;
  - stacking retention mechanisms that protect the same items;
  - rerouting the eviction cascade to gain hit-rate.
- **Open questions:**
  - per-op-time and metadata-size, starting with the cascade-free routing variant from 0003;
  - whether the latest winner overfits the LIRS trace, and how to tune it;
  - ghost history with a larger window, where a ghost hit changes placement rather than a counter;
  - stash capacity and the split of slots between stash and normal tier;
  - whether 0003 v001's exact match with its parent means its change never reached the evaluated build.
- **Per-policy notes:** grouped by stash→target edge, target→probing edge, stash retention, normal tier, and ghost history.


---

- finished: 2026-10-21T04:36:03+00:00
- duration: 24.65s
- status: ok
- input_tokens: 4
- output_tokens: 2241
- cache_read_tokens: 36439
- cache_write_tokens: 9434
- cost_usd: 0.128089

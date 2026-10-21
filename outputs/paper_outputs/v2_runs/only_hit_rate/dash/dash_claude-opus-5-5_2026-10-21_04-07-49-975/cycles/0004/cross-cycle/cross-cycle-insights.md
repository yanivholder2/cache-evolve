# Cross-Cycle Insights (through cycle 0004)

## Recurring wins

- The SLRU/2Q split has won three times by different means (0001 admission threshold, 0002 MRU stash re-entry, 0004 BIP insertion). Fresh keys face a harsh bar, and proven ex-residents get protected. Each cycle's winner made this split sharper.
- Admission to the scarce normal tier sets hit-rate more than placement inside it does. A stash-hit filter at the stash→target edge is cheap because the stash is big enough to hold unproven keys while they wait.
- Stash levers that act on *where an item sits* paid off when they told fresh keys apart from ex-residents: MRU re-entry for demoted items, LRU-end (bimodal) insertion for fresh keys. Victim scans and counters only pay off once positions already separate these two groups.
- The lineage so far is 0001 v003 → 0002 v003 → 0003 v002 → 0004 v003, and each step built on the one before. Gains have gotten smaller, but stacking structural position changes has not regressed.

## Recurring failures

- **Counter/threshold tweaks after the first one:** pushing the stash threshold to 3 helped a little once (0002) and did nothing when stacked on MRU re-entry (0003). Ghost pre-credit (0004) only lowers a bar that keys with long reuse distances never reach. Changing counters does nothing for keys that never get a hit while resident.
- **Frequency gating where position already encodes frequency:** both the target→probing frequency bar (0002) and the TinyLFU duel at stash→target (0004) were flat. Front landing makes the normal-tier eviction candidate almost always a counter-0 fresh arrival, so a frequency check reads what position already tells it.
- **Free-slot or alternate placement in the normal tier** (0001 spill to probing, 0001 MRU landing): normal buckets are almost always full together. MRU landing also lowers the bar for moving to probing, because the "is hottest" check reads position.
- **Overlapping retention mechanisms:** a CLOCK-like scan, frequency protection and MRU placement all protect the same ex-residents. Stacking them adds nothing (0003).
- **Cascade rerouting for hit-rate** (0003): once the stash recovers demoted items cheaply, it just swaps one recoverable victim for another.
- **Process failure, repeated every cycle:** cycles 0001–0004 all ran hit-rate only on one LIRS trace at one cache size. Each continuation asked for per-op-time and metadata-size measurements and more traces, and none of them happened.

## Open questions

- Per-op-time and metadata-size have never been measured for any winner. The best per-op-time candidate is 0003 v003 (cascade-free routing, no hit-rate cost). v003's claimed get-latency gain from 0001 was never checked either.
- Does any gain hold outside this LIRS trace and cache size? BIP insertion in particular may be overfitting LIRS loop patterns.
- BIP tuning: the insertion period, and whether a probation area wider than one slot helps keys with moderate reuse distance.
- Could ghosts work if the window covers roughly cache-size evictions and a ghost hit changes placement or residency, not the counter?
- Are the stash capacity and the stash-to-normal slot ratio untapped structural levers?
- Was 0003 v001's exact match with its parent a real null, or did its overlay never get applied? Build verification has not been confirmed.

## Per-policy notes

- **Stash→target edge:** a threshold of 2 is the main win, and 3 is about the optimum (no gain once stacked with MRU re-entry). Don't push the threshold further, and don't add duels against the LRU-end resident.
- **Target→probing edge:** Front landing already filters it. Frequency gating does nothing, and churn from this edge seems small.
- **Stash retention/eviction:** MRU re-entry for demoted ex-residents and LRU-end BIP insertion for fresh keys are the effective controls. The second-chance (counter-0-preferring) scan is neutral by itself and only became useful once fresh keys were placed at the LRU end.
- **Normal tier:** the demotion cascade into neighbour buckets mattered in 0001. Since MRU stash re-entry (0002) it barely matters for hit-rate. Its remaining value is probably per-op-time.
- **Ghost/non-resident history:** the per-bucket window was too short, and a counter credit was the wrong action. If revisited, change both.

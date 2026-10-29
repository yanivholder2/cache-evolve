# Cross-Cycle Insights (through cycle 0006)

## Recurring wins

- **Don't put a newly proven item in the victim slot.** This placement rule paid off in four cycles: MRU-end stash demotion (0001), `Back` promotion into the target (0003), and `Back` placement into probing (0005, 0006). Changing placement has helped more reliably than tuning admission thresholds.
- **Reuse evidence kept outside the item beats counters inside it.** The FIFO ghost ring of stash-evicted keys (0005) was the first lever to move `web_0` 128K. It decays by overwrite and is consumed when the key is re-put. Cycle 0006 found its lifecycle, factor 2 with one-shot credit, close to optimal.
- **Levers that act on separate tiers or separate traces tend to stack.** Ghost plus probing placement added up on `hm_0` (0006). The two-hit gate plus MRU demotion was a net win (0002). Levers that act on the same small stash population interfere with each other instead.
- **Plain LRU is the right stash eviction rule.** Changing the victim choice by counter lost hit-rate in every cell (0004).
- **A stricter room condition for bypassing the gate** (two free slots) kept the `web_0` 128K gain without the 16K cascade cost (0004). The lever is now mostly used up.

## Recurring failures

- **Easier routes back into a crowded target.** This has lost three times: one-hit return for demoted items (0003), the last-free-slot shortcut (0003), and ghost credit on demotion (0006). When space is tight, an item that already lost its slot should not outrank the target's current residents, because each extra crossing sets off an eviction cascade.
- **Changing the target→probing trigger in either direction.** Delaying it (0004) and speeding it up (0005) both lost. Treat the trigger as settled.
- **In-item frequency counters.** When they reset on moves they stay near zero and do nothing (0002). When they never decay they turn into permanent pinning (0004). Any new frequency signal needs an explicit decay rule.
- **Hard LRU-end insertion of fresh puts** (0002) starves new keys of their probation hit.
- **More ghost reach.** A four-times-longer ring (0006) credited keys that return too late to be worth caching, and it hurt most on the cell it was aimed at.
- **Cost is still unmeasured after six cycles.** Per-op-time and metadata-size have never been reported, so every cost claim in the lineage is unverified. That includes the lookup reorder (0001), the ghost ring's metadata, and the extra scans.

## Open questions

- What limits `web_0` 16K? It has stayed flat through the gate, the ghost, longer ghost reach and probing placement. Remaining candidates are stash size relative to the segment and demotion placement for keys with no ghost history.
- Can the small `web_0` 128K giveback from probing `Back` placement be recovered by softening the foreign-resident cascade into the stash, without losing the `hm_0` gain?
- Would a shorter ghost ring (factor 1) cut stale credit, or is factor 2 already the floor?
- Is there a trace-adaptive or occupancy-conditioned control that serves `hm_0` (which rewards protecting residents) and `web_0` (which rewards fast admission) at the same time? Most timing levers have pulled these two traces in opposite directions.
- Does the target-first lookup reorder actually lower per-op-time? It leaves hit-rate unchanged, but its latency has not been measured.

## Per-policy notes

- **Stash (bucket-local LRU):** Use MRU-end demotion, LRU eviction, and a two-hit exit gate plus ghost-backed credit for returning keys. This tier is small and contested, so levers here interact rather than stack.
- **Target (normal) buckets:** Promotees go to the MRU end. Bypass the gate only when there is real room (two free slots). Never let one-hit or demoted items take the last slot in a small cache.
- **Probing tier:** The trigger is at its optimum. `Back` placement helps `hm_0` (whose hot set reaches probing) but pushes cascades onto foreign residents. Those cascades lead to stash churn on `web_0` 128K.
- **Ghost ring:** FIFO, factor 2, consumed on put, records stash evictions only. Treat it as settled. Whether demotions from normal buckets should check or record into the ghost is still untested.
- **Trace/size regularities:** `hm_0` rewards protecting residents and correct placement. `web_0` 128K is an admission problem for returning keys. `web_0` 16K resists every lever tried so far. Small caches punish any added route into the target. Large caches punish extra admission filters.

# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-11-05T22:46:45+00:00

## timeline

### 22:47:00 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Two-hit stash gate (cycles 1–4).** Requiring a second stash hit before promotion raised hit rate in every cycle where it was measured. Every later line builds on it. It does not cause the `get` slowdown: ungated variants land in the same latency band.\n- **MRU demotion to stash (cycles 2–4).** Demoted normal-tier victims get a full stash residency, and hit rate goes up with or without the gate. This is the strongest single retention lever found so far.\n- **\"Don't evict a proven resident\" (cycles 2–4).** Promotions that use free probing space instead of evicting a target resident add hit rate. This holds for probing overflow and for the probing room gate.\n- **Fewer cascades while keeping stash draining (cycle 3, confirmed by cycle 4's failures).** Probing overflow on the gate + MRU base reversed most of the `put` p99 regression. Current base: gate + MRU demotion + probing overflow.\n- **Reusing existing per-item fields (e.g. `lfu_counter`)** adds policy state without growing metadata.\n- **hm_0 at the large size** responds most to retention changes, in both directions. Use it as the sensitivity probe.\n\n## Recurring failures\n\n- **No Winner after four cycles.** Hit-rate gains keep coming, but some latency metric (`get` p50 and/or a p99) stays worse than baseline every time.\n- **Lookup reordering (normal tier before stash).** No hit-rate change and worse `get`, because stash hits are too large a share of all hits.\n- **Removing first-hit reordering.** Barely helps p50, leaves the tail unchanged, and loses hit rate. The latency cost is not on the first-hit path.\n- **Skipping cascades without an exit from stash.** Unbounded deferral traps items in stash and makes `put` p99 much worse. Skipping cascades elsewhere just moves the cost: to `put` through fuller target buckets, or to a later `get` through foreign probing residents.\n- **Ungated MRU/overflow line.** Does not recover baseline latency. Items placed in probing are found last, and that costs more `get` p99 than overflow saves on `put`. It is not a cheaper route.\n- **Cascade-focused changes never move `get` p50.**\n- **Infrastructure:** builds have failed on Maven/Gradle 429 rate limits (cycle 1). Ideas lost that way were untested, not refuted.\n\n## Open questions\n\n- Where on the common hit path is the `get` p50 gap: the stash check, the scan of a full target bucket, or the move-to-MRU touch?\n- Can a bounded deferral (demote or force-promote after a limit) keep stash drained without bringing the full cascade back?\n- Can the stash-promotion fallback be made cheaper, so the probing room gate keeps its hit-rate gain without the `put` cost?\n- Can the lookup penalty for items placed in probing be cut, for example by moving them back to target once it has space?\n- How much of the remaining small p99 movement is real, and how much is benchmark noise? This has never been separated.\n- H03-style balanced promotion between target and probing has never been tested on its own.\n\n## Per-policy notes\n\n- **Dash (only policy explored so far):**\n  - Lineage: two-hit gate → plus MRU demotion → plus probing overflow (current base). The probing room gate is a candidate add-on if its `put` cost can be fixed.\n  - Tail latency tracks stash fill level and where cascade cost lands, not the raw cascade count.\n  - The `get` slowdown tracks stash-hit traffic (the remove/promote/demote chain), not the gate itself.\n  - Hit-rate gains are small but consistent across traces and sizes. The latency gap to baseline is now mostly `get` p50 on the common hit path.\n"
}
```

### 22:47:06 thinking



### 22:47:06 chat

I wrote `cross-cycle-insights.md` from the four-cycle ledger, using the four required sections in order.

The main finding is that **after four cycles there is still no Winner.** Hit rate goes up every cycle, but every variant is still slower than the baseline somewhere. The remaining gap is mostly median `get` time on the common hit path, not the cascade paths.

- **What keeps working:**
  - Requiring two stash hits before promotion.
  - Sending demoted items to the MRU end of stash. This is the strongest hit-rate lever so far.
  - Promoting into free probing space instead of evicting a resident.
  - The current base combines all three, and that's the line to build on.
- **What keeps failing:**
  - Changing lookup order.
  - Removing the reorder on the first stash hit.
  - Skipping cascades without giving items another way out of stash. This either trapped items in stash or just moved the cost somewhere else.
  - The ungated branch.
  - Cycle 1's two lost variants failed on build rate limits, so they say nothing about the ideas.
- **Open questions:**
  - Where exactly the median `get` time goes.
  - Whether a capped deferral or a cheaper promotion fallback could help.
  - How to cut the lookup cost for items placed in probing.
  - Whether the small tail-latency changes are real or benchmark noise.
  - The balanced-promotion idea from cycle 1, which has never been tested.

Dash is the only policy explored so far, so the per-policy section covers just that one.


---

- finished: 2026-11-05T22:47:07+00:00
- duration: 21.55s
- status: ok
- input_tokens: 4
- output_tokens: 1927
- cache_read_tokens: 37797
- cache_write_tokens: 9246
- cost_usd: 0.120577

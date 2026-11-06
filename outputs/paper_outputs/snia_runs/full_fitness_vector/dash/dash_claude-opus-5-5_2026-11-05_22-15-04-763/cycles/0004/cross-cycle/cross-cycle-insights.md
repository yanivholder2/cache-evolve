## Recurring wins

- **Two-hit stash gate (cycles 1–4).** Requiring a second stash hit before promotion raised hit rate in every cycle where it was measured. Every later line builds on it. It does not cause the `get` slowdown: ungated variants land in the same latency band.
- **MRU demotion to stash (cycles 2–4).** Demoted normal-tier victims get a full stash residency, and hit rate goes up with or without the gate. This is the strongest single retention lever found so far.
- **"Don't evict a proven resident" (cycles 2–4).** Promotions that use free probing space instead of evicting a target resident add hit rate. This holds for probing overflow and for the probing room gate.
- **Fewer cascades while keeping stash draining (cycle 3, confirmed by cycle 4's failures).** Probing overflow on the gate + MRU base reversed most of the `put` p99 regression. Current base: gate + MRU demotion + probing overflow.
- **Reusing existing per-item fields (e.g. `lfu_counter`)** adds policy state without growing metadata.
- **hm_0 at the large size** responds most to retention changes, in both directions. Use it as the sensitivity probe.

## Recurring failures

- **No Winner after four cycles.** Hit-rate gains keep coming, but some latency metric (`get` p50 and/or a p99) stays worse than baseline every time.
- **Lookup reordering (normal tier before stash).** No hit-rate change and worse `get`, because stash hits are too large a share of all hits.
- **Removing first-hit reordering.** Barely helps p50, leaves the tail unchanged, and loses hit rate. The latency cost is not on the first-hit path.
- **Skipping cascades without an exit from stash.** Unbounded deferral traps items in stash and makes `put` p99 much worse. Skipping cascades elsewhere just moves the cost: to `put` through fuller target buckets, or to a later `get` through foreign probing residents.
- **Ungated MRU/overflow line.** Does not recover baseline latency. Items placed in probing are found last, and that costs more `get` p99 than overflow saves on `put`. It is not a cheaper route.
- **Cascade-focused changes never move `get` p50.**
- **Infrastructure:** builds have failed on Maven/Gradle 429 rate limits (cycle 1). Ideas lost that way were untested, not refuted.

## Open questions

- Where on the common hit path is the `get` p50 gap: the stash check, the scan of a full target bucket, or the move-to-MRU touch?
- Can a bounded deferral (demote or force-promote after a limit) keep stash drained without bringing the full cascade back?
- Can the stash-promotion fallback be made cheaper, so the probing room gate keeps its hit-rate gain without the `put` cost?
- Can the lookup penalty for items placed in probing be cut, for example by moving them back to target once it has space?
- How much of the remaining small p99 movement is real, and how much is benchmark noise? This has never been separated.
- H03-style balanced promotion between target and probing has never been tested on its own.

## Per-policy notes

- **Dash (only policy explored so far):**
  - Lineage: two-hit gate → plus MRU demotion → plus probing overflow (current base). The probing room gate is a candidate add-on if its `put` cost can be fixed.
  - Tail latency tracks stash fill level and where cascade cost lands, not the raw cascade count.
  - The `get` slowdown tracks stash-hit traffic (the remove/promote/demote chain), not the gate itself.
  - Hit-rate gains are small but consistent across traces and sizes. The latency gap to baseline is now mostly `get` p50 on the common hit path.

## Recurring wins

- Placement held up three times (c1 stash demotion to MRU, c3 promotion to target `Back`, c5/c6 target→probing `Back`): an item that just proved itself should not land in the victim slot. The gains were largest where slots are contested.
- History that outlives eviction worked where in-item counters failed. The global FIFO ghost ring (factor 2, consumed on put) was the first lever to move `web_0` 128K. Tests of longer, shorter, double-spend and partitioned versions in c6, c7 and c9 all confirmed the current design.
- Levers that act on separate tiers or traces mostly added up when combined (gate + MRU demotion, ghost + probing placement). Levers that act on the same small stash population interfered with each other instead.
- Moves that only use free slots and displace no one never regressed (c4 strict bypass threshold, c7 rescue). Their value is limited to the large size, though, and this direction is now used up.
- The lineage moves forward mostly by avoiding regressions. Since about c7, winners have gained marginally, often at noise level.

## Recurring failures

- Easier routes into a crowded target lost four times (c3 v001, c3 v002, c6 v003, partly c8 v002). In tight caches, a "free" slot or an extra credit triggers an eviction cascade onto later promotions.
- Changes to the target→probing trigger lost in both directions (c4 delayed it, c5 sped it up). The trigger is treated as near-optimal.
- Counter and bit state lost repeatedly. Counters that reset on moves stayed near zero (c2). Counters that never decay pinned items permanently (c4). A 1-bit CLOCK in 2-slot buckets mostly picks the same victim LRU would, and entry and sweeps clear it before it is read (c8, c9).
- Stash victim selection that departs from plain LRU lost on every cell (c4). LRU-end insertion of fresh puts starved new keys (c2).
- `web_0` 16K has been flat for about six cycles. It resisted ghost reach, placement, free-slot routing, CLOCK victim choice, ghost-carried bits and value-aware gating. It looks capacity-saturated, and any lever based on position or a single bit is exhausted there.
- Per-op-time and metadata-size have not been measured for nine cycles. Every cost claim is unverified, so variants whose only purpose is cost reduction (c1 v003, c9 v003) produced no usable signal.

## Open questions

- Would a multi-bit, per-resident reuse count that persists through entry, sweeps and stash stays give victim choice something LRU order doesn't already encode? This is the main remaining idea for `web_0` 16K.
- Should admission or credit depend on how valuable the displaced resident is, rather than on whether a slot is free? `hm_0` 128K shows that displacing residents can pay.
- Would ghost-origin promotees placed at the LRU end with the reference bit set let CLOCK finally diverge from LRU?
- Could an adaptive ghost reach (per trace or occupancy-conditioned) combine `web_0`'s preference for a short ring with `hm_0` 128K's long tail?
- What causes the small `web_0` 128K giveback that appeared in c6? Ruled out so far: foreign retries, stash→full-target promotion and fixed ghost reach. Ghost churn from probing evictions has not been tested.
- Could a per-bucket index over the global ghost FIFO cut lookup cost while keeping its reach? This only matters once per-op-time is measured.

## Per-policy notes

- Stash (bucket-local LRU): plain LRU eviction is confirmed. Use MRU-end demotion and the two-hit exit gate, with the gate bypassed when the target is empty (strict threshold). Placing gated promotees at the target `Back` is essential. The stash-overflow rescue into free slots is settled for the large size.
- Ghost ring: use one global FIFO at factor 2 that is consumed on re-put. It is the main fix for returning keys (`web_0` 128K). Partitioning, longer reach, double-spending and occupancy-conditioned credit all hurt. `hm_0` 128K depends on its long tail.
- Normal buckets (LRU / 1-bit CLOCK): CLOCK is effectively the same as LRU in 2-slot buckets, and the c8/c9 winners were the same as their parents within noise. Move-to-MRU on a hit already carries the recency signal, so any new victim signal must carry information beyond position.
- Probing tier: `Back` placement for target→probing promotees pays off on `hm_0`, whose hot set reaches probing. Leave the trigger timing alone. Changes to how foreign residents retry their target had no effect.
- Traces: `hm_0` rewards protecting residents and long ghost memory, and its 128K cell is the most sensitive to regressions. `web_0` rewards fast first-hit admission when targets have room and a shorter ghost ring. The two traces often pull timing levers in opposite directions.

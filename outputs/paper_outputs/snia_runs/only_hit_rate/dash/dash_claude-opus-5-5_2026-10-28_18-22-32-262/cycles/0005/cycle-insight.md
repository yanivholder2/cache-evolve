# Cycle 0005 Insight

## Cohort outcome

All three variants built on cycle-0004 v001. v001 moved target→probing promotees to the probing MRU end, v002 promoted to probing on any target hit, and v003 added a FIFO ghost ring of stash-evicted keys so a returning key passes the two-hit gate on its first stash hit. The Winner is v003. It is the first lever in this lineage to move `web_0` 128K. It gained on three cells, stayed flat on `web_0` 16K, and regressed nowhere. v001 also had no regressions, but its gains were limited to `hm_0`. v002 lost on every cell.

## Patterns

- The history lever worked where in-item counters had not. The ghost keeps reuse evidence after eviction and decays by FIFO overwrite, which addresses both failure modes from earlier cycles (counters reset on moves, counters that never decay).
- `web_0` 128K turned out to be an admission problem: keys that come back after being evicted from the stash. It was not about placement or the probing trigger. v001 left it unchanged, v002 made it worse, and v003 lifted it.
- The placement rule held for the third time (v001): an item that just proved itself should not land in the victim slot. This time it paid off on `hm_0`, whose hot set actually reaches probing. v001 and v003 act on separate tiers and separate traces, so they look stackable.
- The target→probing trigger looks near its optimum. Cycle 0004 delayed it and this cycle sped it up, and both lost. The cost of a trigger change comes from the eviction cascade each extra crossing sets off.
- The ghost's reach is tied to stash size, which plausibly explains the flat `web_0` 16K cell. The ghost also adds metadata, and metadata-size is still unmeasured for the fifth cycle in a row, as is per-op-time.

## Open questions

- Does a longer ghost (a larger factor, or a ring sized to the segment) lift `web_0` 16K? Does it start costing at 16K `hm_0` through stale credit?
- Do v001's probing MRU placement and v003's ghost stack additively?
- Should demotions from a normal bucket also check the ghost, or record into it?

## Continuation

Build on v003. The top candidates are to add v001's `Back` probing placement and to test the ghost ring's reach (size/factor) on `web_0` 16K. Drop target→probing trigger changes in both directions. Measure metadata-size and per-op-time now, since the ghost ring adds metadata.

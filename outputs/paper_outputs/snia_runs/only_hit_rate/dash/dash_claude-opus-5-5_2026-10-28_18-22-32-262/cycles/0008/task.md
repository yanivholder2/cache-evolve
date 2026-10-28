# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0008

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

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


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants built on cycle-0005 v003 (the stash-eviction ghost ring). v001 added `Back` placement for target→probing promotees, v002 made the ghost ring four times longer, and v003 kept fingerprints after a re-put and spent them on demotion credit. The Winner is v001. It gained on both `hm_0` cells, beating each single lever alone. `web_0` 16K stayed flat and `web_0` 128K slipped slightly. v002 and v003 gained nowhere.

## Patterns

- Placement and the ghost mostly stack, but not perfectly. On `hm_0` the gains add up because the two levers act on separate tiers. The two levers do interact through cascades: keeping proven keys in probing pushes evictions onto foreign residents. Those residents cascade into the stash and churn the ghost, which plausibly explains the small `web_0` 128K giveback.
- The ghost's current design is close to its optimum on both axes tested. Longer reach (v002) credited keys that come back too late to be worth caching and hurt most on the very cell it targeted, `web_0` 16K. Spending the same evidence twice (v003) displaced residents that were just as hot and hurt most where the ghost had gained most, `web_0` 128K. Consume-on-put with a factor-2 FIFO ring is the right lifecycle.
- This is the third time an easier route back into a crowded target has lost: cycle 0003 v001, cycle 0003 v002, and now v003. Ghost-backed evidence does not save that pattern. A key that has already lost its slot should not outrank the target's current residents.
- `web_0` 16K has been flat for several cycles and has now resisted the ghost, its reach, and probing placement. Its bottleneck is not ghost span.
- Per-op-time and metadata-size are still unmeasured. That is now six cycles in a row. v002's longer ring and v003's extra scan per demotion both had per-op-time implications that were never reported.

## Open questions

- Can the `web_0` 128K giveback be recovered by changing how displaced foreign probing residents retry their target, or whether they cascade at all, without losing the `hm_0` gain?
- What limits `web_0` 16K? Candidates are stash size relative to the segment, or demotion placement for items that have no ghost history. Length of memory is now ruled out.
- Would a shorter ring (factor 1) cut stale credit at 16K, or is factor 2 already at the floor?

## Continuation

Build on v001. Drop ghost-reach tuning upward and ghost credit on demotion, and treat the ghost lifecycle as settled. The top candidate is to soften the foreign-resident cascade that v001's probing placement triggers, aiming to recover `web_0` 128K. A second candidate is a new, non-ghost probe of `web_0` 16K. Report per-op-time and metadata-size this cycle.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

All three variants built on cycle-0006 v001, which combines the ghost ring with `Back` probing placement. v001 sent displaced foreign residents straight to the stash when their target was full. v002 rescued stash-overflow victims into free target or probing slots. v003 halved the ghost ring to factor 1. The Winner is v002. It gained on `hm_0` 128K, was marginally up on `web_0` 128K, and regressed nowhere. Its target cell, `web_0` 16K, stayed flat. v001 matched the parent exactly. v003 was mixed: a slight gain on `web_0`, a slip on `hm_0` 128K.

## Patterns

- The `web_0` 128K giveback is still unexplained. The foreign-retry cascade is now ruled out: v001 had no effect at all, which suggests the retry already evicts the foreign item itself. The rescue (v002) and the shorter ring (v003) moved it only marginally.
- Levers that use free capacity only pay off at the large size. At 16K, normal buckets are saturated whenever the stash overflows, so v002's rescue rarely fires there. `web_0` 16K is a capacity-saturated cell, not an idle-slot or ghost-reach problem.
- Ghost reach has an optimum that depends on the trace. `web_0` prefers the shorter ring, which gives less stale credit. `hm_0` 128K needs the longer tail, because its keys return between one and two stash-capacities after eviction. Most of the ghost's value comes from returns within one stash-capacity. Factor 2 remains the safer global setting.
- No-cascade, free-slot-only moves stay safe: v002 displaced no residents and regressed nowhere. This fits the lineage's lesson that an item that has lost its slot must not outrank residents.
- Per-op-time and metadata-size are still unmeasured, for the seventh cycle in a row. v003's cost argument is unverified as a result.

## Open questions

- At 16K, does a different victim choice in full normal buckets move `web_0`? Candidates are recency-vs-frequency victim selection and ghost-informed victim choice.
- Could an adaptive ghost reach (per trace, or occupancy-conditioned) combine v003's `web_0` gain with factor 2's `hm_0` 128K tail?
- Does v002's rescue push targets below the bypass free-slot threshold and silently turn bypass promotions into gated ones?

## Continuation

Build on v002. Drop foreign-retry changes and further fixed ghost-factor tuning. Treat the rescue as settled for the large size. For `web_0` 16K, change which resident is evicted from a full bucket rather than looking for empty slots. Investigate the `web_0` 128K giveback through ghost churn from probing evictions. Measure per-op-time and metadata-size.


## Write

Exactly one file: `cycle-insight.md` in the output dir.

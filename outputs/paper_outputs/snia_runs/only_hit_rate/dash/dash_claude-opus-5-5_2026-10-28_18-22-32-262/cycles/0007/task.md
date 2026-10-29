# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0007

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants built on cycle-0003 v003 (`Back` promotion placement plus the two-hit gate). Each tried a different lever: v001 skipped the gate when the target has two free slots, v002 delayed a fresh promotee's move to probing, and v003 changed which stash item gets evicted. The Winner is v001, with a small lift on `web_0` 128K and no regressions. v003 lost hit-rate in every cell.

## Patterns

- The stricter room threshold worked as intended. It kept cycle-0003 v002's `web_0` 128K gain and avoided its 16K loss, which confirms that the last-free-slot cascade caused that loss. With 2-slot buckets, though, the bypass fires only when the target is empty, so this lever has little room left.
- `web_0` 128K is still the bottleneck. This cycle ruled out two explanations for it: one-hit churn into probing (v002 made it worse, which suggests hot promotees want to reach probing sooner) and stash flushing of half-gated items (v003). `hm_0` behaves the opposite way and rewards protecting residents. The two traces keep pulling the timing levers in opposite directions.
- Counters that never decay are now a recurring failure mode. v003's "one hit" protection turned into permanent pinning, and v002's target-hit count only weakly filtered anything. Together with cycle 0002's near-zero counters, this means the `lfu_counter` lifecycle (when it resets, whether it decays) shapes the outcome more than any threshold placed on top of it.
- Plain LRU is strongly confirmed as the right stash eviction rule under the two-hit gate. Fresh demotees especially need the recency protection that `Back` demotion gives them.
- Per-op-time and metadata-size are still unreported, for the fourth cycle in a row.

## Open questions

- Is v002's `hm_0` gain worth keeping through a trace-adaptive or occupancy-conditioned version that leaves `web_0` 128K alone? Or is it too small to pursue?
- Does `web_0` 128K benefit from faster target→probing movement, for example promoting on any target hit and not only at the hottest position?
- Would a stash protection that expires (newest region only, or cleared at the LRU end) gain at 16K, or does any departure from LRU hurt?

## Continuation

Build on this cycle's v001. Drop stash victim-selection by counter and stop tuning the free-slot threshold. For `web_0` 128K, probe the opposite direction from v002: let promotees reach probing more easily. Any new frequency signal must come with an explicit decay or reset rule. Measure per-op-time and metadata-size before making cost claims.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.

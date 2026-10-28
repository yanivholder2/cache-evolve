# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0006

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0006/hypotheses/H03.md

## Prior cycles digest

### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants built on cycle-0002 v001 (MRU-end stash demotion plus the two-hit gate). Each changed one aspect of the gate: v001 changed which items it applies to (demoted items return on one hit), v002 changed when it applies (skipped when the target has a free slot), and v003 changed where items land once they pass it (target MRU end instead of LRU end). The Winner is v003. Its gains came from both 16K cells and `hm_0` at 128K, while `web_0` at 128K stayed flat.

## Patterns

- Where a promotee lands mattered more than tuning the admission rule. With `Front` placement, an item that passed the gate became the target's next eviction victim, so the gate was partly defeating itself. Fixing placement paid off most where target slots are contested.
- v001 disproved the cycle-0002 explanation. Removing the counter-reset penalty on demoted items barely moved `web_0` at 128K. The regression there comes from gating fresh puts when targets have room, not from the reset.
- v002 is the only variant that recovered `web_0` at 128K. v003 gained in the cells v002 lost (16K) and stayed flat in the cell v002 fixed. The two levers cover opposite cells and look complementary.
- Small caches kept penalizing any easier route into the target (v001's one-hit return, v002's free-slot shortcut). In tight caches, a one-hit item that takes the last slot pushes a cascade onto the next promotion. A free slot is not actually free when space is tight.
- The variant insights again report only hit-rate. Per-op-time and metadata-size are still unverified.

## Open questions

- Does v002's occupancy shortcut stack on v003's `Back` placement, recovering `web_0` 128K without giving back the 16K gains?
- Would a stricter room condition (more than one free slot, or an occupancy fraction) remove v002's 16K cost?
- Is v003's flat `web_0` 128K cell caused by promotees jumping to probing on their next hit? Would requiring one more target hit before probing fix it?

## Continuation

Build on v003. The top candidate is to add v002's occupancy-aware gate on top of it, preferably with a stricter room threshold so the 16K cells are protected. A second candidate is delaying a fresh promotee's move to probing. Drop the demoted-item counter lever (v001), since it is near-neutral. Get per-op-time and metadata-size measured before making any cost claims.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.

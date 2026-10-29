# Task: cycle-insight-writer

Cycle: 0004
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0004

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0004/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0004/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0004/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0004/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0004/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0004/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants branched from baseline and each changed one thing: v001 changed where demoted items land in the stash, v002 added a two-hit gate before stash items promote, and v003 changed the lookup order as a cost-only control. The Winner is v001, which moved demoted items to the stash MRU end. It improved hit-rate on every trace and cache size. v002 also gained hit-rate in most cases, but not all.

## Patterns

- The baseline stash was ordering items badly: items that had already been reused were evicted before brand-new or never-hit items. Both hit-rate variants fixed part of this. v001 changed where items enter the stash. v002 changed what it takes for an item to leave the stash. The two are close to independent and could be combined.
- How much each change helped depended on cache size in different ways. v001 helped more at the large cache size, where more items get demoted. v002 helped most at the small size, where the normal tier is crowded, and gave nothing or slightly worse on web_0 at the large size.
- The whole cohort was scored on hit-rate only. per-op-time and metadata-size were never measured. All three variants' claims about cost (no change for v001, cheaper for v002 and v003) are unverified. v003, whose only purpose was per-op-time, produced no usable signal. It did confirm that the lookup reorder leaves hit-rate exactly unchanged.

## Open questions

- Do v001's demotion position and v002's two-hit gate add up when combined, or do they overlap? Both protect reused items from one-hit arrivals.
- Should the promotion gate scale with capacity or occupancy instead of staying fixed at two hits, so it stops costing hit-rate in roomy caches?
- Does the target-first lookup order actually lower get latency? This needs an evaluation that reports per-op-time.

## Continuation

Build on v001. Next, test it combined with the two-hit gate, and include a variant where the gate scales with occupancy, to see whether the small-cache gains hold without the large-cache regression. Make sure the evaluation reports per-op-time and metadata-size, then re-test the lookup reorder (v003). Its hit-rate result is identical to baseline, so it can be added to any winner without risk. Do not count it as a cost win until latency is measured.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants built on cycle-0001 v001 (MRU-end stash demotion), and each added a different stash lever. v001 added a fixed two-hit exit gate, v002 added a TinyLFU-style frequency comparison against the target's LRU victim, and v003 inserted fresh puts at the stash LRU end with a BIP escape. v001 won with small gains at the small size and on `hm_0` at the large size. It regressed on `web_0` at the large size, the same weak spot as cycle-0001's gate.

## Patterns

- All three levers act on the same small population: items sitting in tiny, bucket-local LRU stash slots. That makes them interact rather than stack. v001's counter reset on demotion penalizes exactly the demoted items that `Back` placement favors. v003's `Front` insertion behaves like "evict on next collision", so new keys lose their probation hit.
- `web_0` at the large size stayed the hardest cell. No variant recovered it, and every extra filter (gate, comparison, LRU-end insertion) cost hit-rate there. When target buckets have spare room, first-hit promotion is already close to optimal.
- Gains appeared mainly where reuse comes soon or contention is high: small size for the gate, `hm_0` at the large size for v002 and v003. Results split across cells far more often than they moved together.
- Frequency signals are weak in this design. Promotions land at the target's eviction end, and counters are halved or reset on moves, so counters stay near zero. v002's comparison therefore ended up acting like plain first-hit promotion.
- Per-op-time and metadata-size were again not reported. All cost claims are still unverified.

## Open questions

- Does the gate help without regression if it applies only to fresh puts, so demoted items keep their count or promote on their first stash hit?
- Would a frequency comparison actually bite if it compared against the bucket's lowest counter, or if promotions didn't insert at the LRU slot?
- Can fresh-key insertion be softened (one slot above LRU, per-bucket bimodal counter) so it keeps the `hm_0` large-size gain without starving new keys?

## Continuation

Build on this cycle's v001. The top candidate is to restrict the two-hit gate to fresh puts and stop resetting counters on demotion, since that targets the `web_0` large-size regression directly. If frequency admission is retried, fix the victim choice (lowest counter, not LRU index 0) first. Drop hard LRU-end insertion of fresh puts. Make sure per-op-time and metadata-size are measured before any cost claim is made.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.

# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0009

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0009/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants built on cycle-0007 v002, which adds the rescue. v001 added a 1-bit CLOCK reference for picking victims in normal buckets. v002 gave ghost credit from the far zone only when the target had a free slot. v003 sent gated stash promotions into free probing slots when the target was full. The Winner is v001. Its gains were marginal, possibly noise, and came only at 128K on both traces. Its target cell, `web_0` 16K, stayed flat once again. v002 moved gains around without lifting the mean: `web_0` went slightly up and `hm_0` 128K slipped. v003 left `web_0` 128K exactly where the parent had it and lost a little on `hm_0` 128K.

## Patterns

- `web_0` 16K survived a victim-choice lever too. In saturated 2-slot buckets, a reference bit almost always picks the same victim LRU would. Move-to-MRU on a hit already encodes recency, and items enter and leave too fast to build up a second signal. Changing victim selection is not enough unless it brings information that LRU order does not already carry.
- `hm_0` 128K is the cohort's sensitive cell. Both losing variants hurt it, each in a different way. v002 denied credit to long-tail returns that had been displacing residents profitably. v003 used up probing headroom early. The lineage rule that "lost-slot items must not outrank residents" does not hold universally: on `hm_0` 128K, displacing residents sometimes pays.
- Levers that only use free slots are exhausted. Once the rescue exists, sending other items into free slots just changes the order in which the same headroom is spent (v003). Conditioning on occupancy reduces to a crude credit drop, because the bypass already handles empty targets (v002).
- The `web_0` 128K giveback is still unexplained. Stash→full-target promotions are now ruled out as its source, as are foreign retries (c7) and ghost reach.
- Per-op-time and metadata-size are still unmeasured. That is now eight cycles in a row.

## Open questions

- If items are not moved to MRU on a hit, so the reference bit becomes the only recency signal, does victim choice at 16K finally diverge from LRU?
- Could ghost credit or victim choice depend on the resident that would be displaced (its recency or hit count) instead of on whether a slot is free?
- Does reuse that survives across stash stays, or frequency derived from the ghost, provide a victim signal that LRU order lacks?

## Continuation

Build on v001. Drop free-slot routing and occupancy-conditioned ghost credit. For `web_0` 16K, try either removing move-to-MRU on hits in normal buckets or a victim signal that carries history across stash stays. Base admission and credit decisions on how valuable the resident being displaced is, since `hm_0` 128K shows that displacement can pay. Measure per-op-time and metadata-size.


## Write

Exactly one file: `cycle-insight.md` in the output dir.

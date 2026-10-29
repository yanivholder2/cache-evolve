# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

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


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

All three variants built on cycle-0008 v001 (CLOCK victim bit). v001 started ghost-returning promotees with the reference bit set. v002 required an extra stash hit when every resident of the full target was referenced. v003 split the ghost ring into per-stash-bucket sub-rings to cut put-path cost. The Winner is v001, but only by not regressing: its 128K gains are rounding-level noise and `web_0` 16K stayed flat for yet another cycle. v002 matched the parent exactly. v003 lost a little everywhere except a noise-level uptick at `web_0` 16K.

## Patterns

- A 1-bit CLOCK in 2-slot buckets can't carry a value signal, and both bit-based levers ran into that. MRU position already protects a newcomer, so setting its bit (v001) can only cancel the LRU resident's protection. MRU hits send items to probing, LRU hits move items to MRU, and entry plus `make_room_clock` clear bits, so an "all residents referenced" state (v002) almost never happens. The bit gets reset before anything reads it.
- The ghost's reuse evidence is real, but it does nothing on victim choice unless it lands where position doesn't already decide the outcome, and unless it is held in state that survives clearing.
- The ghost works because it is global and adaptive. A shared FIFO lends its reach to whichever buckets are evicting. Fixed per-bucket sub-rings (v003) cut reach in exactly the hot buckets, and the rescue path makes evictions more skewed, hitting `hm_0` 128K hardest. This confirms that the ghost lifecycle should stay settled.
- `web_0` 16K has now been flat through ghost reach, placement, free-slot routing, CLOCK victim choice, ghost-carried bits and value-aware gating. Any lever that relies on 1-bit or position state is exhausted there.
- Per-op-time and metadata-size are still unmeasured, now nine cycles in a row. v003's only possible benefit is therefore unverifiable.

## Open questions

- If ghost-origin promotees were placed at the LRU end with bit 1, would CLOCK finally spare them over an unreferenced MRU neighbour?
- Would a multi-bit, per-resident hit count that is not cleared on entry or by the sweep give value-aware victim choice or gating something to act on?
- Could a per-bucket index or filter over the global ghost FIFO make lookups cheaper while keeping global reach?

## Continuation

Build on v001, or on its parent, since the two are effectively the same. Drop single-bit CLOCK tweaks and occupancy- or all-referenced gates, and keep the global ghost FIFO. Partitioned rings are ruled out. For `web_0` 16K, the remaining untried directions are a multi-bit resident counter that persists through entry and sweeps, or ghost-origin placement at LRU with bit 1. If per-op-time is not added to the fitness vector, stop proposing cost-only variants.


### Cycle 0010

# Cycle 0010 Insight

## Cohort outcome

v001 and v002 built on c9 v001. v001 replaced the CLOCK bit with a persistent, aged 2-bit reuse level. v002 made far-zone ghost credit depend on a per-segment feedback score. v003 started from c7 v002, which has plain LRU and no CLOCK, and added a count-min frequency sketch that sits outside the items and ages by halving. v003 uses the sketch to refuse a gated stash→full-target promotion when the promotee's estimate is below the LRU resident's. The Winner is v003, the first variant in several cycles to make small gains in every cell with no regressions. Its gain was not concentrated at 16K, though, and `web_0` 16K moved only slightly. v001 was flat on its target cell, with noise-level dips elsewhere. v002 helped `web_0` slightly and slipped on `hm_0` 128K, a pattern earlier ghost-reach variants also showed.

## Patterns

- The first lever that worked at all at 16K was choosing **who competes**, not how the two residents rank. v001 shows that even a persistent multi-bit level collapses back to LRU in 2-slot buckets. Contested entrants arrive almost saturated, aging per contest turns the field back into recency, hot items leave for probing, and the newcomer is never compared. v003 puts the promotee itself against the resident it would displace.
- Frequency is useful only when it is stored outside the item, survives moves and evictions, and decays. The ghost works for the same reasons. The c2 in-item counter and v001's in-item level fail because they live inside items that churn.
- Making admission stricter is safe, and making it easier keeps failing. v003 can only delay a promotion by a hit, which is why nothing regressed. That fits the lineage's lesson about crowded targets.
- The sketch's aging window scales with cache size, so at 16K it halves too often to tell candidates apart. This is the likely reason for v003's small gain at the small size.
- `hm_0` 128K rejects every ghost-credit restriction tried so far, now including outcome-based feedback (v002). Its far-credited keys pay off on later returns, not immediate re-hits. A one-stay reward signal misreads them, and fixed factor 2 is still the best setting.
- The lineage has moved to a different branch: the Winner sits on c7 v002, not on the CLOCK line. The CLOCK and level work of c8–c10 added almost nothing on top of c7 v002.
- Per-op-time and metadata-size are still unmeasured, now ten cycles in a row. v003 adds sketch memory and a sketch update on every get, and none of that cost is tracked.

## Open questions

- Would a longer or fixed aging window at 16K, or a wider sketch, let the admission check separate candidates where it currently ties?
- Should ties refuse instead of admit at the small size, or should the check use a margin?
- Can the sketch also help elsewhere: rescue, bypass, victim choice, or weighting ghost credit by estimated frequency instead of reach?
- Does stacking the c8/c9 CLOCK bit onto v003 add anything, or should it stay dropped?

## Continuation

Build on v003. Tune the sketch first: aging window at small sizes, tie policy, width. After that, extend sketch-based judgments to other displacement decisions. Drop in-item counters, CLOCK/level victim ranking, and any further ghost-reach or far-credit gating, since `hm_0` 128K defeats them all. Measure per-op-time and metadata-size, now that the Winner adds per-get work and memory.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>

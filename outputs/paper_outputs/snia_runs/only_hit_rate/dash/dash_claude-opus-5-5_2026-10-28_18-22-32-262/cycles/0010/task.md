# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0010

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.

# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0007

## Winners

- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants branched from cycle-0002 v003 (flag-packed SLRU on `u32` nodes). v001 tried an adaptive protected/probation split, v002 a single-`Entry` put path with no key clone, and v003 a gentle fingerprint-ghost second chance inside probation. The Winners are v002 and v003. v002 is the only clean gain: put got faster with policy and layout unchanged. v003 is the first history mechanism that helped web_0 instead of hurting it, though the net hit-rate gain is small and get latency regressed.

## Patterns

- Cutting hash operations on put works. v002 bears out the cycle-0003 finding that put cost tracks hash count. It is the first latency lever across four cycles that moved its target without hurting hit rate or metadata.
- Gentle admission solves web_0's sensitivity. A one-shot reprieve that stays in probation lifted web_0 at both sizes, which reverses the damage cycle 3's ghost and gate did. Its benefit is size-dependent, though: hm_0 gained at the small size and lost slightly at the large size.
- Small per-cache hit-rate effects are hidden by noise. v001's one-epoch hill-climber kept chasing workload swings and lost on hm_0. Its only win was web_0 at the small size, which again suggests probation is undersized there.
- Any extra work on the hit path is very costly for get. v001's per-op tick and v003's masking of `next` both raised get p50 sharply. Header growth past one cache line and heap side arrays probably add to this. Both policy variants also grew the `LruCache` header. Repeated across cycles, the rule is that the `on_hit` link path and the header's hot cache line must stay untouched.
- v002's get p99 rose although its get code was identical. This may be noise, or the insert-before-remove order may be changing the hash table's internal state. It is unresolved.

## Open questions

- Is v002's get-p99 rise real? If so, would removing before inserting (so the new key reuses the freed slot) keep the put gain without the tail?
- Can v003's second-chance bit live somewhere that is free on the hit path, such as a padding byte or the key/value side rather than `next`, so get p50 recovers?
- Should the reprieve be gated by cache size, applying only when probation is small relative to the working set, so it stops costing hm_0 at the large size?
- Would a static per-size retune of the probation share capture v001's web_0 small-size gain with no runtime cost?

## Continuation

Use v002's single-`Entry`, no-clone put as the new default base. Re-check its get p99, and if it is real, try remove-before-insert. If you pursue hit rate, port v003's probation-only reprieve onto that base, with the flag stored off the `next` link and the header kept within one cache line, and consider scaling it by cache size. Drop online hill-climbing of the SLRU split. Try a static per-size probation share instead.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants branched from cycle-0004 v002 (single-`Entry`, no-clone SLRU). v001 ported the fingerprint reprieve with its flag in `prev` and the ghost enabled only for small caches. v002 replaced protected-hit relinks with a CLOCK reference bit. v003 used a static 1/3 probation share at small sizes. v001 is the only Winner. It lifted both traces at 16K and matched the parent at 131K, but gave back v002's put gain by doubling hash work, and get p50 still regressed.

## Patterns

- Gating by size works for hit-rate mechanisms. Both v001 and v003 left 131K untouched by design, so every hit-rate change came from 16K alone. v001's gate removed the large-size hm_0 loss from cycle 4.
- At 16K the reprieve beats resizing. A static wider probation helped hm_0 but hurt web_0, so no fixed share suits both traces. The reprieve gives extra probation time only to keys that come back, and it helped both. Selective second chances do better than a uniformly bigger window.
- Put cost still tracks hash count. v001's separate `hash_one` for ghost tags, on top of the map's own hashing, erased the single-probe gain. This is the third cycle confirming the rule.
- Hit-path changes still hurt get p50, even tiny ones. An extra AND in `set_protected` (v001) and CLOCK moving list work into promotion sweeps (v002) both regressed it. Removing relinks from protected hits did not lower the median. That is more evidence that get p50 is not driven by list work.
- The timing benchmark is too noisy to rank small per-op changes. v003 changed nothing on the hot path, yet its per-op times swung widely in opposite directions. Get p99 swings in every variant now look like noise, including cycle 4's p99 rise.

## Open questions

- Can v001's ghost reuse the hash the map already computes (raw-entry or `hashbrown` API), and run only on the vacant path, to restore v002's put cost?
- If `set_protected(true)` is kept to a single OR and `SC_BIT` is cleared at reprieve or eviction time, does get p50 recover?
- How much timing movement is just run-to-run variance? Repeated paired runs of parent and child are needed before trusting per-op deltas under a few percent.

## Continuation

Make v001 (size-gated `prev`-bit reprieve) the base, and fix its costs next. Derive the ghost tag from the map's own hash, skip ghost work on occupied-key puts, and make `set_protected` a single OR. Drop protected-segment CLOCK and static probation-share retuning. Judge latency claims only from repeated paired runs.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

Two variants branched from cycle-0005 v001 (the size-gated `prev`-bit reprieve). v001 cut the ghost's hashing cost (Fx tags, vacant-path only, single-OR `set_protected`). v003 turned the reprieve back on at 131K with a much smaller ghost, sized at capacity/8. v002 branched from cycle-0004 v002 and replaced hashbrown with a custom tombstone-free index table. All three are Winners, but each won only partly. v001 got most of the median put cost back with no hit-rate loss. v002 fixed the get tail but lost get p50. v003 was roughly neutral on hit rate at 131K.

## Patterns

- Two get-p50 explanations have now been ruled out. v001 removed the flag arithmetic from `on_hit` and get p50 still regressed. v002 touched fewer cache lines per lookup and still lost to hashbrown's SIMD group probing. So get p50 does not depend on flag bit-ops, list work or lookup cache lines. The leading suspects left are code layout and inlining of the shared helpers, and the ghost table's cache footprint.
- Put cost tracks hash count, now for the fourth cycle. A cheap fingerprint hash recovered the put median, and a 16-bit tag does not need a strong hash. v002's backward shift, which runs a full SipHash for each entry it moves, pushed the put tail up the same way.
- Removing the victim before inserting the new key fixes the get tail. v002 is the first evidence that the cycle-4 get-p99 rise was real and came from tombstone churn, not noise.
- Noise now hides the effects this cycle measured. v003's timings swung sharply on code that was byte-identical to v001. Its 16K hit rate moved on unchanged code, most likely because `RandomState` is reseeded on every run. Small hit-rate deltas cannot be trusted while the ghost hasher is unseeded or randomly seeded.
- An undersized ghost works as a reuse-distance filter, but at capacity/8 it is too short for web_0 at 131K. It forgets most of the useful re-references before they come back.

## Open questions

- Does a wider large-size ghost (capacity/4 or capacity/2) recover cycle 4's web_0 gain at 131K while still keeping out hm_0's long-loop keys?
- Is the remaining get-p50 regression caused by inlining and layout of `on_hit`/`unlink`/`push_back` after the put restructure? Forcing or blocking inlining of those helpers would test this.
- Does remove-before-insert on the stock HashMap parent get v002's tail fix without losing hashbrown's probing speed?

## Continuation

Use cycle-0006 v001 (Fx ghost tags, vacant-only ghost work) as the base. Fix the ghost hasher's seed first, so hit-rate deltas can be measured, and time parent and child in repeated paired runs. Then try remove-before-insert eviction on hashbrown, and a large-size ghost wider than capacity/8. Drop the custom scalar index table, and drop further bit-op trimming in `on_hit`. Look into the get-p50 regression through helper inlining and code layout.


## Write

Exactly one file: `cycle-insight.md` in the output dir.

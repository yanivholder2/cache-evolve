# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0007

## Winners

- v002
- v001
- v003

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants branched from cycle-2's v003. v001 was a state-neutral refactor that took redundant hashing and re-scans out of the promotion branches. v002 made the stash two-choice: it fills a free slot in the neighbouring stash bucket before evicting. v003 rescues stash-eviction victims into free slots in their own target or probing bucket. v002 and v003 both confirmed their hypotheses and are the first hit-rate gains since cycle 2, but both cost clear per-op-time. v001 kept hit-rate exactly the same as intended, but its get-p50 claim was refuted.

## Patterns

- Idle capacity, not admission policy, was what held hit-rate back. Both winning hit-rate levers keep items that would otherwise be dropped by using holes elsewhere: a neighbour stash bucket in v002, a normal-tier bucket in v003. Holes from promotions (`remove_at`) are not refilled fast enough by their own keys. This fits cycle 3's verdict that the tiny buckets are the constraint.
- Neither lever adds metadata. The extra placement choice is derived from the key hash or bucket fullness, so metadata-size stays the same in every variant.
- Retaining more costs latency on both sides of the cache. v002 adds a second stash scan to every miss, because the stash is checked first. v003 makes stash overflow a multi-bucket operation and leaves normal buckets fuller, so every lookup scans more. More residency means more scanning, so hit-rate and get latency are now traded off directly.
- Trimming bookkeeping doesn't move get p50 (v001). The median is set by the common non-promoting lookup path and by the promotion moves themselves. Any p50 win has to come from cheaper scans or from doing fewer moves.

## Open questions

- Do v002 and v003 stack? Both touch the stash-insert path. Rescue would fire only when both stash choices are full, so the hit-rate gains may overlap. The latency costs would add up.
- Can the second stash scan be avoided on misses without losing v002's gain? Options include a cheap filter or hint for `s1`, or the target/probing-first order. Cycle 3 found stash-first better on this lineage, so that order needs re-testing under two-choice.
- Would an independent second hash for `s1` keep more items than the fixed `s0+1` stride?
- Does rescue placement at Front, followed by stash-Back re-entry of cold rescued items, ever start to hurt as the cache fills?

## Continuation

Adopt v002, with v001's state-neutral refactor folded in, as the hit-rate parent. Then test v003's rescue on top of it to see if the gains stack. Next, recover get latency on the two-choice structure without changing cache state: filter or short-circuit the `s1` scan on misses, or make per-bucket scans cheaper. Stop pursuing pure bookkeeping trims on the promotion path, because they don't reach p50. From now on, judge any further hit-rate gain against its per-op-time cost.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants branched from cycle-4 v001 and used spare capacity in different ways. v001 added the two-choice stash but probes `s1` last. v002 combined the two-choice stash with normal-tier victim rescue. v003 puts fresh items into free target slots at insert time. All three raised hit-rate over the parent, and v002 confirmed that cycle 4's two levers stack. Only v001 met its latency claim, and only relative to cycle-4 v002. Every variant is still slower than the parent on get, and v003's latency claim was refuted.

## Patterns

- Using idle capacity is still the only lever that raises hit-rate. Every variant that keeps items in neighbour stash buckets or normal-bucket holes gains. Metadata-size stayed flat in all of them. The neighbour-stash and rescue levers draw on different capacity, so their gains add up. v002 can't tell how much each one contributes.
- The cost of extra residency shows up in latency, and the lookup order decides who pays it. Scanning `s1` before the normal tiers charges every target and probing hit. Scanning it last (v001) charges only misses and the rare `s1` hits, with no change in cache state. The reorder is state-neutral because a key lives in at most one slot, and v001's exact hit-rate match confirms this. The same argument should carry over to the rescue structure.
- When an item fills a hole matters. Filling holes late (rescuing stash victims) costs nothing on lookup. Filling them early (fresh puts into target) fills the normal tier with cold items. Later reuse-proven promotions then pay for eviction and demotion cascades that used to be free inserts. The cost moves to promotions instead of going away.
- The get-latency gap that remains on the two-choice structure comes from misses scanning one more bucket. Further reordering can't close it, because `s1` is already last.

## Open questions

- Does v001's `s0 → target → probing → s1` order on top of v002's two-choice + rescue placement keep v002's hit-rate exactly and recover its get-p50 cost? It should, but this hasn't been verified.
- Can most `s1` probes on misses be skipped cheaply? One option is a per-`s0` "has overflowed" bit, another is a fingerprint. What does that do to metadata-size?
- Does fresh-into-target-hole add anything on top of rescue, given they compete for the same holes? Could fresh items be placed so they always give way to promoted items, without competing at the target's LRU end?

## Continuation

Make v002's placement (two-choice stash + rescue) the new parent, combined with v001's lookup order that probes `s1` last. Check that hit-rate is unchanged and get p50 improves. Next, go after the remaining get cost on misses with a cheap `s1` skip filter, and judge any metadata cost against the latency gain. Set aside v003's eager fresh-into-target fill: it trades latency for hit-rate in a way rescue already covers more cheaply. Revisit it only in a form that never blocks later promotions.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants branched from cycle-5 v001 (two-choice stash, `s1` probed last). v001 ported stash-victim rescue. v002 added an exact per-stash-bucket borrowed counter that lets `get` skip the `s1` scan. v003 placed native target victims into holes in their probing bucket. Only v002 confirmed its hypothesis: it cut get p50 and left hit-rate exactly unchanged. v001 matched cycle-5 v002's hit-rate, but its latency claim was mostly refuted. v003's hit-rate gain was within noise.

## Patterns

- Cycle 5 blamed v002's get cost on lookup order. Cache state is the real cause. Rescue keeps normal buckets fuller, so every target/probing hit and every miss scans more occupied slots, whatever the probe order. The hit-rate vs get-latency trade from extra residency is now a cost per scan, and reordering cannot remove it.
- Lossless filters work. A small exact counter, stored per segment and not per item, skips scans with no state change and no growth in tracked metadata. Its cost lands on `put`, and mostly in the tail, because the bookkeeping re-hashes keys the insert/evict path already hashed.
- The supply of idle capacity is running out. Stash-victim rescue still pays off. Probing holes at native-demotion time almost never exist in steady state, so that source can be ruled out. Adding branches on hot paths that rarely fire costs a little latency and gains nothing.
- Every variant raised `put` cost in some form: extra rescue checks, re-hashing for the counter, or longer cascades into fuller buckets. Write-path cost is now an axis to track, not noise.

## Open questions

- Does v002's borrowed-counter gate stack with v001's rescue structure? Does it still skip most `s1` probes once rescue changes stash turnover?
- Can the counter be maintained without re-hashing, by passing `s0` or a native/borrowed flag from the caller, so that put p99 comes back?
- Can in-bucket filtering (tags/fingerprints) shorten the longer scans that rescue causes, and what would it cost in metadata?

## Continuation

New parent: v001 (rescue + `s1`-last) with v002's borrowed-counter gate, with the counter fed the already-computed `s0` / native flag so no re-hashing happens. Check that hit-rate holds and that get p50 and put p99 both improve. Then target the scan cost of fuller normal buckets with in-bucket filtering, weighing it against metadata-size. Drop native-demotion-into-probing-hole and further probe-order changes.


## Write

Exactly one file: `cycle-insight.md` in the output dir.

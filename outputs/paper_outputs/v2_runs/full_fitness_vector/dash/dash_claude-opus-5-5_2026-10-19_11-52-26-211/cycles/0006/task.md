# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0006

## Winners

- v002
- v003
- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0006/hypotheses/H03.md

## Prior cycles digest

### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants were built on cycle-2's v003 (fresh puts at stash Front, demotees at stash Back). v001 ported the target→probing→stash lookup order. v002 required a second stash hit before promoting to target. v003 blocked target→probing promotions that would evict a native neighbour. All three are listed as Winners, but each one met its own falsification condition. None raised hit-rate. v002 and v003 shortened the get tail but lost a little hit-rate. v001 left hit-rate unchanged, as expected, but made get p50 worse.

## Patterns

- The parent's admission policy now looks like a local optimum on both edges. Cycle 2 showed that hotter stash→target promotion hurts. This cycle showed that stricter promotion on either edge also hurts: a second-hit gate on stash→target, and a native-ownership gate on target→probing. Every filter tried so far has blocked real reuse rather than noise. The tiny buckets (2-slot stash, small target) can't hold items long enough for an extra wait to pay off.
- How hits are spread across tiers depends on placement policy. Changes that hold for one cache state can stop holding when placement changes. Fresh-at-stash-Front placement sends every key's first reuse through the stash. That made stash hits common enough to flip the lookup-order result: cycle 1 found target-first was a win, and here stash-first is better.
- The two latency wins share one cause: skipping the remove → promote → demote → stash-insert cascade. Both removed it by promoting less often, and both paid for it in hit-rate. The tail cost comes from how expensive each cascade is, not from how often admission is allowed.
- The signal used to decide admission matters. Bucket position and native/foreign ownership are free to read. But neither tracks recency well enough to beat plain LRU-ordered promotion here.
- Metadata-size stayed the same in every variant. Put-path movements had no cause in the code changes and look like noise.

## Open questions

- Can the promotion cascade be made cheaper without changing which items get promoted? Options include in-place swaps instead of remove + reinsert, or limiting how far a cascade can go.
- Can each bucket scan be made cheaper so get p50 falls under any lookup order? Small per-slot fingerprints would do this, but they would cost metadata-size.
- Would a lookup order chosen per key or adaptively beat a fixed stash-first order? Or does the extra branching cost more than it saves?
- Stash capacity and associativity haven't been tested. Bigger stash buckets might let a second-hit filter work where 2-slot buckets couldn't.

## Continuation

Keep cycle-2's v003 as the parent. Nothing this cycle beat it on hit-rate, and v001's lookup order is a regression on this lineage. Drop admission filters on both promotion edges (second-hit, ownership-based) and drop fixed lookup-order changes. Go after get latency through cheaper cascades and cheaper bucket scans that leave cache state unchanged. For hit-rate, try structural levers instead: stash sizing and associativity, or eviction order inside the target buckets.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.

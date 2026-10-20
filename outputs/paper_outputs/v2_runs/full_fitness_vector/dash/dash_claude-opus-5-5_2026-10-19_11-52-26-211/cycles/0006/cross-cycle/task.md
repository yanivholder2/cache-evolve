# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three hypotheses made a single targeted change to the baseline's `promotion_get` path. One changed the stash-entry position for demoted items (v001), one added balanced placement on stash promotion (v002), and one reordered lookups (v003). All three became Winners. v001 gave the only clear hit-rate gain. v003 gave the only per-op-time gain that the code change itself can explain. v002 was safe but barely changed anything.

## Patterns

- All three variants report large get and put latency drops, including on paths their diffs never touched. The likely cause is a shared shift in the benchmark run or environment, not the code. This cycle's per-op-time comparisons against baseline can't be trusted, except for v003's get-path gain, which has a clear mechanism.
- Hit-rate moves when the policy changes how items are ordered for eviction inside the stash (v001). It barely moves when placement changes only in a rare corner case (v002). The stash holds most of the capacity, so its eviction order is the main hit-rate lever.
- Lookup cost is dominated by the stash scan. v003 shows that most hits are served from target/probing buckets, so checking those first is a free win. Hit-rate and metadata-size stay the same.
- None of the three changes affected metadata-size, and none conflict. They touch independent parts of the path: stash insert position, stash-hit destination, and lookup order.

## Open questions

- Is the stash mostly a holding area for misses, or does it serve real re-reads? v001 suggests demoted items do get reused, but v003 says hits mostly come from normal buckets. How hits split between the two tiers is still unmeasured.
- Promotion placement is still untested as a lever. This includes `get` inserting at Back vs. Front, and target-hit promotions using spare capacity in a neighbouring bucket.
- Would balanced placement at initial `put` time trigger often enough to matter, unlike the stash-hit case in v002?
- The baseline timing needs re-measuring under the current environment before any future latency claim can be trusted.

## Continuation

Combine v001's Back-of-stash demotion with v003's target→probing→stash lookup order as the new parent. They are independent and both are wins. Next, test promotion-placement position and balanced placement on initial put/target-hit paths, plus other stash eviction-order refinements. Drop further stash-hit-path placement tweaks, since v002 showed they rarely trigger. Re-measure baseline latency in the same run as candidates, and only credit per-op-time changes that have a code-level mechanism.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cycle tried three things. v001 combined cycle-1's Back-demotion with the target→probing→stash lookup order. v002 placed stash-hit promotions at the target's MRU end. v003 inserted fresh puts at the stash's LRU end. All three are listed as Winners, but only v001 and v003 actually improved anything. v003 gave the only new hit-rate gain. v002 lowered hit-rate slightly and lengthened the get tail, so it is a negative result, not an improvement.

## Patterns

- Ranking by proven reuse inside the stash keeps paying off. Cycle-1 sent demotees to the stash's MRU end. v003 now sends fresh arrivals to its LRU end, and the two gains add up. The stash is working as a scan-resistant probationary queue, and this workload rewards that.
- The "hot placement" idea only works going down, not going up. Demotions into the stash gain from MRU placement. Stash→target promotions lose from it. In tiny target buckets, a single stash re-read placed at MRU evicts a resident already hit in the target, and it also skips the climb within the bucket that keeps probing selective. Front placement on promotion acts as a useful admission filter.
- The changes compose cleanly. Lookup order and stash insert position are independent: v001 reproduced the expected cache state exactly. Metadata-size did not move in any variant.
- Higher hit-rate costs some latency. More stash hits mean more traffic on the slowest hit path (stash lookup → promote_to_target → possible cascade), which slightly erodes the lookup-order gain on get p50. Front inserts add a small put-p50 cost because the bucket shifts on each insert. Most other latency movements have no code-level cause and look like noise.

## Open questions

- Does v003's fresh-Front put stack on top of v001's combined parent? v003 was built on cycle-1's v001, not on the combined variant. Lookup order shouldn't change cache state, so the hit-rate should carry over, but this hasn't been verified.
- Can the stash-hit → promote_to_target path be made cheaper, now that it carries a bigger share of hits?
- Would requiring a second target hit before promotion into probing protect the hottest tier better?
- The put-path shift cost of Front insertion: is it a real cost or noise?

## Continuation

Make the new parent v001 (lookup order + Back-demotion) plus v003's fresh-put-at-Front. Next, pursue two things: a cheaper stash-hit promotion path to recover get p50, and stricter admission into probing, such as a second-hit requirement. Drop MRU placement for stash→target promotions; v002 showed it hurts hit-rate. Don't revisit lookup order.


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

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>

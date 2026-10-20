# Task: cycle-insight-writer

Cycle: 0003
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0003

## Winners

- v003
- v002
- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0003/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0003/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0003/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0003/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0003/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0003/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.

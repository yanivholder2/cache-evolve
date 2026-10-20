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

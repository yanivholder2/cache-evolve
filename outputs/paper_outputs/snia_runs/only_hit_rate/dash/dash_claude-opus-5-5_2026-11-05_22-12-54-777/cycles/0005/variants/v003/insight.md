# Variant v003 Insight

## Helped

(None. No hit-rate point improved. hm_0 at size 16384, the hypothesis's main target, dropped the least, but it still dropped.)

## Hurt

- hit-rate (hm_0 and web_0, both sizes): In v002, the halving inside `record_hit` on a stash hit was the stash's only ageing. `put_into_stash` and stash eviction never decay counters. `record_hit_saturating` removed that, so items demoted from target/probing now keep their protected-tier counters while they sit in the stash. `lru_tail_lfu_victim` then keeps evicting the fresh and ghost-credited arrivals (counter 0 or 1) in the 8-slot tail instead. The change was meant to protect exactly those arrivals.
- hit-rate (mostly the 131072 points): A stash hit on a demoted item at the cap used to send it to target halved (about 8). Now it arrives at a full 15. With the Front landing, a stale 15-counter item sits inside target's tail window and never loses the min-counter vote, so it holds target/probing slots until another hit there finally ages that bucket. Larger caches demote and re-hit more items that have built up high counters, which fits the bigger drop at size 131072.
- (Not measured: the hypothesis asked for per-op-time, expecting it to be equal or slightly lower because the bucket-wide loop is gone from the stash-hit path. This fitness vector reports only hit-rate.)

## Takeaway

Bucket-wide halving on a stash hit was not a side-effect drain. It was the stash's only decay mechanism, and it also capped the counter a re-promoted item brings back into target. Removing it made demoted items stickier everywhere, which outweighed the ghost credits it saved. The fragility v002 found still needs a fix that keeps stash ageing: for example, age the stash on stash insertion, or reset/halve a demoted item's counter when it enters the stash. Simply skipping the halving is not that fix.

# Variant v001 Insight

## Helped

(none: no trace/size cell improved over the parent cycle/0006/v003)

## Hurt

- hit-rate (hm_0, size 16384): This is the largest regression, the opposite of what the hypothesis predicted. Stash→target promotions land at `Front`, which is inside the aged window. A promoted item or ghost returner usually carries counter 1. It is skipped only on the eviction it caused itself, and the next eviction in that bucket drops it to 0. That removes the one-unit edge the ghost credit and the stash hit gave it in `lru_tail_lfu_victim`, which is the very population the change was meant to protect. With a small cache, a bucket evicts several times between hits, so decrements outpace hit accrual. Counters across the 8-slot window collapse toward 0, and the LFU tail scan loses the signal it uses to tell residents apart.
- hit-rate (hm_0, size 131072): Same mechanism, but milder. A larger cache means fewer evictions per bucket per unit of reuse, so survivors lose less protection between hits.
- hit-rate (web_0, both sizes): A small dip where the hypothesis predicted no change. The decrement charges the normal tier for every eviction, whether or not the incoming item is likely to be reused. On this scan-heavy trace, one-touch promotions keep triggering evictions that wear down genuinely reused residents. Those residents then lose tail scans they used to win. The loss compounds with v003's halving: a survivor aged here and later demoted reaches the stash with an even lower count.

## Takeaway

A CLOCK-style decrement only works when insertion lands outside the ageing window. Here promotions enter at `Front`, so the tail window is where new arrivals sit. Skipping the newcomer protects it for just one eviction, and the decrement then mostly strips the counter-1 credit from recent ghost and stash returners. The cost is also tied to how often evictions happen rather than to hits. That makes it hit small caches hardest, which is the reverse of v003's demotion-halving: that change aged only the item already judged cold. Any future normal-tier ageing should either exclude recently promoted slots or be triggered by something other than eviction count.

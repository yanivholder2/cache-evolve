# Variant v002 Insight

## Helped

- hit-rate (web_0 @ 131072): This is the only cell that improved. At this size, 40% of the main area is a probation segment of tens of thousands of entries. That is long enough for web_0's medium-range-reuse keys to get their second hit and be promoted before they reach the probation LRU. This is the mechanism the hypothesis expected, but it shows up at the large size, not the small one it targeted.

## Hurt

- hit-rate (web_0 @ 16384): The targeted cell got worse, which falsifies the claim that a short probation segment causes the small-size gap. Doubling probation at 16k does not hold medium-reuse keys long enough for a second hit. The change also takes room from protected (in `try_new` and in `climb()`), so keys that are promoted are demoted again sooner. Demoted keys go back to probation, where they are the main-area victim. The net effect is more churn among keys that have already proven themselves.
- hit-rate (hm_0 @ 16384): A slight loss. This is the risk named in the hypothesis rationale: with a 60% cap, fewer of hm_0's frequent keys fit in protected. The overflow is demoted into probation, where it is exposed as the main-area victim.
- hit-rate (hm_0 @ 131072): A larger loss than at 16k. Here hm_0's reuse set fit inside the old 80% protected segment. Shrinking the cap demotes part of that set into the eviction path. The TinyLFU duel only compares the candidate against the single victim. It does not prevent the protected segment itself from shrinking.

## Takeaway

Moving the probation/protected split from 80/20 to 60/40 is close to neutral overall. It helps only web_0 at the large size and costs the small-size web_0 cell it was meant to fix. So the split is not the lever for the small-size web_0 gap. Of the two structural causes named in cycle 0006's rationale, the main-area split is now ruled out and admission (the duel/sketch) remains. The 80% protected cap should stay. If anything, hm_0's losses at both sizes argue for keeping protected large.

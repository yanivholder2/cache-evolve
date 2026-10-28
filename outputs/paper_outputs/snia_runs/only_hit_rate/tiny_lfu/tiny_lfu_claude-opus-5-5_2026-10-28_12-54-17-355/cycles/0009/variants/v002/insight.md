# Variant v002 Insight

## Helped

## Hurt

- hit-rate (hm_0, both sizes): hm_0 was the target, and it regressed instead of improving. Protected-overflow demotees are entries that have already been hit at least twice. Pushing them to the probation head with `push_front` makes them the first thing the victim sample sees. So the cache now throws out entries with proven reuse to give unproven new admissions a longer stay. On hm_0, a demotee's next reuse is evidently more likely than a fresh admission's second hit.
- hit-rate (web_0, both sizes): web_0 also regressed, even though the hypothesis claimed the admission contest was untouched. That claim is wrong. `sampled_main_victim()` returns the minimum estimate among the 4 head nodes, and that minimum is the bar in `cand_freq > victim_freq`. With demotees now sitting at the head, the sample is mostly high-count entries, so the minimum rises. The admission bar therefore tightens, which undoes the very mechanism that gave the parent its web_0 gain.

## Takeaway

Where demotions land in probation is not independent of admission. With a min-of-head-sample victim, whatever sits at the probation head sets the bar that window candidates must clear. Putting already-proven demotees there both raises that bar and sacrifices entries with reuse history, so all four cells drop. If a future change wants to lengthen new admissions' probation stay for hm_0, it should not populate the sampled head with high-frequency entries. One option is to keep demotees at the MRU end and instead change where *winners* land, or how far the sample reaches.

Note: this was written from the hypothesis, the parent's insight, and both fitness vectors. The overlay source could not be listed or read, so the code-level explanations assume the change was implemented as H02 describes.

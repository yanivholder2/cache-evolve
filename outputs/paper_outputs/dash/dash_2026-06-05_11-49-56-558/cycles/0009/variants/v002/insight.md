# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Adding the `lfu_counter >= 2` promotion gate on top of the 7/8 geometry produced a negligible regression, because at this tier width the bottleneck is total normal-slot capacity rather than probing-slot quality — delaying correct promotions by requiring a second access costs a small number of hits without reclaiming useful probing slots from genuinely cold items.

## Takeaway

The two-hit gate and the 7/8 geometry are independent but not additive: the gate's value comes from filtering spurious promotions when the normal tier is narrow and probing slots are scarce; once the tier expands to 7/8 (28 normal buckets), items reaching the promotion threshold are already strongly hot, so the extra evidence requirement only slows them into probing without improving slot quality. The combined result matches v001 within noise, confirming that at the 7/8 geometry the remaining loss site is capacity, not selectivity, and a stricter promotion gate cannot recover hits that were never lost to bad promotions.

# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Lowering the small-queue promotion gate from `freq > 1` to `freq > 0` admitted single-access items into main, diluting it with lower-quality entries that the second-chance eviction loop in `evict_main()` then had to churn through — slightly reducing the effective residence time of genuinely reused items without a compensating increase in ghost-hit promotions.

## Takeaway

The halved ghost window introduced in v003 already extracts the maximum value from the ghost-precision mechanism; adding a more permissive small-queue promotion threshold on top does not amplify it. On this workload, items that exit small with exactly one access (freq = 1) are not meaningfully more likely to be re-requested than items with zero accesses, so routing them into main increases main pressure without increasing hits. The two mechanisms are orthogonal here rather than synergistic: ghost precision is only as useful as the quality of items eventually evicted from main, and weakening the admission filter degrades that quality.

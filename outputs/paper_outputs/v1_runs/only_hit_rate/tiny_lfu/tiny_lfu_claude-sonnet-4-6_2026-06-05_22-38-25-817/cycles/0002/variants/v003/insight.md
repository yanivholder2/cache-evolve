# Variant v003 Insight

## Helped

## Hurt

## Takeaway

Adding `PROMO_THRESHOLD = 2` — so a probationary item must have a sketch estimate strictly above 2 before crossing into protected — produced no measurable change in hit rate on this trace. This tells us that on workload-a, borderline-frequency items (those with exactly 2 observed accesses) are not a meaningful source of protected-segment churn: either they are rare enough relative to the working set size to be inconsequential, or refreshing them within probationary (as the new else-branch does) is functionally equivalent to early promotion because they accumulate enough frequency to earn promotion shortly after anyway. The promotion gate is a sound concept but the threshold of 2 lands in a dead zone for this specific trace.

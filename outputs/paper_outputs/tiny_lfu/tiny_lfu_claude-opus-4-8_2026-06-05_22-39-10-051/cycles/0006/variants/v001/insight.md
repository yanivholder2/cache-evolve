# Variant v001 Insight

## Helped

- hit-rate: Replacing the single-victim `prob_head` pick with `sample_prob_victim` (5-node frequency-ranked scan of the probationary LRU) lets eviction skip a head node that is incidentally hotter than a nearby probationary neighbor, nudging hit-rate a hair above the v002 parent.

## Hurt

- hit-rate: Stacking the sampling victim choice on top of v002's promotion-gate contest produced no compounding — the combined gain over the parent is within fourth-decimal noise, so the hypothesized "disjoint rare cases that add" did not materialize.

## Takeaway

On workload-a at size 2048, victim sampling and frequency-gated promotion overlap rather than stack: both mechanisms intervene on the same scarce "head hides a hotter key" events, so grafting v001's sampler onto v002 buys essentially nothing beyond v002 alone. The two refinements are redundant on this trace, not additive — the compounding bet does not pay off.

# Variant v003 Insight

## Helped

- hit-rate: A marginal gain over the parent, consistent with the ref bit keeping recently touched items out of demotion. The effect is tiny, so the second-chance victim choice barely changes which items get evicted on this trace.

## Hurt

- per-op-time: Both get and put median and tail latency got worse than the parent. Every get hit now writes the ref bit, and victim selection scans the bucket and clears bits on skipped slots, so work was added to the hot path and to eviction.

## Takeaway

A CLOCK-style second-chance bit layered on top of the second-hit admission filter bought almost no hit-rate, because admission already screens out most one-touch items. It cost a clear amount of latency, since it adds a write on every hit and a bucket scan at eviction. Packing the bit into spare bits kept metadata-size unchanged, but per-op-time is the price of the extra recency signal.

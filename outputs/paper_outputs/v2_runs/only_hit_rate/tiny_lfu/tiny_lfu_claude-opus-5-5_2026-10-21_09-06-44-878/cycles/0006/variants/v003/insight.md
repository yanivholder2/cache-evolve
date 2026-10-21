# Variant v003 Insight

## Helped

## Hurt

## Takeaway

On workload-a, choosing which protected entry gets demoted from frequency rather than recency had no effect on hit-rate. The two layers below it already handle hot keys. A demoted key goes to the probation MRU end, so it has to pass every other probation entry before it reaches the 8-entry victim sample. Even inside that sample, `select_victim`'s min-estimate scan skips it if its lifetime count is high, which is exactly the kind of key H03 meant to protect. As a result, `select_demotion` changes which key is demoted but almost never which key is evicted. It also adds up to 4 sketch estimates to every promotion that overflows, and only hit-rate is scored, so that cost doesn't show up. The demotion policy is not a useful lever while probation keeps second-chance placement and frequency-aware victim sampling. Further gains have to come from changing what enters probation or how counts age, not from which protected entry leaves.

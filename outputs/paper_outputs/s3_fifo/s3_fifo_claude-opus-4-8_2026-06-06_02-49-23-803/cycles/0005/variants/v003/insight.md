# Variant v003 Insight

## Helped

- hit-rate: recording the zero-frequency main victim in the ghost queue (the `else { break; }` arm of `evict_main()`) lets proven-but-aged main residents be fast-re-admitted on reaccess, nudging hit-rate above the parent that dropped those victims without trace.

## Hurt

- per-op-time: routing main victims through the ghost adds an extra `pop_back`/`push_front` churn per main eviction and keeps the fixed-cap ghost full more often, so put p50/p99 and get p50 each tick up slightly.

## Takeaway

Populating the ghost from `evict_main()` (not just `evict_small()`) confirms main victims are worthwhile re-admission candidates: holding the ghost length fixed at 50% of main, swapping in proven aged keys alongside small-demotion keys buys a small hit-rate gain. The lift is marginal, suggesting workload-a has only a thin tail of reused-then-aged main residents, paid for with a minor latency churn cost.

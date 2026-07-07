# Variant v001 Insight

## Helped

- hit-rate: The counter-decay fallback breaks warm-lock — when no cold candidate exists, decrementing every item's counter by 1 restores the cold-first eviction path's ability to target newly-demoted items rather than falling back to positional LRU, recovering selectivity that the parent lost in fully-warmed buckets.

## Hurt

## Takeaway

Warm-lock is a real but infrequent failure mode for cold-first eviction on this trace: the decay pass fired rarely enough that the improvement is small, but consistently positive, confirming that cold pool exhaustion does degrade hit-rate in steady state. The one-shot decay-on-lock mechanism is an effective self-correcting patch — it leaves the normal cold-first path completely untouched and only intervenes when the invariant the parent relied on (at least one cold item per eviction) actually breaks down.

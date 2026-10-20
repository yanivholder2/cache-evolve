# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Applying the saturating access counter only to stash-bucket victim selection on top of v001 pulled hit rate below the parent's level instead of restoring the frequency gain, so the gain does not come from stash residents alone.
- per-op-time: The get tail grew even though the hypothesis expected the stash-hit path to stay as cheap as v001, which points to the saturating counter update on each stash hit as the added work.
- per-op-time: Put latency rose on both p50 and p99, which the hypothesis did not predict because target-bucket puts were meant to skip the new scan; the regression therefore likely comes from puts that evict from the stash, where the lowest-counter search now runs.

## Takeaway

Confining frequency-aware eviction to stash residents did not keep the cycle's hit-rate gain on workload-a at size 2048; it made hit rate worse than v001 and left both get and put tails slower, so the frequency signal is not a cheap add-on to v001's deferred promotion. This is one trace, so the result says the stash-only restriction fails here rather than that frequency-aware eviction fails in general.

# Variant v002 Insight

## Helped

- hit-rate: Only nominally. Adding a 64-entry FIFO ghost of `freq == 0` keys evicted from `main` (`capacity / 32`) lets those keys skip the 25-slot `small` probation and go straight back into `main` when re-put soon after. The movement is about as small as the metric can show, so on workload-a at size 2048 very few `main` victims come back within 64 `main` evictions. `small_target`, `ghost_cap`, `MAX_FREQ`, `get` and the `small` branch of `evict_one` are unchanged from v003. That means the main ghost is the only cause, and it found almost nothing to recover.

## Hurt

(No measured dimension regressed. Main-ghost hits re-enter `main` with `freq = 0` and no starting credit. Each one forces an eviction at the head of `main` and then gets a full lazy-reinsertion pass. That costs about the same `main` space it saves, which helps explain why the net effect is roughly zero rather than clearly positive or negative.)

## Takeaway

On workload-a, `main`'s eviction of zero-frequency keys already rejects keys almost correctly. The idea that `main` evicts keys "too early", which the SIEVE result in cycle 8 suggested, cannot be recovered with a short re-admission window. Feeding a ghost from `main` evictions can be treated as closed, as H02's own reading rule says. A larger main-ghost window or giving main-ghost hits a starting credit would only loosen admission, and the history says loose admission hurts. The remaining lever is how ghost-admitted keys are credited once they are in `main`, not which evictions feed a ghost.

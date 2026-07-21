# Variant v003 Insight

## Helped

- **hit-rate**: Tightening `prob_cap` from `capacity/8` to `capacity/10` keeps a larger fraction of working-set items in the protected tier, reducing the rate at which hot entries are demoted and must re-earn their place through probationary hits.

## Hurt

- **per-op-time**: The lazy-demotion mechanism adds a flag-check branch and an `is_pending_demotion()` test on every probationary graduation in `get()`; with the tighter 90% quota, overflow occurs more frequently, so this branch is taken more often and the additional bit-OR/AND operations (plus the eager-fallback full demotion on consecutive graduations) raise get p50.

## Takeaway

Deferring the linked-list pointer work from `get()` to `put()` via `DEMOTION_FLAG` does not eliminate get-path cost — it replaces two pointer updates with a conditional bit check and a bit-OR, and adds an eager-fallback code path for back-to-back graduations, which together still increase observed get latency. The tighter quota itself is beneficial and improved hit-rate, confirming the directional value of 90% protected; the bottleneck is the residual per-get overhead of the demotion-flag protocol rather than the quota level.

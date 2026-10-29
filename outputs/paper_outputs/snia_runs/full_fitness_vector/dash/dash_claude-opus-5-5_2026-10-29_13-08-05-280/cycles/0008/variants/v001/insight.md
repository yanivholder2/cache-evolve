# Variant v001 Insight

## Helped

- hit-rate: The large-cache hm_0 cell is the only one that moved meaningfully. It went back up to the level v002 reached when it dropped the probing evictee. In big caches, the demoted probing LRU resident was rarely reused, so dropping it outright cost nothing. The slot left free in the target bucket gives the next insert room without a put-side eviction. The other three cells changed only at noise level, so the hypothesis's hit-rate floor holds and its refutation condition (that the swap's retention mattered) is not met.

## Hurt

- per-op-time (get and put, p50 and p99): All four latency figures got worse by a similar large factor, put p50 included. That breaks the hypothesis's latency confirmation. The change only touches the promoting branch of the get-path `is_hottest` arm, and that branch is already throttled by `PROBE_WINDOW`. Swapping a two-bucket swap for an eviction plus a move takes away writes; it doesn't add any. It also can't explain a near-uniform shift that includes the median of an untouched put path. This matches the "uniform shift = environmental" pattern the hypothesis flagged for v002. The latency effect of the change is still unmeasured, not shown to be a regression.

## Takeaway

On the window-throttled lineage, giving up the probing LRU resident instead of swapping it into the target slot doesn't hurt hit-rate. It slightly helps the large-cache hm_0 cell. That confirms the swap's "keep the demoted resident" behaviour was not worth anything once `PROBE_WINDOW` gates promotions. The hoped-for tail-latency win from leaving a free slot is still unproven, because this run's timings look shifted by the environment. The variant should be re-timed, or compared against a re-run of v003 on the same host, before anyone concludes it has a latency effect. Caveat: I could not list or open the overlay source in this session, so the code-level reasoning assumes H01 was implemented as written.

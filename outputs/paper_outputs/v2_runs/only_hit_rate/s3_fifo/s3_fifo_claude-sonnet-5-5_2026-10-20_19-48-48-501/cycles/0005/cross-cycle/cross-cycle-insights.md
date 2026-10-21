## Recurring wins

- Capacity/sizing levers beat rule-level tweaks: shrinking the small queue was the clearest gain, but returns flattened quickly and are now exhausted.
- Stricter ghost *length* (shorter ghost) helped or was neutral; ghost length acts as an admission filter. Combined with the small-queue shrink, gains are not additive and are tiny.
- Adding ghost coverage of main-queue victims (cycle 1) was a safe, marginal gain and remains in the base.
- Since cycle 3, the best variant at best ties its parent; all gains are small and may be within noise.

## Recurring failures

- Loosening admission (looser promotion, pre-protected or boosted ghost re-admits, longer ghost) hurts by letting weakly-reused keys pollute main.
- Ageing or decay changes in main (faster freq decay, halving, lower freq cap) lost every time; this workload rewards keeping reuse history.
- Filtering or penalising ghost hits hurts most; ghost hits are strong reuse evidence.
- Extra credit for ghost-admitted keys (grace pass, freq boost, per-key ghost flags) lost slightly and added state with no signal.
- Stricter promotion gate and main insertion-position tweaks are null or possibly no-ops; behavioural change was never verified.

## Open questions

- Did the null-result variants (stricter promotion gate, protected-end insertion) change behaviour at all, or were they no-ops? Verify diffs alter behaviour.
- Do any gains hold on a second trace and a different cache size? All evidence is one trace at one size.
- Can metadata-size and per-op-time be restored? They have been missing every cycle, so cost claims are unverified.
- Is S3-FIFO at a local optimum here? Untested levers: main-eviction scanning, reinsertion rules, and structurally different designs.
- Does a very small probation window weaken filtering on scan-heavy traces?

## Per-policy notes

- S3-FIFO (only policy so far): the baseline's strict filtering is well tuned for the LIRS workload. The current base is main-victim ghosting plus a small queue much smaller than baseline plus a shorter ghost. Treat the latest tied variant as a neutral base and prefer the simplest parent.
- Ghost-hit path should stay permissive; do not add bonuses or filters there.
- Stop tuning capacity ratio, ghost length, promotion thresholds, and ageing; they are saturated or negative.

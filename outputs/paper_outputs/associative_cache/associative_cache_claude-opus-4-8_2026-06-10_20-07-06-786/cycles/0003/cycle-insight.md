# Cycle 0003 Insight

## Cohort outcome

This cycle probed the inherited segmented-LRU (v003 of Cycle 0002) along
three axes: probationary eviction order (v002), equal-frequency tiebreak
granularity (v001), and a protected-segment cap with LRU demotion (v003).
The Winner, v002, reversed probation to evict the *oldest* zero-count entry —
making probation a FIFO queue — which recovered rising keys the prior
newest-first churn was discarding.

## Patterns

- Within the segmented bucket, *which* probationary item is evicted is itself
  a high-leverage lever: keeping the same scan but reversing it to oldest-first
  gives a fresh key a full lap to earn its second-hit promotion, and that
  alone lifted hit-rate with no change to admission, promotion, or metadata.
- Two of three changes were inert on hit-rate. The equal-frequency tiebreak
  (v001) and the protected cap (v003) left retention identical — true recency
  and the position proxy pick the same victims, and the protected segment
  never saturated, so its cap-and-demote branch never fired.
- The inert changes split on cost: v001's reorder-free path is a structural
  per-op *saving* (removes the per-hit shuffle), while v003's cap check is a
  structural per-op *cost* (extra linear pass per promotion) with no offset.
- Consistent with prior cycles, the trace remains collision-bound: the gain
  came from giving rising keys a promotion window, not from recency or
  staleness signals, which stayed neutral.

## Open questions

- Is the FIFO-probation win compatible with v001's reorder-free tiebreak — can
  the per-op saving be banked alongside the hit-rate gain?
- Does any workload regime ever saturate the protected segment, making v003's
  cap relevant, or is it permanently dead code on this trace?
- With probation order now tuned, what is the next hit-rate lever — segment
  sizing, or is the FIFO+promote-on-second-hit scheme near its ceiling here?

## Continuation

Adopt v002's oldest-first (FIFO) probation as the new baseline — it is the
strongest scan-resistant scheme so far. Pursue composing it with v001's
reorder-free tiebreak to capture per-op savings without risking hit-rate, and
push on segment-sizing as the remaining hit-rate axis. Drop the protected-cap
demotion: it is inert on this trace and only adds promotion-path scan cost.

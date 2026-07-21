# Variant v001 Insight

## Helped

- **hit-rate**: Shrinking `prob_cap` from `capacity/4` to `capacity/8` expands the protected queue's share of capacity from 75% to 87.5%, reducing the rate at which moderately-frequent Zipfian items are demoted back to probationary and subsequently evicted, which lifted the hit rate.

## Hurt

_(no tracked dimension summary degraded)_

## Takeaway

Halving the probationary quota confirms that the hit-rate gain first observed in cycle/0005/v003 is a real effect of the quota policy, not a measurement artefact: the improvement reproduces cleanly against v002's non-zero p50 baseline. More importantly, `per-op-time` is identical to the parent, settling the open question from cycle 0005 — the apparent latency cost in v003 was noise introduced by its parent's anomalous zero-floor p50, not a genuine consequence of reducing probationary capacity.

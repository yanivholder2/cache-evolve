# Variant v002 Insight

## Helped

- per-op-time: Put p99 moved slightly lower than the parent, consistent with H02's aim of removing repeated victim-frequency reads from `put()`. The gain is small and comes from a single run, so it is not separated from noise.

## Hurt

- per-op-time: Get p99 moved slightly higher even though H02 did not touch the get path, so this is most likely run-to-run noise rather than an effect of the change.

## Takeaway

H02 was meant to be a decision-preserving put-path speedup, and the hit-rate and metadata dimensions stayed flat, which fits that intent. The put p50 did not move, so the only put signal is a small p99 shift from one run; H02 asked for at least three repeated put timings and the evidence does not include them. The overlay source could not be read in this session, so I could not confirm whether the victim estimate was actually read more than once in `put()` (the hypothesis's no-op case) or whether the local-variable change landed as described.

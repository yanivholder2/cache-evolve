# Variant v003 Insight

## Helped

- (none): No measured dimension improved. The doorkeeper rejects first-seen keys on eviction-needing inserts, which was meant to filter one-hit wonders, but that did not pay off on this trace.

## Hurt

- hit-rate: Rejecting the first insert of every key that needs eviction means a key that would have been reused soon is still missed. The 256-bit per-segment filter also saturates quickly and is wiped at 50% fill, so its "seen twice" memory is short and noisy. Collisions between the two derived hashes admit and reject the wrong keys, which pushed hit-rate below the baseline.
- metadata-size / per-op-time: The fitness vector has no entry for either dimension, so the variant's cost for the extra 32 bytes per segment and the extra hash and bit work on put cannot be judged. The missing data is itself a gap, because the evaluation can't confirm the change is cheap.

## Takeaway

A small periodically-reset doorkeeper placed only on the eviction path loses hit-rate on this LIRS workload. The workload seems to reward admitting on first touch, or the filter is too small and resets too often to hold useful frequency information. Admission gating at segment level needs a larger or decaying filter, or a second-chance rule for rejected keys, before it can beat unconditional admission.

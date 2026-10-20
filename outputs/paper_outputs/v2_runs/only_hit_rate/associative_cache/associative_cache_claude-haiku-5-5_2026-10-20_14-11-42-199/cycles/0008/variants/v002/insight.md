# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Pruning non-resident count-1 keys from `freq` once it passes 4x capacity erases one access of history for each such key, so when the key returns its admission-gate count restarts at 1 instead of 2 and the `incoming <= victim` check can bypass an insert that v001 would have admitted.

## Takeaway

Discarding singleton history from the shared frequency map is not free: the only observable effect is on the admission gate, and it showed up as a small hit-rate decrease rather than the metadata saving the hypothesis predicted. The fitness vector has no metadata-size dimension, so the memory benefit that motivated the change is unmeasured and the hypothesis's central claim remains unverified. The prune also has no hysteresis, so whenever the surviving map is still above the threshold after a pass (for example, mostly count-2+ keys), a full `retain` runs on every access.

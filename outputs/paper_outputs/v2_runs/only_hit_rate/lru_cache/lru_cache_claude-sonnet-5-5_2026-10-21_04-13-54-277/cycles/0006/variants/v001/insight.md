# Variant v001 Insight

## Helped

- (none): No dimension improved; hit-rate, the only dimension, did not gain over the parent.

## Hurt

- hit-rate: Not counting rejected misses in the sketch (and admitting on ties with `>=`) slightly lowered hit-rate. Keys that return after a rejection no longer accumulate the frequency that would let them pass the gate, so genuinely recurring keys are kept out longer, and tie-admission does not offset this.

## Takeaway

The parent's retry-based frequency build-up from rejected misses was helping rather than hurting: it lets recurring keys earn admission, while the scan keys the hypothesis wanted to starve were already mostly filtered by the segmented LRU. Tightening the sketch update on the admission gate moves in the wrong direction, so the gain is unlikely to come from further admission-gate tuning.

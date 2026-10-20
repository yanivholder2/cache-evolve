# Variant v002 Insight

## Helped

- hit-rate: Growing the protected segment from 3/4 to 7/8 of capacity leaves re-referenced keys more room. Ghost-gated LRU-end insertion already keeps scan keys in a small probation slice, so shrinking that slice cost little. The gain is small.

## Hurt

- per-op-time: get p50 rose slightly and put p99 improved. The change only alters a capacity constant, so I don't see a mechanism for it in the code. It is most likely measurement noise, not an effect of the 7/8 split.

## Takeaway

With ghost-gated cold insertion in place, probation can shrink to about 1/8 of capacity and hit-rate still rises a little, so the protected/probation split is a weak, low-risk tuning knob. Changing the constant doesn't touch metadata size or the code path, so those dimensions stay put. The shift is too small to tell apart from noise in timing. I could not read the overlay sources in this session, so this rests on the hypothesis text and the fitness vectors.

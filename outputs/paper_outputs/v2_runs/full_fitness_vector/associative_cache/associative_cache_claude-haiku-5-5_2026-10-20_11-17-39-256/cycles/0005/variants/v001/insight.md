# Variant v001 Insight

## Helped

- per-op-time: The early return on the first zero-counter slot shortens the put-side minimum scan in the frequency policy, so the median put moved in the right direction, though only slightly.

## Hurt

- per-op-time: The tail latency of put and the get path both moved the wrong way, but the change touches neither the get path nor anything that could lengthen the put scan, so this most likely reflects timing noise rather than a cost of the early exit.

## Takeaway

The early exit kept eviction behavior identical to the parent, as the hypothesis predicted, since no hit-rate or metadata change appeared. The gain is small, which suggests zero-counter slots are rare at the moment of a put on this workload, so the scan usually runs to completion anyway. Whether this shortcut is worth keeping depends on how often a zero counter appears at put time, and that should be measured directly before building on it.

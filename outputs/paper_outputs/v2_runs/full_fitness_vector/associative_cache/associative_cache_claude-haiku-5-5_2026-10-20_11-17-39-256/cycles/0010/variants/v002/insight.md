# Variant v002 Insight

## Helped

- per-op-time: Halving the slots per bucket from 32 to 16 (with twice the buckets) halves the put-side victim scan, and the put median and tail both came down, which is the direction the hypothesis predicted for the put path.
- per-op-time: The get tail (p99) tightened under the narrower buckets, consistent with each get probing a shorter linear scan in the one bucket it touches, even though the get median did not follow.

## Hurt

- per-op-time: The get median rose slightly rather than falling as the hypothesis predicted, so the shorter get scan did not translate into a faster typical lookup; the cost on the get path appears to be dominated by something other than scan width at this bucket size, or the difference is within run-to-run noise.

## Takeaway

Narrowing buckets is a real lever on the put path: the victim search is the part of put that scales with bucket width, and shrinking it moved the put tail the most. Hit-rate did not move, so the expected small hit-rate cost of fewer competing slots per key did not show up on this trace. The get-side result is the weak point: the median did not improve, so the get claim in the hypothesis is not supported by a single run and needs the three-run spread the hypothesis asked for before it can be credited.

# Variant v003 Insight

## Helped

## Hurt

- hit-rate: The ghost ring holds only four evicted keys per bucket, and a returning key is only protected if it is still in that ring when it comes back, so on workload-a the midpoint reinsertion rarely fired in a way that improved hits; the net effect was a small regression.
- per-op-time: `put` now runs a linear ghost scan on every miss, plus a `Vec::insert` at the midpoint for ghost hits, and both costs show up in the measured put latency as the hypothesis predicted. `get` has no code change in the overlay, yet its p50 also rose, which the diff does not explain; the two new fields on the bucket may be shifting layout, but this is unverified.

## Takeaway

Bucket-local ghost memory with a four-slot ring did not buy hit-rate on workload-a, and the cost landed on the `put` path as expected. The ring is small enough that it may be expiring returning keys before they come back, so a larger ghost window or a hit-count gate on the midpoint placement is the obvious next thing to test before judging the idea itself. Metadata-size did not move, so the Vec header and ghost buffer were not captured by the existing size probe.

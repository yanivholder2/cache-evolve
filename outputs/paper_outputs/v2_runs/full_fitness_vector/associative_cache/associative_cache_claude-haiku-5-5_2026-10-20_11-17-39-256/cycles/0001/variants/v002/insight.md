# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Front-half hits now move to the midpoint instead of the back, so a re-referenced item in the front half stays inside the eviction-candidate region and is pushed toward eviction by later insertions; the protection H02 assumed for the back half does not extend to these items.
- per-op-time: The get path now does a `remove` plus a mid-vector `insert` for front-half hits, which shifts roughly half the bucket, where the baseline did a `remove` plus a `push` with no shifting; this is the likely source of the get-latency increase.

## Takeaway

Promoting hits to the midpoint instead of the back only protects items that are already in the back half, so it weakens recency for hits landing in the front half, which is where this workload's re-referenced items were. The change did not reduce pollution on workload-a; it lowered hit-rate, which suggests that segmented promotion needs a protected region that a midpoint-landing hit can actually reach before it is evicted.

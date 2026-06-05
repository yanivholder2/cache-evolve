# Variant v003 Insight

## Helped

- hit-rate: Replacing promote-on-get with a per-node reference bit plus a CLOCK second-chance scan let recently-touched keys survive one eviction pass on this LIRS trace, so warm keys were retained more often than strict LRU's move-on-every-access ordering allowed.

## Hurt

- (none): the fitness vector measures only hit-rate, so no other dimension moved.

## Takeaway

On workload-a, approximating recency with a reference bit and a bounded CLOCK hand beats strict LRU promotion: sparing referenced nodes for a second pass protects keys that LRU would have ranked purely by last-touch position, and the gain came without adding a second list or changing the capacity bound.

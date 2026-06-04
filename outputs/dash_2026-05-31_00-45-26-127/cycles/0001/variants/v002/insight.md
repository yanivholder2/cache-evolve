# Variant v002 Insight

## Helped

(no measured dimension improved — hit-rate is the only fitness signal and
it regressed; the predicted per-op-time win from replacing `Vec::remove` +
`push` with a single `Vec::swap` is structurally plausible but not
captured by the fitness vector.)

## Hurt

- hit-rate: bubbling an item only one slot per hit is a weaker LRU
  approximation than strict move-to-back — cold items linger near
  position 0 for many extra hits before drifting off the front, and hot
  items never fully cluster at the back, so the front is a less accurate
  cold-set proxy for `eviction_candidate_index`.

## Takeaway

On Dash buckets of 64 slots, the "lazy bubble-back" approximation costs a
small but real slice of `lirs:multi1` hit-rate: a single-step swap is too
slow to concentrate hot items at the back within the trace's reuse
distances, so eviction at position 0 picks colder-than-LRU victims less
reliably. The lesson for bucket-movement is that ordering accuracy at the
cold end matters per hit, not per many hits — any cheaper-than-O(n)
promotion needs to move warm items faster than one slot at a time, or
combine bubble-back with a tiebreaker at the front, to keep eviction
quality intact.

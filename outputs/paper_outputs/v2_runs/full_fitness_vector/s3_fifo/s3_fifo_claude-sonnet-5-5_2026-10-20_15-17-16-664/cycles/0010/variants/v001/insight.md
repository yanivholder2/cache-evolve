# Variant v001 Insight

## Helped

- per-op-time: The put p99 tail looks lower than the parent's, but get p50/p99 and put p50 are essentially unchanged. Zeroing `freq` on promotion adds no work, so the put tail difference is most likely measurement noise rather than an effect of the change.

## Hurt

- Nothing moved. hit-rate and metadata-size are identical to the parent.

## Takeaway

Resetting `freq` to 0 on small-to-main promotion did not change hit rate on this trace and cache size. Either the stale freq was already consumed or decayed by main's decrement-and-rotate before it affected retention, or promoted objects rarely reach the eviction decision with a distinguishing freq. The "tighter main admission" idea does not translate into a measurable gain here. Note: I could not read the overlay sources in this session (only Read/Write were available), so this rests on the hypothesis and the fitness vectors.

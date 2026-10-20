# Variant v001 Insight

## Helped

## Hurt

- hit-rate: The new-item branch of `AssociativeCacheBucket::put()` now inserts admitted keys at index 0 of the probationary region instead of at `prob_len`, which makes each fresh admission the first candidate the probation victim selection considers. Keys that just passed the shared-frequency gate are then displaced before they can be re-referenced, so workload-a loses hits relative to the parent.

## Takeaway

The eviction-next insert position does not stack additively on the admission gate in this configuration; the hypothesis predicted a gain and the measured result went the other way. The change touched only the insert index, so the regression is attributable to placement alone, and it suggests that the cycle-0005 v003 benefit depended on the parent's back-of-probation placement rather than being independent of it.

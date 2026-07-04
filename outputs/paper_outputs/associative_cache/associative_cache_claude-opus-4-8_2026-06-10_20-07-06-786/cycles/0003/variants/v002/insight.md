# Variant v002 Insight

## Helped

- hit-rate: Reversing `eviction_candidate_index` to evict the *oldest* `lfu_counter == 0` entry (highest index) instead of the front-most one turns probation into a FIFO queue, so a freshly admitted key now survives a full lap through the bucket and gets the chance to register a second hit and be promoted before it can become the victim — recovering the rising keys that v003's newest-first churn was throwing away.

## Hurt

- hit-rate: No regression on the measured dimension; this was a strict improvement over the parent.

## Takeaway

In v003's promote-on-second-hit SLRU, *which* probationary item you evict matters as much as having the probationary segment: evicting the newest entry structurally starves rising keys of a promotion window, and switching to oldest-first (FIFO) probation is what unlocks the extra hit-rate — the same scan, reversed, with no change to admission, promotion, or metadata.

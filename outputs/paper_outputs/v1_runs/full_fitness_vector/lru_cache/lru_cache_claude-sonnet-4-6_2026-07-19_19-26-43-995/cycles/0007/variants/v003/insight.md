# Variant v003 Insight

## Helped

_(no tracked dimension improved)_

## Hurt

- **hit-rate**: Placing demoted overflow-protected items at the LRU end of probationary via `push_front_prob` produced no measurable gain over the parent's MRU-end demotion; with an 87.5% protected quota, overflow demotions are rare enough that their probationary ordering has negligible influence on whether fresh insertions survive long enough to be hit.
- **per-op-time**: The summary rose relative to the parent; `push_front_prob` performs the same number of O(1) pointer writes as `push_back_prob`, so the increase reflects resolution of the parent's anomalous zero-floor `p50_get` rather than any overhead introduced by the change.

## Takeaway

Reordering demoted items to the front of probationary (LRU end) has no effect on hit-rate because at 87.5% protected capacity the protected queue overflows only rarely, making the relative placement of those occasional demoted items within the 12.5% probationary budget inconsequential. The hypothesis's expected benefit — reducing wasted probationary residence by expired-protected items — requires a quota configuration or workload that triggers frequent overflow demotions; at the current capacity split, this mechanism is effectively idle.

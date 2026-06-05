# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Replacing position-based promotion candidacy (`items.len() - 1`) with timestamp-based promotion (`max last_accessed`) caused newly-inserted, never-hit items to win promotion over genuinely-warm items — because a cold item's creation timestamp can be more recent than the last-access timestamp of a hot item that was accessed before the cold item arrived in the bucket.

## Takeaway

Grafting timestamp-based promotion onto the frequency-guard eviction baseline did not compound the gains — it actively regressed them. The root conflict is that `ClassicLRUFreqProtect` guarantees the promotion candidate has been accessed at least once (the MRU-position item is always a moved item), whereas `TimestampFreqProtect`'s max-timestamp scan can be won by a freshly inserted cold item whose creation `Instant` is simply newer than all prior hit timestamps. The two axes are not independent on this workload: the eviction guard succeeds by anchoring decisions in `lfu_counter`, but swapping promotion to a timestamp oracle introduces a cold-item promotion leak that more than cancels the eviction improvement.

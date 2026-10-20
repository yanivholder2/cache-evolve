# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Replacing the MRU-end promotion candidate with the highest-`lfu_counter` item caused recency information to be discarded at the promotion decision point; on this workload the item most recently moved to MRU is a better predictor of near-term probing-tier utility than the item with the most accumulated in-tier accesses.

## Takeaway

Using `lfu_counter` to rank promotion candidates in `ClassicLRUFreqProtect` backfires because the counter and the MRU position diverge exactly in the case where an item's access rate has already slowed — an older high-counter item is passed over by a fresh single access, making it the frequency winner but a recency loser. The `lfu_counter` signal is effective when used on the eviction side (skipping items with counter > 0 in favour of cold insertions), but applying it to promotion selection moves an item whose popularity may be fading into the probing tier's protected space while leaving a currently-hot item behind.

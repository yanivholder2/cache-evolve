# Variant v003 Insight

## Helped

(none: hit-rate fell in every trace × size cell)

## Hurt

- hit-rate (16K, `web_0` and `hm_0`): These are the largest losses, the reverse of the hypothesis's prediction. The new victim rule in `stash_insert` protects any item with `lfu_counter >= 1`. That counter never decays during a stash stay. It resets only on promotion or on re-entry from a normal bucket. So an item with one stale hit is immune forever, and small, contested stash buckets fill up with these items. The few zero-hit slots that remain churn among the newest arrivals. In effect the stash shrinks, and its eviction is no longer based on recency.
- hit-rate (16K, both traces): Items demoted from a normal bucket get `lfu_counter = 0` on entry. Under the new rule they are the first victims, ahead of old one-hit residents. This undoes the protection that `STASH_DEMOTION_POS = Back` gave items that had already earned a normal-bucket slot. The LRU-most zero-hit item can easily be a recent demotee that sits only just behind the pinned items.
- hit-rate (128K, `web_0` and `hm_0`): Same mechanism, smaller effect. Larger stash buckets take longer to fill with one-hit items, so the zero-hit scan usually still finds an old victim. But wherever buckets saturate, the same stale pinning replaces plain LRU.

## Takeaway

A permanent, non-decaying "one hit" flag is a poor eviction signal for the stash. Under the two-hit gate, the stash's job is to filter by recency. Making one-hit items immune until every slot has a hit turns old, half-gated items into long-lived squatters, and it evicts fresh demotees in their place. Sparing items halfway through the gate only works if the protection expires: for example, it applies only within the newest part of the bucket, or the counter decays or is cleared when the item reaches the LRU end. Otherwise, plain LRU stash eviction is clearly better than this victim rule.

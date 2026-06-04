# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Gap-fill insertion drops the fresh item into the same front-half slot that v003's untouched-first filter is most likely to target on the next miss, so newly-inserted keys are evicted before they can register the one hit that would flip `lfu_counter` above zero and earn protection.

## Takeaway

On this trace, push-to-back insertion wasn't an arbitrary default — it was the grace period that made v003's `lfu_counter == 0` filter work, by giving every new key the longest possible window to score its first hit before becoming an eviction candidate. Gap-fill collapses that window because the placement axis and the eviction-selection axis are coupled through `lfu_counter`: landing a fresh (counter-zero) item in the slot the filter scans first means the next miss will often evict the item that just arrived. Inserting at the victim's index only pays off if pairs the eviction filter with a different protection signal — under the untouched-first rule, the back of the bucket is the only safe landing zone for a churn-absorber to grow.

# Variant v003 Insight

## Helped

## Hurt

- hit_rate: `rposition` now kills the *most-recently-inserted* untouched slot first, so a freshly inserted key is evicted before it ever gets the single hit that would promote it (move to back + bump `lfu_counter`), while the genuinely cold leftmost untouched slot — the one ClassicLRU was designed to evict — survives at the front of the bucket.

## Takeaway

The cycle-0003 v002 autopsy's claim that back-of-bucket untouched slots are predominantly one-shot churn is falsified on this trace: those slots are mostly *new* arrivals that still have their first hit ahead of them, and evicting them first short-circuits the move-to-back promotion that ClassicLRU relies on. For Dash-style bucket-movement, the untouched-first filter is only useful when paired with the front-first scan direction — once you have an untouched-vs-touched signal, the LRU front is the correct tiebreaker, not the back.

# Variant v002 Insight

## Helped

- hit-rate (`hm_0` at the large size, and marginally `web_0` at the small size): rejecting a stash item that has fewer hits than a hotter LRU resident of a full target, while also halving that resident's counter, kept some reused target residents in place long enough to be hit again. That is the clearest sign of the intended frequency filtering. It shows up only where the cache is large enough for target residents to build up counters above 1.

## Hurt

- hit-rate (`web_0` at the large size and `hm_0` at the small size): when the gate rejects an item, the item stays in the stash, and every further hit pays the stash path again instead of settling into a normal bucket. Each rejection also halves the victim's counter, which wears down the protection the gate gave. In these settings the gate cost about as much as it saved, so v001's small regression on `web_0` at the large size was not recovered.

## Takeaway

The TinyLFU-style comparison barely bites in this design. Stash→target promotion inserts at `Front`, which is the LRU slot, so the victim the gate compares against is usually the item promoted most recently. That item's counter is low, and halving on demotion keeps counters low as well. The newcomer is bumped before the comparison and wins ties, so it almost always ties or wins. Hot items are also moved off index 0 by the hottest→probing path before they can serve as a strong victim. As a result, v002 behaves almost exactly like v001's first-hit promotion: per-cell moves split both ways and the summary stays flat. That contradicts H02's claim of a gain at both sizes. A frequency gate here needs a victim whose counter reflects real residency. Options are comparing against the bucket's lowest counter and not its LRU slot, or not inserting promotions at the eviction end.

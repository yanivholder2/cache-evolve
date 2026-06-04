# Variant v002 Insight

## Helped

*(no dimensions helped)*

## Hurt

- hit-rate: giving items with high pre-eviction counters a larger promotion credit (up to +4 instead of the flat +2 in v003) caused them to occupy primary slots too long after promotion, crowding out items that were actively accumulating frequency under the current workload pattern.

## Takeaway

Frequency-proportional promotion credit backfired because a high `lfu_counter` at eviction time reflects past popularity, not future access likelihood — items displaced from primary during a burst often do not return to high demand quickly. Over-boosting their counter on promotion locks them into primary across multiple eviction cycles, reducing turnover for genuinely hot items and erasing the gains the fixed +2 credit had achieved. The result shows that differentiated credit calibrated on stale history is less reliable than a uniform head-start sized just enough to survive one additional cold-burst cycle.

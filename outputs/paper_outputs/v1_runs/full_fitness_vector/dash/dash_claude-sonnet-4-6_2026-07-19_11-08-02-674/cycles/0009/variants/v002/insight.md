# Variant v002 Insight

## Helped

_(no dimensions improved)_

## Hurt

- **hit-rate**: Replacing the MRU-position promotion candidate with a max-`lfu_counter` scan caused a small regression, because on this Zipfian trace the item at the back of the bucket (just moved there by `get_from_position` on the most recent hit) is already the strongest short-term predictor of the next access — cumulative frequency added noise rather than signal by occasionally nominating historically-hot but currently-cooling items instead.

## Takeaway

Recency (MRU position) and frequency (`lfu_counter`) converge for truly hot items but diverge for items that were popular in an earlier phase of the trace; on workload-a the MRU position carried more predictive value for the promotion decision, so substituting a max-frequency scan eroded hit rate slightly without touching latency or struct size. The hypothesis assumed recency and frequency could diverge harmfully, but this trace's Zipfian access pattern keeps its hottest items at the MRU end consistently enough that the existing position-based promotion was already doing effective frequency selection.

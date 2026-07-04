# Variant v003 Insight

## Helped

(none — no measured dimension improved over the parent)

## Hurt

- hit-rate: Replacing recency ordering with CLOCK lost ground — leaving hit items in place (only flipping the `lfu_counter` reference bit) instead of move-to-front discards the fine-grained recency signal, so the front-scan eviction hand often clears warm keys that LRU would have kept.

## Takeaway

On this collision-bound, small-bucket trace a single reprieve bit is a coarser substitute for full recency ordering: a CLOCK hand only distinguishes "touched since last scan" vs "not," whereas the parent's positional reordering ranks every item. The hypothesis that a reference bit would shield warm keys better than tier promotion was not borne out — second-chance underperformed plain recency tracking here.

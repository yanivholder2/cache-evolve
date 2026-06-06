# Variant v002 Insight

## Helped

(none)

## Hurt

- hit-rate: Replacing move-to-front reprieve with an in-place backward scan removed the positional protection that kept recently re-accessed items near the head of the main queue, causing warm items sitting near the tail to be evicted prematurely despite their frequency.

## Takeaway

The baseline's move-to-front behaviour in `evict_main()` does more than grant a second chance — it continuously re-sorts the main queue so that eviction pressure falls on the oldest cold items while recently active items migrate away from the tail. The CLOCK-style in-place scan decouples position from recency, which sounds principled but eliminates that implicit recency protection. On this workload, temporal re-positioning of warm items was the primary driver of hit rate, and removing it caused a substantial regression; decrementing in place is not a drop-in substitute for the reprieve.

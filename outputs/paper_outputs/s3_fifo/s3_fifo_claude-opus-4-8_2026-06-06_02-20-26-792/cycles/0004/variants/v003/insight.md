# Variant v003 Insight

## Helped

- hit-rate: Promoting a `main` hit to the front via `remove`+`push_front` (alongside the existing `freq` bump) makes the resident queue recency-ordered, so recently-reused keys cluster at the front and the tail that `evict_main` scans now holds the coldest-by-recency entries — a small net gain over the parent's pure-FIFO main.

## Hurt

## Takeaway

Layering true move-to-front recency on top of the `freq` second-chance counter does retain a few more hot keys than position-blind FIFO, but the lift is marginal — the `freq` counter was already approximating recency well enough that explicit reordering recovers only a sliver. On this LIRS trace the ordering discipline of main is a weaker lever than the partition-ratio sizing that drove earlier gains; the bolder mechanism change paid, but barely.

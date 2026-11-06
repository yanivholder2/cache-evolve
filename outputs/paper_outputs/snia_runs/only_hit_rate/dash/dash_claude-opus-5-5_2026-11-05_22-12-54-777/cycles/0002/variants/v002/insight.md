# Variant v002 Insight

## Helped

- hit-rate: Swapping pure-LRU victim choice for the lowest-`lfu_counter` slot among each bucket's 8 least-recent slots, on the parent's 24 normal / 8 stash topology, raised hit-rate above the parent on all four trace/size cells. The largest gain was hm_0 at size 16384, the cell where the parent's topology change alone did the least. That fits the mechanism. When the cache is too small to hold the working set, choosing which item in the LRU tail to evict matters more than how large the protected tier is. Also, counting stash hits and carrying the counter through promotion lets a reused block that was just promoted beat one-touch blocks near the tail.

## Hurt

(none measured. This variant's fitness vector reports only hit-rate. The hypothesis says metadata-size should stay at 40, because the existing `lfu_counter` field is reused. It also calls for checking per-op-time against baseline's 75, because the victim scan and the bucket-wide halving are on the eviction path. Neither was re-measured here, so the added cost on the eviction path is still unknown.)

## Takeaway

Frequency-aware tail eviction and a larger normal tier fix different weaknesses, and their gains roughly add up. The parent's topology gain was concentrated at size 131072. The frequency filter's gain is concentrated at size 16384, most of all on hm_0. Together they lift every cell, and no cell got worse. The open question is cost, not quality. The next step should measure per-op-time for the 8-slot scan and the halve-on-cap ageing before building more on this combination. A lower cap or a narrower scan window is the obvious knob if that cost turns out too high.

<!-- Note: the overlay source files could not be listed or read in this session (directory listing unavailable). This insight is based on the hypothesis's description of the change plus both fitness vectors and the parent insight. -->

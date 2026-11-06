# Variant v003 Insight

## Helped

(None. Hit rate rose a little only at web_0 131072, and that change is inside noise. metadata-size stayed the same because `SPARED` reuses a high bit of the existing `lfu_counter` and adds no field, so `Item` is still 40 bytes.)

## Hurt

- hit-rate: Hit rate fell in three of four cells, including hm_0 at 131072, the cell the hypothesis named as the decisive check. The summary dropped too. Every change is inside the ±0.001 noise band, but the direction is now mostly negative, where the parent's two-slot window moved every cell up. With the whole-bucket window, nearly every full-bucket eviction finds some never-hit slot, so the spare fires often. It mostly evicts items that just entered stash (new inserts or fresh demotions) before they get any chance at a first hit. Meanwhile the spared once-hit LRU item usually takes no second hit and is evicted at the next insert anyway. The net effect is to swap a young item's chance for an old item's slightly longer stay, which does not pay off on these traces.
- per-op-time: Put p99 rose sharply, and put p50 and both get latencies rose more moderately. The put tail fits the change: the victim path now often runs an O(bucket) scan, then a mid-bucket remove-and-shift and `refresh_stash_mask`, where the parent rarely left the tail-evict path. The get slowdown is harder to tie to the change, since the read path only gained a `& 0x7F` mask. Part of it is probably the same benchmark variance that made the parent look faster than its own parent.

## Takeaway

Widening the stash victim window to the whole bucket, with a protect-once bound, keeps residency bounded (no 0004/0005-style jam). But it gives no hit-rate gain and makes put tails noticeably worse. This is the decisive test from the 0009 plan: hm_0 at 131072 did not rise past noise, so stash victim choice by per-item hit count should be dropped as a lever. The narrow and the wide window both leave hit rate at about 0.535, so the plateau is not set by which stash slot gets evicted.

(Note: Directory listing and search were not available in this session, so I could not find or open the overlay source files. I took the description of the code change from the hypothesis and did not check it against the diff.)

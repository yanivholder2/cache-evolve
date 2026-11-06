# Variant v003 Insight

## Helped

(none measured. This variant's fitness vector reports only hit-rate, and hit-rate did not improve on any cell. Per-op-time was the dimension H03 targeted, but it was not measured here either, so the claimed saving on the eviction path is still unverified.)

## Hurt

- hit-rate: Narrowing the victim scan from 8 to `EVICTION_WINDOW` = 4 slots and lowering the cap from 15 to `LFU_CAP` = 7 lowered hit-rate a little on all four trace/size cells. The overall drop stays inside H03's 0.003 tolerance. The largest loss is on hm_0 at size 16384, the same cell where v002's frequency filter gained the most. That is what you would expect if slots 5–8 of the LRU tail often held the true one-touch victim when the cache is under pressure. With a 4-slot window, the scan falls back to evicting a reused block in those cases. web_0 at size 16384 barely moved, so the faster ageing from the 3-bit cap gave no visible phase-change benefit to offset the loss.

## Takeaway

Halving the window kept most of v002's frequency-filter gain but not all of it. The loss is concentrated on the small-cache, high-pressure cell, so the 8-slot window does contribute there. The tail is not exhausted at 4 slots. On cost, the code change is smaller than the hypothesis suggests. v002's halving already used integer `/= 2` on an unsigned counter, which compiles to the same shift as `>>= 1`. The new `min_by_key` over `(counter, index)` tuples is not cheaper than v002's hand-written strict-`<` loop. And the lower cap makes the bucket-wide ageing pass run about twice as often on hot keys. So any per-op-time gain depends only on scanning 4 fewer slots, offset by more frequent ageing. This variant is not a clear win until per-op-time is actually measured against v002. If per-op-time is measured, test window and cap separately (for example window 4 with cap 15, or window 8 with cap 7) to tell which change caused the hm_0 loss.

<!-- Based on both overlays' src/shared/eviction_policy.rs, both fitness vectors, H03, and the v002 insight. The eval/ directory and the promotion_get stash-hit path could not be listed or inspected in this session. -->

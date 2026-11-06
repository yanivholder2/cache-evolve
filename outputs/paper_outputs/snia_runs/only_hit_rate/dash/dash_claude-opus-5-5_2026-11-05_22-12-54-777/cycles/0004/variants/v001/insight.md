# Variant v001 Insight

## Helped

- hit-rate (hm_0 @ 16384 only): Widening `LRU_TAIL_WINDOW` from 8 to 16 gave a small gain on the one cell where the frequency filter already mattered most, the small cache on hm_0. When capacity is tight, letting the scan reach slots 8–15 sometimes finds a one-touch block that the 8-slot window would have kept, while it evicted an older resident that had been reused.

## Hurt

- hit-rate (web_0 @ 16384, web_0 @ 131072, hm_0 @ 131072, and the summary): The wider window lets a zero-count item in slots 8–15 beat an older slot-0 item with count 1, so fairly recent arrivals get evicted ahead of stale ones. In the 8-slot stash buckets the scan now covers the whole bucket, so recency plays no part in victim choice there. The policy drifts from "LRU tail filtered by frequency" toward plain LFU, using a low-resolution counter that halves on cap. That costs hit-rate everywhere except the tightest hm_0 cell. The summary ends up slightly below the parent, so the hypothesis's claim that hit-rate would rise above v002 is refuted.
- per-op-time / metadata-size: Neither was measured, even though the hypothesis asked for per-op-time against baseline's 75. The diff only changes the constant and the tests, and `Item` is untouched, so metadata-size should stay at 40. Each eviction now scans up to twice as many slots, and that added cost has not been quantified.

## Takeaway

The diff is clean: only `LRU_TAIL_WINDOW` changed, plus the tests, with the cap, the ageing and the strict-`<` tie-break all unchanged. That makes this a clean read on window width. Combined with cycle 0003's finding (cited in the hypothesis) that a 4-slot window lost on every cell, it shows hit-rate peaks near 8: narrower loses the frequency filter, and wider loses recency protection. The one exception is small-cache hm_0, where frequency outweighs recency. That hints the best width depends on cache size more than it calls for a single fixed constant. Otherwise the fixed window-width lever is spent. Further gains should come from elsewhere, such as counter resolution or ageing, or a window scaled per bucket type (stash vs normal), not from widening it again.

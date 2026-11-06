# Variant v003 Insight

## Helped

- hit-rate (web_0, size 16384 only): Decrementing the window's survivors on every eviction gave web_0's small cache a gain within noise, and it is the only cell that did not drop. Faster ageing does retire stale tail counters there, but the effect is too small to show that halve-on-cap was holding web_0 back, which is what H03 claimed.

## Hurt

- hit-rate (hm_0, size 16384): This cell lost the most. In v002 it was the cell the frequency filter helped most. With a small cache, evictions happen almost every miss, so subtracting 1 from all 8 tail slots per eviction pushes those counters back to 0 within a few misses. Once the counts tie at 0, the victim rule falls back to "oldest slot". That is plain LRU, which undoes the filter's gain.
- hit-rate (hm_0 and web_0, size 131072): Both large-cache cells dropped a little. The drop is smaller because evictions are rarer there, so counters drain more slowly. The direction is still the same. Each miss can age up to three buckets: the target eviction in `put_at_aged`, then the stash demotion, then the cascade. Reused items lose protection faster than one extra hit can restore it.
- hit-rate (all cells, mechanism): Stash→target promotions land at `InsertPosition::Front`, inside the 8-slot tail window. A just-promoted item carries a counter of 1 from its counted stash hit, and the target bucket's next eviction ages it straight back to 0. That removes the advantage over one-touch blocks that v002's stash-hit counting was built to give. The hypothesis's "do not age the demoted item twice" rule does not protect against this case.

## Takeaway

Ageing on every eviction is too aggressive for an 8-slot tail filter with a 15 cap. The filter needs counts to outlive several evictions, and a decrement of 1 per eviction erases them on about the same timescale as a single item's tenure in the tail. That is worst in small caches, where the filter mattered most. The result supports H03's own fallback reading: halve-on-cap's slow, rare ageing is part of why v002's filter works. Future ageing changes should slow decay relative to eviction rate, for example by ageing every Nth eviction or exempting newly promoted Front-inserted items, not speed it up. Per-op-time and metadata-size were not re-measured, so H03's claim that the 8-slot decrement is cheaper than full-bucket halving is still untested.

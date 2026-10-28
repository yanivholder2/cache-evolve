## Recurring wins

- Changes that alter the admission contest beat changes that alter eviction order alone. The baseline's weakness is its admission contest. Every winner so far has changed who competes (window), how big the window is (adaptive climber), or which resident is the victim (SLRU).
- Each winning component has been added one at a time, and each has added to the last: a 1% window, then an adaptive window, then an adaptive window with SLRU. Building up W-TinyLFU one component per cycle has worked.
- Structural changes have added to each other more reliably than timing changes. SLRU helped web_0 on its own in two cycles, and in cycle 0003 it was the only change that combined well with the adaptive window.
- The window changes pay off mostly at the small cache size. SLRU and the main-region changes pay off mostly on web_0, especially at the large size, where window tuning does nothing.

## Recurring failures

- Global periodic sketch aging has been tried three ways and has not held up. With a counter cap, it regressed on web_0 at the large size. Cap-free and counted per increment, it hurt hm_0 at the small size. Counted per request on top of the adaptive window, it regressed at the small size and did nothing at the large size. The cause is the interaction: halving wears down the main region's frequency lead once the window gives newcomers room. Drop it on this lineage.
- Periods tied to capacity or fixed as a constant miss at one end of the size range. A sample or reset period tied to capacity is too slow at the large size. A fixed constant is too fast at the small size and lets the climber oscillate, because lazy rebalancing takes about one full turnover to take effect.
- Counter caps combined with strict `>` admission cause ties that favour the victim, so hot newcomers get rejected.
- Metadata-size and per-op-time have been requested in every cycle and have never been measured. So far, every winner has been judged on hit-rate alone.

## Open questions

- Can an adaptive or swept protected/probation split win back hm_0 without giving up web_0's gains?
- Would a climber sample period tied to rebalance completion or cache turnover, or eager rebalancing, make the climber useful at both size ends?
- Could targeted aging fix stale inflated counts on demoted entries without the broad erosion seen so far? Two candidates: aging only demoted protected entries, or aging only when the climber is in a given state.
- Does a different tie-break in admission (`>=` or random) matter now that ties are common and SLRU picks the victims?
- What do the third list, demotions, per-get sampling, and full-table resets actually cost?

## Per-policy notes

- **Baseline TinyLFU:** it rejects first-time keys once the cache is full. Its eviction order was not the bottleneck.
- **Window (fixed 1% → adaptive):** the fixed 1% window was too small at the small size. The hill-climbing window fixed that but stays flat at the large size. Its feedback period is now the main thing to tune on this axis.
- **SLRU main region (80/20):** it consistently lifts web_0, which is scan-heavy with many one-hit keys, at both sizes, and it is the current lineage winner. It costs hm_0 a little, because the short probation segment cuts the stay of recency-driven entries short.
- **Sketch aging:** it helped only before the adaptive window existed, mostly on web_0. On the current lineage it is a net loss.
- **Traces:** hm_0 is the hard trace. A plain LRU already holds most of its reuse set, and almost every variant has dipped it somewhere. web_0 responds most to structural changes in the main region.

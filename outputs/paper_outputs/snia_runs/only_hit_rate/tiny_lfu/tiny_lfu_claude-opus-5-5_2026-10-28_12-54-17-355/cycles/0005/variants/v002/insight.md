# Variant v002 Insight

## Helped

_(none: hit-rate did not improve in any trace/size cell)_

## Hurt

- hit-rate: Every cell fell below the parent, and the small size fell hardest on both traces. That refutes H02's core claim. The controller runs on every non-resident `put()`, not once per sample period, and its step cap (`capacity / 32`) is several times the 1% starting window. So one `ghost_main` hit can throw away many window slots, and the window then swings far on single events instead of drifting.
- hit-rate: The two ghost lists are fed very unevenly. `evict_win` runs on every lost admission contest, which is the common case under strict `>` with a tiny window. `evict_main` runs only when a candidate wins or on a direct evict. So `ghost_win` stays near full and `ghost_main` stays small. The ARC ratio then makes grows tiny (`main/win`, about 1) and shrinks large (`win/main`, hitting the cap). That pins the window near its floor and removes the window growth the parent's climber could make for recency-heavy phases.
- hit-rate: Moving the window back and forth works against lazy rebalancing and weakens the admission filter. When the window target rises, `evict_for_insert` takes the `win.len < window_cap` branch and evicts main victims with no frequency contest. Those evictions feed `ghost_main`, whose hits then shrink the window. When the target falls, the window LRU is spilled into probation, again with no contest. Per-event oscillation pushes far more traffic through these contest-free paths than the parent's per-period climber did.
- hit-rate: The web_0 large cell, which H02 singled out as the place the climber "stalls", lost the most of the large-size cells. In W-TinyLFU, a `ghost_win` hit mostly means "the sketch rejected a key that came back". The returning key's sketch count has already gone up, so the filter will usually admit it next time without help. That signal says little about window size, unlike ARC, where list size alone decides retention.

## Takeaway

ARC-style ghost lists do not carry over cleanly to W-TinyLFU. Window rejections are produced by the frequency filter, not by window capacity, so `ghost_win` floods and the size-ratio delta becomes a one-way ratchet toward a minimal window. Applying the result on every miss also defeats lazy rebalancing and sends traffic around the admission contest. A ghost signal could still be useful, but it needs balanced inputs: either count only window evictions caused by size, or normalise each list by its own insert rate. It should also be applied in batches at sample-period boundaries with a small, bounded step, not per event. The ghost sets add up to `2 * capacity` hashed u64s of metadata, which this eval did not measure, so that cost still needs reporting.

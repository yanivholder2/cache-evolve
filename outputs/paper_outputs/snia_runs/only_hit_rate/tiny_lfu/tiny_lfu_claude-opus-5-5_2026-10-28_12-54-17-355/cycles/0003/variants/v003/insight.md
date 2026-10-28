# Variant v003 Insight

## Helped

- hit-rate: At the large size, web_0 gained slightly. This is the point H03 was aimed at, and it fits the idea that the parent's `sample_period = capacity` (131072) gave the climber too few steps there. With a fixed 4096-request sample and a step floor of about 131 entries, the window finally moved at that size. The gain was small, though, and web_0 is still behind the other cycle-2 variants that the parent insight compared it with.

## Hurt

- hit-rate: Both small-size points (web_0 and hm_0) dropped. At 16384 the change cut the sample from 16384 to 4096 requests, so the climber now judges a step on a quarter of a cache turnover. The lazy rebalance in `evict_for_insert` moves only one entry per insert. Each new retarget therefore lands before the lists have reached the last one, so the climber is scoring window sizes that never fully took effect.
- hit-rate: hm_0 at the large size dropped, which wiped out the parent's modest gain there. At 131072, a 4096-request sample is about 3% of a turnover. The window moves by at least `capacity/1000` on every climb. The EWMA (0.3 weight on the new sample) is compared with the previous EWMA, so a reversal fires only after several steps in the wrong direction. Together, these let the window drift and oscillate instead of settling, and on a trace that already liked its window, that is a loss.

## Takeaway

Making the sample period independent of capacity fixed the "too few climbs" problem but caused a worse one. The climber's feedback loop is now much faster than the cache's response time, because it retargets `window_cap` every 4096 requests while the lazy one-entry-per-insert rebalance and the eviction churn need on the order of a full turnover to show a new window's effect. Under H03's own refutation criterion, the result is mostly negative: the large size improved on one trace and fell on the other, and the small size dropped. The sample period should probably follow how fast the window actually moves, or how fast the cache turns over, rather than a fixed constant. The step floor and the lagging EWMA comparison are additional suspects for the oscillation.

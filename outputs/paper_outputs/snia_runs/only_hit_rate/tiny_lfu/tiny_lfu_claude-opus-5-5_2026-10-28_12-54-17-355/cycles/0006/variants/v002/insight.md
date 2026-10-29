# Variant v002 Insight

## Helped

- hit-rate: the small-cache cells (web_0 and hm_0 at 16384) moved up a little. With fewer resident entries, the 8-node walk still covers a small, old part of probation, and taking the minimum over more candidates finds a slightly colder victim more often. These gains are marginal and do not cover the loss below.

## Hurt

- hit-rate: web_0 at the large size gave back part of the gain v003 got from sampling, and that pulled the summary below the parent instead of above it. Taking the *minimum* of 8 count-min estimates instead of 4 lowers the `cand_freq > victim_freq` bar further. Each `estimate()` is a noisy floor, so the minimum of more samples drifts toward entries that only *look* cold: lightly-referenced but still-useful entries, or entries that only recently aged out of the window. As a result, more weak window candidates win the contest and push out main entries that would have hit again, which is the opposite of the scan resistance web_0 relies on. The direct-evict path (`win.len < window_cap`) amplifies this, because it now also takes the minimum over 8.
- hit-rate: hm_0 at the large size was flat to slightly down. The rationale's warning (that recent probation entries fall inside the sample) did not show up as a clear hm_0 regression. The sample still sits near the LRU end, so a larger sample barely changes hm_0, as v003's insight predicted.
- (cost) Each full-cache insert now does up to twice as many sketch estimates (4 hash rows each) inside `sampled_main_victim()`. This variant's eval has no per-op-time dimension, so it does not measure that cost, but it is paid for no hit-rate gain.

## Takeaway

Sample size for the coldest-of-LRU-head victim is not monotone: going from 4 to 8 nodes does not extend v003's web_0 gain and partly reverses it at the large size. The likely reason is that the minimum over more noisy count-min estimates makes the admission bar too easy to clear. v003's 4-node sample looks close to the sweet spot. Further gains from this mechanism probably need a better victim *estimate* (for example, less noisy counts, or a contest margin instead of strict `>` against a minimum), not a wider sample. A test of 2 or 6 nodes would only refine the sweep, not open a new direction.

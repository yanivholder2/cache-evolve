# Variant v003 Insight

## Helped

(none: hit-rate did not improve at any trace/size point)

## Hurt

- hit-rate: web_0 at 131072 fell the most, and this is the point H03 was meant to fix. Ending each sample after one window turnover makes samples very short while the window is near its 1% start, yet the step is still about 5% of capacity, several times the window itself. Each climb therefore judges a large retarget on a short, noisy hit-rate sample against a tight `REVERSE_TOLERANCE`. Direction reversals then follow noise rather than the real effect of the window size.
- hit-rate: The turnover trigger also fires many times while the cache is still filling (`window_inserts` counts from the first put). Hit rate rises on its own during warmup, so the climber is rewarded for whatever direction it started in and takes several large, slowly decaying steps before steady state. The parent's capacity-length sample mostly skipped this phase.
- hit-rate: hm_0 regressed at both sizes, and web_0 at 16384 dipped slightly. A sample lasts as long as one turnover of the current window, so the climber stays longer at large windows than at small ones. That biases where the window settles, separate from which size actually scores better. A large window also shrinks main, and with it the 80/20 protected segment that v001 relied on. Eager rebalancing then applies each oversized retarget at once: protected entries are demoted in bulk and a block of probation is pushed to the window LRU as immediate contest candidates. Under lazy rebalancing this disruption was spread over many inserts.

## Takeaway

Eager rebalancing removed the lag between a retarget and its effect, but tying the sample length to window turnover left the step size and sample length mismatched. Short samples at a small window, combined with capacity-scaled steps, turned the climber into a noise-driven, warmup-biased walk. H03 is refuted by its own criterion. If a turnover-tied period is tried again, it needs a step scaled to the current window (or a minimum sample of a fixed fraction of capacity), and the climber should be suppressed until the cache is full.

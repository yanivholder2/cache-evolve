# Variant v003 Insight

## Helped

- hit-rate (hm_0 @ 131072): A small recovery, as the hypothesis predicted. Halving the step magnitude on each direction flip, with a floor of `step_init / 8`, lets the window settle faster after the 1024 first step overshoots, so fewer scan entries reach the window on this scan-heavy trace.

## Hurt

- hit-rate (web_0 @ 131072): A small loss that gives back part of v001's gain. The hypothesis assumed web_0 benefits only from the rounds before the first flip. The loss suggests web_0 also needs large moves after reversals to keep following its shifting recency set, and damping takes those moves away.
- hit-rate (summary): Flat to marginally below the parent. The web_0 loss and the hm_0 gain at 131072 are about the same size, and both 16384 cells are unchanged, as expected with damping disabled at capacity ≤ 32_768. Damping therefore moves hit-rate from one large-size trace to the other without adding any net gain.

## Takeaway

Damping on flips is not a free fix for overshoot. At 131072 the same step magnitude that overshoots on hm_0 is useful exploration on web_0. So a single damping rule that ignores the workload only moves the operating point along the web_0/hm_0 trade-off, not past it. The small cells stayed exactly the same, which confirms that the gate is contained. More gains at large size probably need a signal that tells the two traces apart, such as how strongly the hit rate changes after a flip or how scan-heavy the window traffic is, rather than one more fixed rule for the step size.

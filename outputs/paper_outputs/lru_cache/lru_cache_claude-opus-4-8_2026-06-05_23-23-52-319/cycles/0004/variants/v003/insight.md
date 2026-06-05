# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Forcing every brand-new key through the ~32-slot admission window (`window_cap = capacity / 64`) before it may enter main means a key only reaches the SLRU if its *second* touch arrives while still in that tiny window; on this trace most reuses are spaced wider than 32 window insertions, so legitimate keys are evicted from the window before promotion and never get the long probationary residency the parent's 205-slot segment gave them.

## Takeaway

The W-TinyLFU admission window is the wrong lever for this LIRS workload: scan resistance assumes one-hit-wonders dominate, but here the window is far too small relative to the trace's reuse distances, so it filters out genuinely reused keys before their second access rather than just churning dead keys. Shrinking the first-touch holding area from the full probationary segment to 32 slots traded away far more real hits than it saved by blocking pollution — confirming the cross-cycle suspicion that a non-recency admission signal is inert (here actively harmful) on this trace, while the parent's 90/10 ratio tuning remains the productive direction.

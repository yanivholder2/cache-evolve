# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Doubling the ghost queue (`ghost_size = 2 * main_size`) lengthens the window in which evicted keys are remembered, so more re-references become ghost hits. Each ghost hit bypasses the small queue and is pushed straight into `main`, which forces `evict_main()` to run more often and displaces resident entries; the net effect on this trace is a loss, not the gain the hypothesis predicted.

## Takeaway

Widening the ghost window alone did not raise hit-rate on this workload: the extra ghost hits are admitted directly to `main`, and that admission path appears to cost more resident hits than the longer reuse distances recover. Any future ghost-size change should be paired with a change to how ghost hits are admitted, or it will likely keep trading main-queue occupancy for ghost coverage.

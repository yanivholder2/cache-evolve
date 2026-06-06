# Variant v003 Insight

## Helped

- (none — hit-rate did not improve on any axis)

## Hurt

- hit-rate: Routing ghost hits back into `small` at `freq=1` instead of straight into `main` forces a re-confirming reaccess before promotion, but those re-admitted keys are often evicted out of the short small queue before their next hit arrives, losing reuse that the direct-to-main path used to capture.

## Takeaway

Gating ghost re-admission behind a small-queue re-confirmation does not beat baseline: a ghost hit is itself already evidence of reuse, and demoting that signal to `freq=1` in `small` discards it whenever the long-gap reaccess outlives the small queue's residency. The cycle-0001 finding that direct-to-main paths are contention-sensitive does not imply the re-admission path should be throttled — here the throttle costs more hits than the main-queue protection saves.
